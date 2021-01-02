use crate::{Credentials, Mode, Store};
use anyhow::{anyhow, Result};
use async_trait::async_trait;

use bytes::buf::BufExt;
use chrono::{NaiveDate, Utc};
use http::{
  header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
  Method, Request, Response,
};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use tokio::io::{self, *};

use hyper::{
  client::{connect::dns::GaiResolver, HttpConnector},
  Client,
};
use hyper_tls::HttpsConnector;

use secstr::SecUtf8;

use std::{collections::BTreeSet, fmt::Debug};

use tower_service::Service;

use super::{LIVE_URL, SANDBOX_URL};

const SANDBOX_NAMESPACE: &str = "etradesandbox";
const LIVE_NAMESPACE: &str = "etrade";

const API_KEY: &str = "apikey";
const SECRET_KEY: &str = "secret";
const ACCESS_TOKEN_KEY: &str = "access_token_key";
const ACCESS_TOKEN_SECRET: &str = "access_token_secret";
const REQUEST_TOKEN_KEY: &str = "request_token_key";
const REQUEST_TOKEN_SECRET: &str = "request_token_secret";
const REQUEST_TOKEN_CREATED: &str = "request_token_ts";

const REQUEST_TOKEN_URL: &str = "https://api.etrade.com/oauth/request_token";
const ACCESS_TOKEN_URL: &str = "https://api.etrade.com/oauth/access_token";
const RENEW_ACCESS_TOKEN_URL: &str = "https://api.etrade.com/oauth/renew_access_token";

type HttpClient = Client<HttpsConnector<HttpConnector<GaiResolver>>, hyper::Body>;

#[async_trait]
pub trait CallbackProvider: Clone {
  async fn verifier_code(&self, url: &str) -> Result<String>;
}

#[derive(Debug, Clone, Copy)]
pub struct OOB;

#[async_trait]
impl CallbackProvider for OOB {
  async fn verifier_code(&self, url: &str) -> Result<String> {
    let msg = format!("please visit and accept the license: {}\ninput pin: \n", url, );
    io::stderr().write_all(msg.as_bytes()).await?;

    let stdin = io::stdin();
    let mut user_input = String::new();
    io::BufReader::new(stdin).read_line(&mut user_input).await?;

    let result = Ok(user_input.trim().to_owned());
    debug!("got verificaton code: {}", result.as_ref().unwrap());
    result
  }
}

#[derive(Debug, Clone, Copy)]
struct UrlConfig<'a> {
  pub access_token_url: &'a str,
  pub renew_access_token_url: &'a str,
  pub request_token_url: &'a str,
}

impl<'a> UrlConfig<'a> {
  pub fn authorize_url(&self, key: &SecUtf8, token: &SecUtf8) -> String {
    format!(
      "https://us.etrade.com/e/t/etws/authorize?key={}&token={}",
      key.unsecure(),
      token.unsecure(),
    )
  }
}

impl<'a> Default for UrlConfig<'a> {
  fn default() -> Self {
    Self {
      access_token_url: ACCESS_TOKEN_URL,
      renew_access_token_url: RENEW_ACCESS_TOKEN_URL,
      request_token_url: REQUEST_TOKEN_URL,
    }
  }
}

pub struct Session<T: Store> {
  store: T,
  mode: Mode,
  client: HttpClient,
  urls: UrlConfig<'static>,
}

impl<T> Session<T>
  where
      T: Store,
{
  pub fn new(mode: Mode, store: T) -> Self {
    let https = HttpsConnector::new();

    Self {
      store,
      mode,
      client: Client::builder().build(https),
      urls: UrlConfig::default(),
    }
  }

  fn base_url(&self) -> &str {
    match &self.mode {
      &Mode::Sandbox => SANDBOX_URL,
      &Mode::Live => LIVE_URL,
    }
  }

  fn namespace(&self) -> &str {
    match &self.mode {
      &Mode::Sandbox => SANDBOX_NAMESPACE,
      &Mode::Live => LIVE_NAMESPACE,
    }
  }

  pub async fn initialize(&self, key: String, secret: String) -> Result<()> {
    self.store.put(self.namespace(), API_KEY, key)?;
    self.store.put(self.namespace(), SECRET_KEY, secret)?;
    Ok(())
  }

  async fn consumer(&self) -> Result<Credentials> {
    let consumer_key = self
        .store
        .get(self.namespace(), API_KEY)
        .and_then(|r| r.ok_or(anyhow!("secret {}@{} not found.", API_KEY, self.namespace())))?;
    let consumer_secret = self
        .store
        .get(self.namespace(), SECRET_KEY)
        .and_then(|r| r.ok_or(anyhow!("secret {}@{} not found.", SECRET_KEY, self.namespace())))?;

    Ok(Credentials::new(consumer_key, consumer_secret))
  }

  pub async fn invalidate(&self) -> Result<()> {
    debug!("invalidating credentials");
    self.store.del(self.namespace(), ACCESS_TOKEN_KEY)?;
    self.store.del(self.namespace(), ACCESS_TOKEN_SECRET)?;

    self.store.del(self.namespace(), REQUEST_TOKEN_SECRET)?;
    self.store.del(self.namespace(), REQUEST_TOKEN_KEY)?;
    self.store.del(self.namespace(), REQUEST_TOKEN_CREATED)
  }

  async fn request_token(&self, consumer: &Credentials) -> Result<Credentials> {
    debug!("getting a request token");
    let request_token = self.store.get(self.namespace(), REQUEST_TOKEN_KEY)?;
    let request_secret = self.store.get(self.namespace(), REQUEST_TOKEN_SECRET)?;

    let request_token_ts = self.store.get(self.namespace(), REQUEST_TOKEN_CREATED)?.and_then(|v| {
      let b = NaiveDate::parse_from_str(v.unsecure(), "%Y-%m-%d").unwrap();

      let d = Utc::today().with_timezone(&chrono_tz::US::Eastern).naive_local();
      if b.eq(&d) {
        Some(d)
      } else {
        None
      }
    });
    match (request_token_ts, request_token, request_secret) {
      (Some(_), Some(rt), Some(rs)) => {
        debug!("using cached request token");
        Ok(Credentials::new(rt, rs))
      }
      _ => {
        debug!("getting a new request token");
        let uri = http::Uri::from_static(self.urls.request_token_url);
        let authorization = oauth::Builder::<_, _>::new(consumer.clone().into(), oauth::HmacSha1)
            .callback("oob")
            .get(&uri, &());

        let body = send_request(uri, authorization, &self.client).await;
        let creds: oauth_credentials::Credentials<Box<str>> = serde_urlencoded::from_bytes(&body)?;

        debug!("created request token: {:?}", &creds);
        let request_token: Credentials = creds.into();
        self
            .store
            .put(self.namespace(), REQUEST_TOKEN_KEY, request_token.key.unsecure())?;
        self
            .store
            .put(self.namespace(), REQUEST_TOKEN_SECRET, request_token.secret.unsecure())?;

        let today = Utc::today()
            .with_timezone(&chrono_tz::US::Eastern)
            .format("%Y-%m-%d")
            .to_string();
        self.store.put(self.namespace(), REQUEST_TOKEN_CREATED, &today)?;
        Ok(request_token)
      }
    }
  }

  async fn access_token(&self, callback: impl CallbackProvider) -> Result<Credentials> {
    let consumer = self.consumer().await?;

    let access_token = self.store.get(self.namespace(), ACCESS_TOKEN_KEY)?;
    let access_secret = self.store.get(self.namespace(), ACCESS_TOKEN_SECRET)?;

    match (access_token, access_secret) {
      (Some(token), Some(secret)) => {
        debug!("using cached access token");
        Ok(Credentials::new(token, secret))
      }
      _ => {
        let request_token = self.request_token(&consumer).await;
        if request_token.is_err() {
          debug!("restarting full flow because request token has an error");
          return self.full_access_token_flow(consumer, callback).await;
        }

        match self.renew_access_token(&consumer, &request_token.unwrap()).await {
          Ok(access_token) => {
            debug!("using renewed access token");
            Ok(access_token)
          }
          Err(_) => self.full_access_token_flow(consumer, callback).await,
        }
      }
    }
  }

  async fn full_access_token_flow(
    &self,
    consumer: Credentials,
    callback: impl CallbackProvider,
  ) -> Result<Credentials> {
    self.invalidate().await?;

    let request_token = self.request_token(&consumer).await?;
    let auth_url = self.urls.authorize_url(&consumer.key, &request_token.key);
    let pin = callback.verifier_code(&auth_url).await?;

    let access_token = self.create_access_token(&consumer, &request_token, &pin).await?;

    Ok(access_token)
  }

  async fn create_access_token(
    &self,
    consumer: &Credentials,
    request_token: &Credentials,
    pin: impl AsRef<str>,
  ) -> Result<Credentials> {
    debug!("getting an access token");
    let uri = http::Uri::from_static(self.urls.access_token_url);
    let authorization = oauth::Builder::<_, _>::new(consumer.clone().into(), oauth::HmacSha1)
        .token(Some(request_token.clone().into()))
        .verifier(pin.as_ref())
        .get(&uri, &());
    let body = send_request(uri, authorization, &self.client).await;
    let creds: oauth_credentials::Credentials<Box<str>> = serde_urlencoded::from_bytes(&body)?;

    debug!("created access token: {:?}", &creds);
    let access_token: Credentials = creds.into();
    self
        .store
        .put(self.namespace(), ACCESS_TOKEN_KEY, access_token.key.unsecure())?;
    self
        .store
        .put(self.namespace(), ACCESS_TOKEN_SECRET, access_token.secret.unsecure())?;
    Ok(access_token)
  }

  async fn renew_access_token(&self, consumer: &Credentials, request_token: &Credentials) -> Result<Credentials> {
    debug!("renewing an access token");
    let uri = http::Uri::from_static(self.urls.renew_access_token_url);
    let authorization = oauth::Builder::<_, _>::new(consumer.clone().into(), oauth::HmacSha1)
        .token(Some(request_token.clone().into()))
        .get(&uri, &());

    let body = send_request(uri, authorization, &self.client).await;
    let creds: oauth_credentials::Credentials<Box<str>> = serde_urlencoded::from_bytes(&body)?;
    debug!("renewed access token: {:?}", &creds);
    let access_token: Credentials = creds.into();
    self
        .store
        .put(self.namespace(), ACCESS_TOKEN_KEY, access_token.key.unsecure())?;
    self
        .store
        .put(self.namespace(), ACCESS_TOKEN_SECRET, access_token.secret.unsecure())?;
    Ok(access_token)
  }

  async fn do_send<P, B, C>(
    &self,
    method: http::Method,
    path: P,
    input: Option<B>,
    callback: C,
  ) -> Result<Response<hyper::Body>>
    where
        P: AsRef<str> + Send + Sync,
        B: Serialize + Clone + Send + Sync,
        C: CallbackProvider + Clone,
  {
    let consumer = self.consumer().await?;
    let access_token = self.access_token(callback.clone()).await?;

    let uri = format!("{}{}", self.base_url(), path.as_ref());

    let (bare_uri, full_uri, params): (&str, String, Option<BTreeSet<(String, String)>>) = match &method {
      &Method::GET => {
        let qs = serde_urlencoded::to_string(&input)?;
        if qs.is_empty() {
          (&uri, uri.clone(), None)
        } else {
          (
            &uri,
            format!("{}?{}", uri, serde_urlencoded::to_string(&input)?).parse()?,
            Some(serde_urlencoded::from_str(qs.as_ref())?),
          )
        }
      }
      _ => (&uri, uri.clone(), None),
    };

    let authorization = oauth::Builder::new(consumer.into(), oauth::HmacSha1)
        .token(Some(access_token.into()))
        .build(method.as_str(), &bare_uri, &params);

    let body: hyper::Body = match input.clone() {
      Some(v) => match &method {
        &Method::GET => hyper::Body::empty(),
        _ => serde_json::to_vec(&v)?.into(),
      },
      _ => hyper::Body::empty(),
    };

    let req = Request::builder()
        .method(method.clone())
        .header(ACCEPT, "application/json")
        .header(AUTHORIZATION, authorization)
        .uri(full_uri)
        .body(body)
        .unwrap();

    // let req = builder
    debug!("{:?}", req);
    let resp = self.client.request(req).await?;
    debug!("{:?}", resp);
    Ok(resp)
  }

  pub async fn send<P, B, R, C>(&self, method: http::Method, path: P, input: Option<B>, callback: C) -> Result<R>
    where
        P: AsRef<str> + Send + Sync,
        B: Serialize + Clone + Send + Sync,
        R: DeserializeOwned + Send + Sync,
        C: CallbackProvider + Clone,
  {
    let mut resp = self
        .do_send(method.clone(), path.as_ref(), input.clone(), callback.clone())
        .await?;

    if resp.status().as_u16() == 401 {
      debug!("auth error, retrying with invalidated session");
      self.invalidate().await?;
      resp = self.do_send(method, path, input, callback).await?;
    }

    debug!("reading status code");
    let status_code = resp.status().as_u16();
    debug!("reading content type code");
    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .map(|ct| ct.to_str().unwrap_or("application/json"))
        .unwrap_or("application/json")
        .to_string();
    debug!("aggregating body");

    let body = hyper::body::aggregate(resp).await?;
    if status_code / 100 != 2 {
      debug!("non 200 status code, reading error");
      let edata: ErrorData = quick_xml::de::from_reader(body.reader())?;
      return Err(anyhow!("{} (code: {})", edata.message, edata.code));
    }
    debug!("got a successful response");
    match content_type.as_str() {
      "application/xml" => Ok(quick_xml::de::from_reader(body.reader())?),
      "application/json" => Ok(serde_json::from_reader(body.reader())?),
      v => return Err(anyhow!("api responded with unknown content type {}", v)),
    }
  }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ErrorData {
  pub code: isize,
  pub message: String,
}

async fn send_request<S, B>(uri: http::Uri, authorization: String, mut http: S) -> Vec<u8>
  where
      S: Service<http::Request<B>, Response=http::Response<B>>,
      S::Error: Debug,
      B: http_body::Body<Error=S::Error> + Default + From<Vec<u8>> + Debug,
{
  let req = http::Request::get(uri)
      .header(AUTHORIZATION, authorization)
      .body(B::default())
      .unwrap();

  debug!("{:?}", req);
  let resp = http.call(req).await.unwrap();
  debug!("{:?}", resp);
  if resp.status().as_u16() / 100 == 2 {
    let res = hyper::body::to_bytes(resp.into_body()).await.unwrap().to_vec();
    res
  } else {
    vec![]
  }
}

#[cfg(test)]
mod tests {
  use std::net::TcpListener;

  use hyper::Client;

  #[test]
  fn encodes_query_string() {
    crate::tests::init();
    let data = Some(&[("blah", "some things go here"), ("other", "and others go here")]);
    let v = serde_urlencoded::to_string(data).unwrap();
    info!("query string: '{}'", v);
    let n = serde_urlencoded::to_string(None as Option<&[u8]>).unwrap();
    info!("query string: '{}'", n);
  }

  #[test]
  fn encodes_json_string() {
    crate::tests::init();
    let data = Some(&[("blah", "some things go here"), ("other", "and others go here")]);

    let data2: Option<&[(&str, &str)]> = None;

    // let v = serde_json::to_string(&data).unwrap();
    info!(
      "query string: '{:?}'",
      data
          .map(|v| hyper::body::Body::from(serde_json::to_string(&v).unwrap()))
          .unwrap_or(hyper::Body::empty())
    );
    info!(
      "query string: '{:?}'",
      data2
          .map(|v| hyper::body::Body::from(serde_json::to_string(&v).unwrap()))
          .unwrap_or(hyper::Body::empty())
    );
    // let n = serde_json::to_string(&None as &Option<&[u8]>).unwrap();
    // info!("query string: '{}'", n);
  }

  #[tokio::test]
  async fn it_works() {
    crate::tests::init();
    info!("inside the working test");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let client = Client::new();
    let base_url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());

    let th = tokio::task::spawn(async move { server::test_server(listener).await });
    let uri: http::Uri = base_url.parse().unwrap();
    let resp = client.get(uri).await.unwrap();
    let body = String::from_utf8(hyper::body::to_bytes(resp.into_body()).await.unwrap().to_vec()).unwrap();

    assert_eq!("Hello, world!", &body);
  }

  mod server {
    use anyhow::{anyhow, Result};
    use http::{Request, Response};
    use hyper::service::{make_service_fn, service_fn};
    use hyper::Body;
    use hyper::Server;
    use std::{convert::Infallible, net::TcpListener};

    pub async fn test_server(listener: TcpListener) -> Result<()> {
      let server = Server::from_tcp(listener)?;
      let service = service_fn(|req| async move {
        info!("{:?}", req);
        Ok::<_, Infallible>(Response::new(Body::from("Hello, world!")))
      });
      let make_service = make_service_fn(|_| async move { Ok::<_, Infallible>(service) });
      server
          .tcp_nodelay(true)
          .tcp_keepalive(None)
          .serve(make_service)
          .await
          .map_err(|e| anyhow!("{}", e))
    }
  }
}

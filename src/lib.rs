#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;

use std::{
  collections::{BTreeSet, HashMap},
  sync::Arc,
};

use anyhow::Result;
use async_trait::async_trait;
use secstr::SecUtf8;
use std::sync::Mutex;
use strum::EnumString;

pub mod accounts;
pub mod alerts;
pub mod market;
pub mod options;
pub mod orders;
mod session;
pub mod transactions;

#[cfg(all(feature = "keychain", target_os = "linux"))]
mod linux;
#[cfg(all(feature = "keychain", target_os = "linux"))]
pub use linux::KeychainStore;

#[cfg(all(feature = "keychain", target_os = "macos"))]
mod macos;
#[cfg(all(feature = "keychain", target_os = "macos"))]
pub use macos::KeychainStore;

#[cfg(all(feature = "keychain", target_os = "windows"))]
mod windows;
#[cfg(all(feature = "keychain", target_os = "windows"))]
pub use windows::KeychainStore;

pub use accounts::Api as Accounts;
pub use session::CallbackProvider;
pub use session::Session;
pub use session::OOB;

// The sandbox url to use as base url for the etrade api
const SANDBOX_URL: &str = "https://apisb.etrade.com";

// The production url to use as base url for the etrade api
const LIVE_URL: &str = "https://api.etrade.com";

fn qs_params<'a, T: serde::Serialize + serde::Deserialize<'a>>(
  params: &T,
) -> Result<Option<BTreeSet<(String, String)>>> {
  let qss = serde_urlencoded::to_string(params)?;
  let qs: BTreeSet<(String, String)> = serde_urlencoded::from_str(&qss)?;
  if qs.is_empty() {
    Ok(None)
  } else {
    Ok(Some(qs))
  }
}

fn empty_body() -> Option<()> {
  None
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString, strum::Display)]
pub enum Mode {
  Sandbox,
  Live,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Messages {
  #[serde(rename = "Message", skip_serializing_if = "Vec::is_empty")]
  pub message: Vec<Message>,
}

impl Messages {
  pub fn is_empty(&self) -> bool {
    self.message.is_empty()
  }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Message {
  pub description: String,
  pub code: i32,
  #[serde(rename = "type")]
  pub tpe: MessageType,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum MessageType {
  #[serde(rename = "WARNING")]
  Warning,
  #[serde(rename = "INFO")]
  Info,
  #[serde(rename = "INFO_HOLD")]
  InfoHold,
  #[serde(rename = "ERROR")]
  Error,
}

impl Default for MessageType {
  fn default() -> Self {
    MessageType::Info
  }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct Product {
  pub symbol: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub security_type: Option<SecurityType>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub security_sub_type: Option<String>,
  pub call_put: String,
  pub expiry_year: i32,
  pub expiry_month: i32,
  pub expiry_day: i32,
  pub strike_price: f64,
  pub expiry_type: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum SecurityType {
  #[serde(rename = "EQ")]
  Eq,
  #[serde(rename = "OPTN")]
  Optn,
  #[serde(rename = "MF")]
  Mf,
  #[serde(rename = "MMF")]
  Mmf,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum MarketSession {
  #[serde(rename = "REGULAR")]
  Regular,
  #[serde(rename = "EXTENDED")]
  Extended,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum OptionType {
  #[serde(rename = "CALL")]
  Call,
  #[serde(rename = "PUT")]
  Put,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum SortOrder {
  #[serde(rename = "ASC")]
  Asc,
  #[serde(rename = "DESC")]
  Desc,
}

#[derive(Debug, Clone)]
pub struct Credentials {
  pub key: SecUtf8,
  pub secret: SecUtf8,
}

impl Credentials {
  pub fn new(key: SecUtf8, secret: SecUtf8) -> Credentials {
    Credentials { key, secret }
  }
}

impl Into<oauth::Credentials> for Credentials {
  fn into(self) -> oauth::Credentials {
    oauth::Credentials::new(self.key.into_unsecure(), self.secret.into_unsecure())
  }
}

impl<T> From<oauth::Credentials<T>> for Credentials
where
  T: Into<SecUtf8>,
{
  fn from(input: oauth::Credentials<T>) -> Self {
    Credentials {
      key: input.identifier.into(),
      secret: input.secret.into(),
    }
  }
}

#[async_trait]
pub trait Store {
  async fn put(
    &self,
    namespace: impl Into<String> + Send,
    key: impl Into<String> + Send,
    value: impl Into<SecUtf8> + Send,
  ) -> Result<()>;
  async fn del(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<()>;
  async fn get(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<Option<SecUtf8>>;
}

#[derive(Debug)]
pub struct Memstore {
  data: Arc<Mutex<HashMap<String, HashMap<String, SecUtf8>>>>,
}

impl Memstore {
  pub fn new() -> Self {
    Memstore {
      data: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

impl Default for Memstore {
  fn default() -> Self {
    Memstore::new()
  }
}

#[async_trait]
impl Store for Memstore {
  async fn put(
    &self,
    namespace: impl Into<String> + Send,
    key: impl Into<String> + Send,
    value: impl Into<SecUtf8> + Send,
  ) -> Result<()> {
    let mut data = self.data.lock().unwrap();

    let svc_state = data.entry(namespace.into()).or_insert_with(HashMap::new);
    svc_state.insert(key.into(), value.into());
    Ok(())
  }

  async fn del(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<()> {
    let mut data = self.data.lock().unwrap();

    if let Some(st) = data.get_mut(namespace.as_ref()) {
      st.remove(key.as_ref());
    }
    Ok(())
  }

  async fn get(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<Option<SecUtf8>> {
    let data = self.data.lock().unwrap();
    Ok(data.get(namespace.as_ref()).and_then(|r| r.get(key.as_ref()).cloned()))
  }
}

#[cfg(test)]
pub mod tests {

  use super::{Memstore, Store};
  use anyhow::Result;
  use secstr::SecUtf8;
  pub(crate) fn init() {
    std::env::set_var("RUST_LOG", "debug");
    let _ = pretty_env_logger::try_init();
  }
  #[tokio::test]
  async fn test_mem_store() {
    verify_token_store(Memstore::new()).await;
  }

  pub async fn verify_token_store(token_store: impl Store) {
    let expected: Result<SecUtf8> = Ok("hello".into());
    token_store.put("my_svc", "api_key", "hello").await.unwrap();
    assert_eq!(token_store.get("my_svc", "api_key").await.ok(), Some(expected.ok()));
    assert!(token_store.del("my_svc", "api_key").await.is_ok());
    assert!(token_store.get("my_svc", "api_key").await.unwrap().is_none());
  }
}

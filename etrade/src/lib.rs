#[macro_use]
extern crate serde;

#[macro_use]
extern crate log;

use async_trait::async_trait;
use oauth1_request::{Credentials, Request};
use reqwest::header::{self, HeaderMap};
use reqwest_oauth1::{
    OAuthClientProvider, Secrets, SecretsProvider, Signer, TokenReader, TokenReaderFuture,
    TokenResponse,
};
use secstr::{SecStr, SecUtf8};
use serde::export::Formatter;
use std::fmt::Display;
use std::future::Future;
use std::intrinsics::write_bytes;
use thiserror::Error;

// The sandbox url to use as base url for the etrade api
const SANDBOX_URL: &str = "https://apisb.etrade.com";

// The production url to use as base url for the etrade api
const LIVE_URL: &str = "https://api.etrade.com";

#[derive(Error, Debug)]
pub enum Error {
    Reqwest(#[from] reqwest::Error),
    Oauth(#[from] reqwest_oauth1::Error),
    Json(#[from] serde_json::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

type Result<T, E = Error> = anyhow::Result<T, E>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RequestToken {
    pub key: SecUtf8,
    pub secret: SecUtf8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessToken {
    pub key: SecUtf8,
    pub secret: SecUtf8,
}

#[async_trait]
trait AuthProvider {
    async fn consumer() -> Result<Credentials<SecUtf8>>;
    async fn request_token() -> Result<Credentials<SecUtf8>>;
    async fn access_token() -> Result<Credentials<SecUtf8>>;
}

pub struct AuthenticatedClient {
    base_url: &'static str,
    consumer: (SecUtf8, SecUtf8),
    request_token: (SecUtf8, SecUtf8),
    token: (SecUtf8, SecUtf8),
    http: reqwest::Client,
}

impl AuthenticatedClient {
    pub fn sandbox(
        consumer_key: SecUtf8,
        consumer_secret: SecUtf8,
        request_token_key: SecUtf8,
        request_token_secret: SecUtf8,
        token_key: SecUtf8,
        token_secret: SecUtf8,
    ) -> AuthenticatedClient {
        Self::new(
            SANDBOX_URL,
            (consumer_key, consumer_secret),
            (request_token_key, request_token_secret),
            (token_key, token_secret),
        )
    }

    pub fn live(
        consumer_key: SecUtf8,
        consumer_secret: SecUtf8,
        request_token_key: SecUtf8,
        request_token_secret: SecUtf8,
        token_key: SecUtf8,
        token_secret: SecUtf8,
    ) -> AuthenticatedClient {
        Self::new(
            LIVE_URL,
            (consumer_key, consumer_secret),
            (request_token_key, request_token_secret),
            (token_key, token_secret),
        )
    }

    fn new(
        base_url: &'static str,
        consumer: (SecUtf8, SecUtf8),
        request_token: (SecUtf8, SecUtf8),
        token: (SecUtf8, SecUtf8),
    ) -> AuthenticatedClient {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        AuthenticatedClient {
            base_url,
            consumer,
            request_token,
            token,
            http: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }

    pub async fn account_list(&self) -> Result<Vec<Account>> {
        let account_list_url = format!("{}/v1/accounts/list", self.base_url);
        debug!("getting account list: {}", &account_list_url);

        let account_json: AccountListResponse = self
            .client()
            .get(&account_list_url)
            .send()
            .await
            .and_then(|result| result.error_for_status().map_err(|e| e.into()))?
            .json()
            .await?;
        Ok(account_json.response.accounts.account)
    }

    pub async fn account_balance(
        &self,
        account: &Account,
        real_time: bool,
    ) -> Result<BalanceResponse> {
        let account_balance_url = format!(
            "{}/v1/accounts/{}/balance",
            self.base_url, &account.account_id_key
        );
        debug!("getting account balance: {}", &account.account_name);

        let balance: serde_json::Value = self
            .client()
            .get(&account_balance_url)
            .query(&[
                ("instType", &account.institution_type),
                ("realTimeNAV", &real_time.to_string()),
            ])
            .send()
            .await
            .and_then(|result| result.error_for_status().map_err(|e| e.into()))?
            .json()
            .await?;

        // debug!("{}", serde_json::to_string_pretty(&balance)?);
        Ok(serde_json::from_value(
            balance.get("BalanceResponse").unwrap().clone(),
        )?)
    }

    pub async fn renew_access_token(&self) -> Result<AccessToken> {
        debug!("renewing access token");
        let secrets = Secrets::new(self.consumer.0.unsecure(), self.consumer.1.unsecure());
        self.http
            .clone()
            .oauth1(secrets.token(
                self.request_token.0.unsecure(),
                self.request_token.1.unsecure(),
            ))
            .get("https://api.etrade.com/oauth/renew_access_token")
            .send()
            .await
            .and_then(|result| result.error_for_status().map_err(|e| e.into()))?
            .parse_oauth_token()
            .await
            .map(|tr| AccessToken {
                key: urldecode::decode(tr.oauth_token).into(),
                secret: urldecode::decode(tr.oauth_token_secret).into(),
            })
            .map_err(|e| e.into())
    }

    pub fn with_access_token(self, token: AccessToken) -> AuthenticatedClient {
        AuthenticatedClient {
            base_url: self.base_url,
            http: self.http,
            consumer: self.consumer,
            request_token: self.request_token,
            token: (token.key, token.secret),
        }
    }

    fn client<'a>(
        &'a self,
    ) -> reqwest_oauth1::Client<Signer<'a, Secrets, oauth1_request::signature_method::HmacSha1>>
    {
        self.http.clone().oauth1(Secrets::new_with_token(
            self.consumer.0.unsecure(),
            self.consumer.1.unsecure(),
            self.token.0.unsecure(),
            self.token.1.unsecure(),
        ))
    }
}

pub struct Client {
    consumer: (SecUtf8, SecUtf8),
    http: reqwest::Client,
}

impl Client {
    pub fn new<T: Into<SecUtf8>>(key: T, secret: T) -> Client {
        let key: SecUtf8 = key.into();
        let secret: SecUtf8 = secret.into();
        debug!(
            "creating client (key: {}, secret: {})",
            key.unsecure(),
            secret.unsecure()
        );
        Client {
            consumer: (key, secret.into()),
            http: reqwest::Client::builder()
                .connection_verbose(true)
                .build()
                .unwrap(),
        }
    }

    pub async fn request_token(&self) -> Result<RequestToken> {
        debug!("getting request token: {:?}", &self.consumer);
        let secrets = Secrets::new(self.consumer.0.unsecure(), self.consumer.1.unsecure());
        self.http
            .clone()
            .oauth1(secrets)
            .get("https://api.etrade.com/oauth/request_token")
            .query(&[("oauth_callback", "oob"), ("format", "json")])
            .send()
            .await
            .and_then(|result| result.error_for_status().map_err(|e| e.into()))?
            .parse_oauth_token()
            .await
            .map(|tr| RequestToken {
                key: urldecode::decode(tr.oauth_token).into(),
                secret: urldecode::decode(tr.oauth_token_secret).into(),
            })
            .map_err(|e| e.into())
    }

    pub async fn verifier_url(&self, key: &RequestToken) -> Result<String> {
        debug!("getting verifier url");
        Ok(format!(
            "https://us.etrade.com/e/t/etws/authorize?key={}&token={}",
            &self.consumer.0.unsecure(),
            &key.key.unsecure(),
        ))
    }

    pub async fn access_token(&self, token: RequestToken, pin: &str) -> Result<AccessToken> {
        debug!("getting access token");
        let secrets = Secrets::new(self.consumer.0.unsecure(), self.consumer.1.unsecure());
        self.http
            .clone()
            .oauth1(secrets.token(token.key.unsecure(), token.secret.unsecure()))
            .get("https://api.etrade.com/oauth/access_token")
            .query(&[("oauth_verifier", pin)])
            .send()
            .await
            .and_then(|result| result.error_for_status().map_err(|e| e.into()))?
            .parse_oauth_token()
            .await
            .map(|tr| AccessToken {
                key: urldecode::decode(tr.oauth_token).into(),
                secret: urldecode::decode(tr.oauth_token_secret).into(),
            })
            .map_err(|e| e.into())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct AccountListResponse {
    #[serde(rename = "AccountListResponse")]
    response: AccountList,
}

#[derive(Debug, Deserialize, Serialize)]
struct AccountList {
    #[serde(rename = "Accounts")]
    accounts: AccountHolder,
}

#[derive(Debug, Deserialize, Serialize)]
struct AccountHolder {
    #[serde(rename = "Account")]
    account: Vec<Account>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub inst_no: Option<i32>,
    pub account_id: String,
    pub account_id_key: String,
    pub account_mode: String,
    pub account_desc: String,
    pub account_name: String,
    pub account_type: String,
    pub institution_type: String,
    pub account_status: String,
    pub closed_date: i64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    pub account_id: String,
    pub institution_type: Option<String>,
    pub as_of_date: Option<i64>,
    pub account_type: String,
    pub option_level: String,
    pub account_description: String,
    pub quote_mode: Option<i32>,
    pub day_trader_status: Option<String>,
    pub account_mode: Option<String>,
    pub account_desc: Option<String>,
    #[serde(default)]
    pub open_calls: Vec<OpenCalls>,
    #[serde(rename = "Cash")]
    pub cash: Option<Cash>,
    #[serde(rename = "Margin")]
    pub margin: Option<Margin>,
    pub lending: Option<Lending>,
    #[serde(rename = "Computed")]
    pub computed_balance: ComputedBalance,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenCalls {
    pub min_equity_call: Option<f64>,
    pub fed_call: Option<f64>,
    pub cash_call: f64,
    pub house_call: Option<f64>,
}
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Cash {
    pub funds_for_open_orders_cash: f64,
    pub money_mkt_balance: f64,
}
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Margin {
    pub dt_cash_open_order_reserve: f64,
    pub dt_margin_open_order_reserve: f64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Lending {
    pub current_balance: f64,
    pub credit_line: f64,
    pub outstanding_balance: f64,
    pub min_payment_due: f64,
    pub amount_past_due: f64,
    pub available_credit: f64,
    pub ytd_interest_paid: f64,
    pub last_ytd_interest_paid: f64,
    pub payment_due_date: i64,
    pub last_payment_received_date: i64,
    pub payment_received_mtd: f64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComputedBalance {
    pub cash_available_for_investment: f64,
    pub cash_available_for_withdrawal: f64,
    pub total_available_for_withdrawal: Option<f64>,
    pub net_cash: f64,
    pub cash_balance: f64,
    pub settled_cash_for_investment: f64,
    pub un_settled_cash_for_investment: f64,
    pub funds_withheld_from_purchase_power: f64,
    pub funds_withheld_from_withdrawal: f64,
    pub margin_buying_power: Option<f64>,
    pub cash_buying_power: Option<f64>,
    pub dt_margin_buying_power: Option<f64>,
    pub dt_cash_buying_power: Option<f64>,
    pub margin_balance: Option<f64>,
    pub short_adjust_balance: Option<f64>,
    pub regt_equity: Option<f64>,
    pub regt_equity_percent: Option<f64>,
    pub account_balance: Option<f64>,
    #[serde(rename = "OpenCalls")]
    pub open_calls: OpenCalls,
    #[serde(rename = "RealTimeValues")]
    pub real_time_values: RealTimeValues,
    #[serde(rename = "PortfolioMargin")]
    pub portfolio_margin: Option<PortfolioMargin>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioMargin {
    pub dt_cash_open_order_reserve: f64,
    pub dt_margin_open_order_reserve: f64,
    pub liquidating_equity: f64,
    pub house_excess_equity: f64,
    pub total_house_requirement: f64,
    pub excess_equity_minus_requirement: f64,
    pub total_margin_rqmts: f64,
    pub avail_excess_equity: f64,
    pub excess_equity: f64,
    pub open_order_reserve: f64,
    pub funds_on_hold: f64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RealTimeValues {
    pub total_account_value: f64,
    pub net_mv: f64,
    pub net_mv_long: f64,
    pub net_mv_short: Option<f64>,
    pub total_long_value: Option<f64>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

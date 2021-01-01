use crate::{empty_body, qs_params, session::CallbackProvider, Session, SortOrder, Store};
use anyhow::Result;
use http::Method;
use std::sync::Arc;
use strum::EnumString;

pub struct Api<T: Store> {
  session: Arc<Session<T>>,
}

impl<T> Api<T>
where
  T: Store,
{
  pub fn new(session: Arc<Session<T>>) -> Self {
    Self { session }
  }

  pub async fn list(&self, params: ListAlertsRequest, callbacks: impl CallbackProvider) -> Result<AlertsResponse> {
    let alerts: serde_json::Value = self
      .session
      .send(Method::GET, "/v1/users/alerts", qs_params(&params)?, callbacks)
      .await?;
    debug!("alerts json: {}", serde_json::to_string_pretty(&alerts)?);
    Ok(serde_json::from_value(alerts.get("AlertsResponse").unwrap().clone())?)
  }

  pub async fn details(
    &self,
    alert_id: &str,
    html: bool,
    callbacks: impl CallbackProvider,
  ) -> Result<AlertDetailsResponse> {
    let alerts: serde_json::Value = self
      .session
      .send(
        Method::GET,
        format!("/v1/users/alerts/{}", alert_id),
        if html { Some(vec![("htmlTags", true)]) } else { None },
        callbacks,
      )
      .await?;
    debug!("alert json: {}", serde_json::to_string_pretty(&alerts)?);
    Ok(serde_json::from_value(
      alerts.get("AlertDetailsResponse").unwrap().clone(),
    )?)
  }

  pub async fn delete(&self, alert_id: &str, callbacks: impl CallbackProvider) -> Result<DeleteAlertsResponse> {
    let alerts: serde_json::Value = self
      .session
      .send(
        Method::DELETE,
        format!("/v1/users/alerts/{}", alert_id),
        empty_body(),
        callbacks,
      )
      .await?;
    debug!("alert json: {}", serde_json::to_string_pretty(&alerts)?);
    Ok(serde_json::from_value(alerts.get("AlertsResponse").unwrap().clone())?)
  }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ListAlertsRequest {
  pub count: Option<usize>,
  pub category: Option<Category>,
  pub status: Option<Status>,
  pub direction: Option<SortOrder>,
  pub search: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AlertsResponse {
  pub total_alerts: i64,
  pub alerts: Vec<Alert>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AlertDetailsResponse {
  pub id: i64,
  pub create_time: i64,
  pub subject: String,
  pub msg_text: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub read_time: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub delete_time: Option<i64>,
  pub symbol: Option<String>,
  pub next: String,
  pub prev: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct DeleteAlertsResponse {
  pub result: String,
  pub failed_alerts: FailedAlerts,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct FailedAlerts {
  pub alert_id: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Alert {
  pub id: i64,
  pub create_time: i64,
  pub subject: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<Status>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Category {
  #[serde(rename = "STOCK")]
  Stock,
  #[serde(rename = "ACCOUNT")]
  Account,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
  #[serde(rename = "READ")]
  Read,
  #[serde(rename = "UNREAD")]
  Unread,
  #[serde(rename = "DELETED")]
  Deleted,
}

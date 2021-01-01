use anyhow::Result;
use http::Method;
use std::sync::Arc;

use crate::{qs_params, session::CallbackProvider, Product, Session, SortOrder, Store};

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

  pub async fn list<'a>(
    &self,
    account_id_key: &'a str,
    params: ListTransactionsRequest<'a>,
    callbacks: impl CallbackProvider,
  ) -> Result<TransactionListResponse> {
    let orders: serde_json::Value = self
      .session
      .send(
        Method::GET,
        format!("/v1/accounts/{}/transactions", account_id_key),
        qs_params(&params)?,
        callbacks,
      )
      .await?;
    debug!("orders json: {}", serde_json::to_string_pretty(&orders)?);
    Ok(serde_json::from_value(
      orders.get("TransactionListResponse").unwrap().clone(),
    )?)
  }

  pub async fn details<'a>(
    &self,
    account_id_key: &'a str,
    tranid: &'a str,
    store_id: &'a str,
    callbacks: impl CallbackProvider,
  ) -> Result<TransactionDetailsResponse> {
    let orders: serde_json::Value = self
      .session
      .send(
        Method::GET,
        format!("/v1/accounts/{}/transactions/{}", account_id_key, tranid),
        if store_id.is_empty() {
          None
        } else {
          Some(vec![("storeId", store_id)])
        },
        callbacks,
      )
      .await?;
    debug!("orders json: {}", serde_json::to_string_pretty(&orders)?);
    Ok(serde_json::from_value(
      orders.get("TransactionDetailsResponse").unwrap().clone(),
    )?)
  }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ListTransactionsRequest<'a> {
  pub start_date: Option<&'a str>,
  pub end_date: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sort_order: Option<SortOrder>,
  pub marker: Option<&'a str>,
  pub count: Option<usize>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TransactionListResponse {
  pub page_marker: String,
  pub more_transactions: bool,
  pub transaction_count: usize,
  pub total_count: usize,
  #[serde(rename = "Transaction", skip_serializing_if = "Vec::is_empty")]
  pub transaction: Vec<TransactionDetailsResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TransactionDetailsResponse {
  pub transaction_id: i64,
  pub account_id: String,
  pub tranaction_date: i64,
  pub postdate: i64,
  pub amount: f64,
  #[serde(rename = "Category", skip_serializing_if = "Option::is_none")]
  pub category: Option<Category>,
  #[serde(rename = "Brokerage", skip_serializing_if = "Option::is_none")]
  pub brokerage: Option<Brokerage>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Category {
  pub category_id: String,
  pub parent_id: String,
  pub category_name: String,
  pub parent_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Brokerage {
  pub transaction_type: String,
  pub product: Product,
  pub quantity: f64,
  pub price: f64,
  pub settlement_currency: String,
  pub payment_currency: String,
  pub fee: f64,
  pub memo: String,
  pub check_no: String,
  pub order_no: String,
}

pub enum ListFormat {
  Xls,
  Xlx,
  Json,
  Xml,
}

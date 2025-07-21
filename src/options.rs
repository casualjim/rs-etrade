use crate::{accounts::QuoteStatus, empty_body, qs_params, session::CallbackProvider, Messages};
use crate::{Product, Session, Store};
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

  pub async fn quotes(
    &self,
    symbols: &str,
    params: GetQuotesRequest,
    callbacks: impl CallbackProvider,
  ) -> Result<QuoteResponse> {
    let quotes: serde_json::Value = self
      .session
      .send(
        Method::GET,
        format!("/v1/market/quote/{}", symbols),
        qs_params(&params)?,
        callbacks,
      )
      .await?;
    debug!("quotes json: {}", serde_json::to_string_pretty(&quotes)?);
    Ok(serde_json::from_value(quotes.get("QuoteResponse").unwrap().clone())?)
  }

  pub async fn product(&self, search: &str, callbacks: impl CallbackProvider) -> Result<LookupResponse> {
    let product: serde_json::Value = self
      .session
      .send(
        Method::GET,
        format!("/v1/market/quote/{}", search),
        empty_body(),
        callbacks,
      )
      .await?;
    debug!("product json: {}", serde_json::to_string_pretty(&product)?);
    Ok(serde_json::from_value(product.get("LookupResponse").unwrap().clone())?)
  }

  pub async fn chains<'a>(
    &self,
    params: &'a GetOptionChainsRequest<'a>,
    callbacks: impl CallbackProvider,
  ) -> Result<OptionChainResponse> {
    let chains: serde_json::Value = self
      .session
      .send(
        Method::GET,
        "/v1/market/optionchains",
        qs_params(params)?,
        callbacks,
      )
      .await?;
    debug!("chains json: {}", serde_json::to_string_pretty(&chains)?);
    
    // Check if we got an OptionChainResponse or if the API returned a QuoteResponse instead
    if let Some(option_chain_response) = chains.get("OptionChainResponse") {
      Ok(serde_json::from_value(option_chain_response.clone())?)
    } else {
      // If we got a QuoteResponse instead, this might indicate the endpoint is wrong
      // or the parameters are not being passed correctly
      Err(anyhow::anyhow!(
        "Expected OptionChainResponse but got: {}",
        serde_json::to_string_pretty(&chains)?
      ))
    }
  }

  pub async fn expire_dates<'a>(
    &self,
    params: &'a GetOptionExpireDatesRequest<'a>,
    callbacks: impl CallbackProvider,
  ) -> Result<OptionExpireDateResponse> {
    let dates: serde_json::Value = self
      .session
      .send(
        Method::GET,
        "/v1/market/quote/optionexpiredate",
        qs_params(params)?,
        callbacks,
      )
      .await?;
    debug!("dates json: {}", serde_json::to_string_pretty(&dates)?);
    
    // Check if we got an OptionExpireDateResponse or if the API returned something else
    if let Some(option_expire_date_response) = dates.get("OptionExpireDateResponse") {
      Ok(serde_json::from_value(option_expire_date_response.clone())?)
    } else {
      // If we got something else, this might indicate the endpoint is wrong
      // or the parameters are not being passed correctly
      Err(anyhow::anyhow!(
        "Expected OptionExpireDateResponse but got: {}",
        serde_json::to_string_pretty(&dates)?
      ))
    }
  }
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GetOptionExpireDatesRequest<'a> {
  pub expiry_type: Option<ExpiryType>,
  pub symbol: &'a str,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionExpireDateResponse {
  #[serde(rename = "ExpirationDate", skip_serializing_if = "Vec::is_empty")]
  pub expiration_dates: Vec<ExpirationDate>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ExpirationDate {
  pub year: i32,
  pub month: i32,
  pub day: i32,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub expiry_type: Option<ExpiryType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GetOptionChainsRequest<'a> {
  pub symbol: &'a str,
  pub expiry_year: Option<usize>,
  pub expiry_month: Option<usize>,
  pub expiry_day: Option<usize>,
  pub strike_price_near: Option<f64>,
  pub no_of_strikes: Option<u16>,
  pub include_weekly: Option<bool>,
  pub skip_adjusted: Option<bool>,
  pub option_category: Option<OptionCategory>,
  pub chain_type: Option<ChainType>,
  pub price_type: Option<PriceType>,
}

impl<'a> Default for GetOptionChainsRequest<'a> {
  fn default() -> Self {
    Self {
      symbol: "",
      expiry_year: None,
      expiry_month: None,
      expiry_day: None,
      strike_price_near: None,
      no_of_strikes: None,
      include_weekly: None,
      skip_adjusted: None,
      option_category: None,
      chain_type: None,
      price_type: None,
    }
  }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionChainResponse {
  #[serde(rename = "OptionPair", skip_serializing_if = "Vec::is_empty")]
  pub option_pairs: Vec<OptionChainPair>,
  pub time_stamp: i64,
  pub quote_type: String,
  pub near_price: f64,
  #[serde(rename = "SelectedED", skip_serializing_if = "Option::is_none")]
  pub selected: Option<SelectedED>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionChainPair {
  #[serde(rename = "Call", skip_serializing_if = "Option::is_none")]
  pub call: Option<OptionDetails>,
  #[serde(rename = "Put", skip_serializing_if = "Option::is_none")]
  pub put: Option<OptionDetails>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pair_type: Option<PairType>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionDetails {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub option_category: Option<OptionCategory>,
  pub option_root_symbol: String,
  pub time_stamp: i64,
  pub adjusted_flag: bool,
  pub display_symbol: String,
  pub option_type: String,
  pub strike_price: f64,
  pub symbol: String,
  pub bid: f64,
  pub ask: f64,
  pub bid_size: i64,
  pub ask_size: i64,
  pub in_the_money: String,
  pub volume: i64,
  pub open_interest: i64,
  pub net_change: f64,
  pub last_price: f64,
  pub quote_detail: String,
  pub osi_key: String,
  #[serde(rename = "OptionGreeks", skip_serializing_if = "Option::is_none")]
  pub option_greeks: Option<OptionGreeks>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SelectedED {
  pub month: i32,
  pub year: i32,
  pub day: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LookupResponse {
  #[serde(rename = "Data", skip_serializing_if = "Vec::is_empty")]
  pub data: Vec<Data>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Data {
  pub symbol: String,
  pub description: String,
  #[serde(rename = "type")]
  pub symbol_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GetQuotesRequest {
  pub detail_flag: Option<DetailFlag>,
  pub require_earnings_date: Option<bool>,
  pub override_symbol_count: Option<bool>,
  pub skip_mini_options_check: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct QuoteResponse {
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub quote_data: Vec<QuoteData>,
  #[serde(skip_serializing_if = "Messages::is_empty")]
  pub message_list: Messages,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct QuoteData {
  #[serde(rename = "All", skip_serializing_if = "Option::is_none")]
  pub all: Option<AllQuoteDetails>,
  pub date_time: String,
  #[serde(rename = "dateTimeUTC")]
  pub date_time_utc: i64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quote_status: Option<QuoteStatus>,
  pub ah_flags: String,
  pub error_message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fundamental: Option<FundamentalQuoteDetails>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub intraday: Option<IntraQuoteDetails>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub option: Option<OptionQuoteDetails>,
  #[serde(rename = "Product", skip_serializing_if = "Option::is_none")]
  pub product: Option<Product>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub week52: Option<Week52QuoteDetails>,
  #[serde(rename = "MutualFund", skip_serializing_if = "Option::is_none")]
  pub mutual_fund: Option<MutualFund>,
  pub time_zone: String,
  pub dst_flag: bool,
  pub has_mini_options: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct MutualFund {
  pub symbol_description: String,
  pub cusip: String,
  pub change_close: f64,
  pub previous_close: f64,
  pub transaction_fee: f64,
  pub early_redemption_fee: String,
  pub availability: String,
  pub initial_investment: f64,
  pub subsequent_investment: f64,
  pub fund_family: String,
  pub fund_name: String,
  pub change_close_percentage: f64,
  pub time_of_last_trade: i64,
  pub net_asset_value: f64,
  pub public_offer_price: f64,
  pub net_expense_ratio: f64,
  pub gross_expense_ratio: f64,
  pub order_cutoff_time: i64,
  pub sales_charge: String,
  pub initial_ira_investment: f64,
  pub subsequent_ira_investment: f64,
  pub net_assets: NetAsset,
  pub fund_inception_date: i64,
  pub average_annual_returns: f64,
  pub seven_day_current_yield: f64,
  pub annual_total_return: f64,
  pub weighted_average_maturity: f64,
  pub average_annual_returns_1_yr: f64,
  pub average_annual_returns_3_yr: f64,
  pub average_annual_returns_5_yr: f64,
  pub average_annual_returns_10_yr: f64,
  pub high52: f64,
  pub low52: f64,
  pub week_52_low_date: i64,
  pub week_52_hi_date: i64,
  pub exchange_name: String,
  pub since_inception: f64,
  pub quarterly_since_inception: f64,
  pub last_trade: f64,
  #[serde(rename = "actual12B1Fee")]
  pub actual_12b1_fee: f64,
  pub performance_as_of_date: String,
  pub qtrly_performance_as_of_date: String,
  pub redemption: Redemption,
  pub morning_star_category: String,
  #[serde(rename = "monthlyTrailingReturn1Y")]
  pub monthly_trailing_return_1y: f64,
  #[serde(rename = "monthlyTrailingReturn3Y")]
  pub monthly_trailing_return_3y: f64,
  #[serde(rename = "monthlyTrailingReturn5Y")]
  pub monthly_trailing_return_5y: f64,
  #[serde(rename = "monthlyTrailingReturn10Y")]
  pub monthly_trailing_return_10y: f64,
  pub etrade_early_redemption_fee: String,
  pub max_sales_load: f64,
  #[serde(rename = "monthlyTrailingReturnYTD")]
  pub monthly_trailing_return_ytd: f64,
  #[serde(rename = "monthlyTrailingReturn1M")]
  pub monthly_trailing_return_1m: f64,
  #[serde(rename = "monthlyTrailingReturn3M")]
  pub monthly_trailing_return_3m: f64,
  #[serde(rename = "monthlyTrailingReturn6M")]
  pub monthly_trailing_return_6m: f64,
  #[serde(rename = "qtrlyTrailingReturnYTD")]
  pub qtrly_trailing_return_ytd: f64,
  #[serde(rename = "qtrlyTrailingReturn1M")]
  pub qtrly_trailing_return_1m: f64,
  #[serde(rename = "qtrlyTrailingReturn3M")]
  pub qtrly_trailing_return_3m: f64,
  #[serde(rename = "qtrlyTrailingReturn6M")]
  pub qtrly_trailing_return_6m: f64,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub deferred_sales_changes: Vec<SaleChargeValues>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub frontend_sales_changes: Vec<SaleChargeValues>,
  pub exchange_code: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Redemption {
  pub min_month: String,
  pub fee_percent: String,
  pub is_front_end: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub front_end_values: Vec<Values>,
  pub redemption_duration_type: String,
  pub is_sales: String,
  pub sales_duration_type: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub sales_values: Vec<Values>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Values {
  pub low: String,
  pub high: String,
  pub percent: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SaleChargeValues {
  pub lowhigh: String,
  pub percent: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NetAsset {
  pub value: f64,
  pub as_of_date: i64,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Week52QuoteDetails {
  pub annual_dividend: f64,
  pub company_name: String,
  pub high52: f64,
  pub last_trade: f64,
  pub low52: f64,
  pub perf_12_months: f64,
  pub previous_close: f64,
  pub symbol_description: String,
  pub total_volume: i64,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionQuoteDetails {
  pub ask: f64,
  pub ask_size: i64,
  pub bid: f64,
  pub bid_size: i64,
  pub company_name: String,
  pub days_to_expiration: i64,
  pub last_trade: f64,
  pub open_interest: i64,
  pub option_previous_bid_price: f64,
  pub option_previous_ask_price: f64,
  pub osi_key: String,
  pub intrinsic_value: f64,
  pub time_premium: f64,
  pub option_multiplier: f64,
  pub contract_size: f64,
  pub symbol_description: String,
  #[serde(rename = "OptionGreeks", skip_serializing_if = "Option::is_none")]
  pub option_greeks: Option<OptionGreeks>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionGreeks {
  pub rho: f64,
  pub vega: f64,
  pub theta: f64,
  pub delta: f64,
  pub gamma: f64,
  pub iv: f64,
  pub current_value: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct IntraQuoteDetails {
  pub ask: f64,
  pub bid: f64,
  pub change_close: f64,
  pub change_close_percentage: f64,
  pub company_name: String,
  pub high: f64,
  pub last_trade: f64,
  pub low: f64,
  pub total_volume: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct FundamentalQuoteDetails {
  pub company_name: String,
  pub eps: f64,
  pub est_earnings: f64,
  pub high52: f64,
  pub last_trade: f64,
  pub low52: f64,
  pub symbol_description: String,
  pub volume_10_day: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AllQuoteDetails {
  pub adjusted_flag: bool,
  pub annual_dividend: Option<f64>,
  pub ask: f64,
  pub ask_exchange: Option<String>,
  pub ask_size: i64,
  pub ask_time: String,
  pub average_volume: i64,
  pub beta: f64,
  pub bid: f64,
  pub bid_exchange: Option<String>,
  pub bid_size: i64,
  pub bid_time: String,
  pub cash_deliverable: f64,
  pub change_close: f64,
  pub change_close_percentage: f64,
  pub company_name: String,
  pub contract_size: f64,
  pub days_to_expiration: i64,
  pub declared_dividend: f64,
  pub dir_last: String,
  pub dividend: f64,
  pub dividend_payable_date: i64,
  pub eps: f64,
  pub est_earnings: f64,
  pub ex_dividend_date: i64,
  pub expiration_date: i64,
  pub exchg_last_trade: Option<String>,
  pub fsi: Option<String>,
  pub high: f64,
  pub high52: f64,
  pub intrinsic_value: f64,
  pub last_trade: f64,
  pub low: f64,
  pub low52: f64,
  pub market_cap: f64,
  pub next_earning_date: String,
  pub open: f64,
  pub open_interest: i64,
  pub open_multiplier: f64,
  pub option_style: Option<String>,
  pub pe: f64,
  pub previous_close: f64,
  pub previous_day_volume: i64,
  pub primary_exchange: String,
  pub shares_outstanding: f64,
  pub symbol_description: String,
  pub time_of_last_trade: i64,
  pub time_premium: f64,
  pub total_volume: i64,
  pub upc: i64,
  pub week_52_hi_date: i64,
  pub week_52_low_date: i64,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionDeliverable {
  pub root_symbol: String,
  pub deliverable_symbol: String,
  pub deliverable_type_code: String,
  pub deliverable_exchange_code: String,
  pub deliverable_strike_percent: f64,
  pub deliverable_c_i_l_shares: f64,
  pub deliverable_whole_shares: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ExtendedHourQuoteDetail {
  pub last_price: f64,
  pub change: f64,
  pub percent_change: f64,
  pub bid: f64,
  pub bid_size: i64,
  pub ask: f64,
  pub ask_size: i64,
  pub volume: i64,
  pub time_of_last_trade: i64,
  pub time_zone: String,
  pub quote_status: String,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum DetailFlag {
  #[serde(rename = "ALL")]
  All,
  #[serde(rename = "FUNDAMENTAL")]
  Fundamental,
  #[serde(rename = "INTRADAY")]
  Intraday,
  #[serde(rename = "OPTIONS")]
  Options,
  #[serde(rename = "WEEK_52")]
  Week52,
  #[serde(rename = "MF_DETAIL")]
  MfDetail,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum OptionCategory {
  #[serde(rename = "STANDARD")]
  Standard,
  #[serde(rename = "ALL")]
  All,
  #[serde(rename = "MINI")]
  Mini,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ChainType {
  #[serde(rename = "CALL")]
  Call,
  #[serde(rename = "PUT")]
  Put,
  #[serde(rename = "CALLPUT")]
  CallPut,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum PairType {
  #[serde(rename = "CALLONLY")]
  CallOnly,
  #[serde(rename = "PUTONLY")]
  PutOnly,
  #[serde(rename = "CALLPUT")]
  CallPut,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum PriceType {
  #[serde(rename = "ATNM")]
  Atnm,
  #[serde(rename = "ALL")]
  All,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ExpiryType {
  #[serde(rename = "UNSPECIFIED")]
  Unspecified,
  #[serde(rename = "DAILY")]
  Daily,
  #[serde(rename = "WEEKLY")]
  Weekly,
  #[serde(rename = "MONTHLY")]
  Monthly,
  #[serde(rename = "QUARTERLY")]
  Quarterly,
  #[serde(rename = "VIX")]
  Vix,
  #[serde(rename = "ALL")]
  All,
  #[serde(rename = "MONTHEND")]
  Monthend,
}

impl Default for ExpiryType {
  fn default() -> Self {
    ExpiryType::Unspecified
  }
}

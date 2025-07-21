use crate::{Session, Store, qs_params, Messages, CallbackProvider};
use anyhow::{anyhow, Result};
use http::Method;
use serde::{Deserialize, Serialize, Deserializer};
use std::sync::Arc;

// Custom deserializer for ah_flag that can handle both string and bool
fn deserialize_ah_flag<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    let value = serde_json::Value::deserialize(deserializer)?;
    
    match value {
        serde_json::Value::Bool(b) => Ok(Some(b)),
        serde_json::Value::String(s) => {
            match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => Ok(Some(true)),
                "false" | "0" | "no" => Ok(Some(false)),
                _ => Err(Error::custom(format!("Invalid boolean string: {}", s))),
            }
        }
        serde_json::Value::Null => Ok(None),
        _ => Err(Error::custom("Expected boolean or string")),
    }
}

pub struct Api<T: Store> {
    session: Arc<Session<T>>,
}

impl<T: Store> Api<T> {
    pub fn new(session: Arc<Session<T>>) -> Self {
        Self { session }
    }

    /// Fetches quote information for one or more symbols.
    pub async fn quote(
        &self,
        symbols: &[&str],
        params: Option<GetQuotesRequest>,
        callbacks: impl CallbackProvider,
    ) -> Result<QuoteResponse> {
        if symbols.len() > 25 {
            return Err(anyhow!("Maximum of 25 symbols allowed"));
        }
        let val: serde_json::Value = self.session
            .send(
                Method::GET,
                format!("/v1/market/quote/{}", symbols.join(",")),
                qs_params(&params.unwrap_or_default())?,
                callbacks,
            )
            .await?;
        debug!("quote: {}", val.to_string());
        Ok(serde_json::from_value(val.get("QuoteResponse").unwrap().clone())?)
    }

    /// Fetches option expiration dates for a given symbol.
    pub async fn option_expire_dates(
        &self,
        params: Option<GetOptionExpireDatesRequest>,
        callbacks: impl CallbackProvider,
    ) -> Result<OptionExpireDateResponse> {
        let val: serde_json::Value = self.session
            .send(
                Method::GET,
                format!("/v1/market/optionexpiredate"),
                qs_params(&params)?,
                callbacks,
            )
            .await?;
        Ok(serde_json::from_value(val.get("OptionExpireDateResponse").unwrap().clone())?)
    }
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
pub struct GetOptionExpireDatesRequest {
    pub symbol: String,
    pub expiry_type: Option<ExpiryType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ExpiryType {
    Unspecified,
    All,
    Monthly,
    Weekly,
    Daily,
    Quarterly,
    Vix,
    MonthEnd,
}

impl Default for ExpiryType {
    fn default() -> Self {
        ExpiryType::All
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DetailFlag {
    ALL,
    FUNDAMENTAL,
    INTRADAY,
    OPTIONS,
    WEEK_52,
    MF_DETAIL,
}

impl Default for DetailFlag {
    fn default() -> Self {
        DetailFlag::ALL
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct QuoteResponse {
    #[serde(rename = "QuoteData", skip_serializing_if = "Vec::is_empty")]
    pub quote_data: Vec<QuoteData>,
    #[serde(skip_serializing_if = "Messages::is_empty")]
    pub messages: Messages,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionExpireDateResponse {
    #[serde(rename = "ExpirationDate", skip_serializing_if = "Vec::is_empty")]
    pub expiration_dates: Vec<ExpirationDate>,
    #[serde(skip_serializing_if = "Messages::is_empty")]
    pub messages: Messages,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ExpirationDate {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub expiry_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct QuoteData {
    #[serde(rename = "All", skip_serializing_if = "Option::is_none")]
    pub all: Option<AllQuoteDetails>,
    pub date_time: Option<String>,
    #[serde(rename = "dateTimeUTC")]
    pub date_time_utc: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_status: Option<String>,
    // This needs to parse a string
    #[serde(deserialize_with = "deserialize_ah_flag")]
    pub ah_flag: Option<bool>,
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fundamental: Option<FundamentalQuoteDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intraday: Option<IntraQuoteDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<OptionQuoteDetails>,
    #[serde(rename = "Product", skip_serializing_if = "Option::is_none")]
    pub product: Option<crate::Product>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub week52: Option<Week52QuoteDetails>,
    #[serde(rename = "MutualFund", skip_serializing_if = "Option::is_none")]
    pub mutual_fund: Option<MutualFund>,
    pub time_zone: Option<String>,
    pub dst_flag: Option<bool>,
    pub has_mini_options: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AllQuoteDetails {
    pub ask: Option<f64>,
    pub bid: Option<f64>,
    pub last_trade: Option<f64>,
    pub company_name: Option<String>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub open: Option<f64>,
    pub previous_close: Option<f64>,
    pub total_volume: Option<i64>,
    pub change_close: Option<f64>,
    pub change_close_percentage: Option<f64>,
    pub days_to_expiration: Option<i64>,
    pub open_interest: Option<i64>,
    pub option_style: Option<String>,
    pub option_underlier: Option<String>,
    pub intrinsic_value: Option<f64>,
    pub time_premium: Option<f64>,
    pub option_multiplier: Option<f64>,
    pub contract_size: Option<f64>,
    pub expiration_date: Option<i64>,
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub theta: Option<f64>,
    pub vega: Option<f64>,
    pub implied_volatility: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct FundamentalQuoteDetails {
    pub company_name: Option<String>,
    pub eps: Option<f64>,
    pub est_earnings: Option<f64>,
    pub high52: Option<f64>,
    pub last_trade: Option<f64>,
    pub low52: Option<f64>,
    pub symbol_description: Option<String>,
    pub volume_10_day: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct IntraQuoteDetails {
    pub ask: Option<f64>,
    pub bid: Option<f64>,
    pub change_close: Option<f64>,
    pub change_close_percentage: Option<f64>,
    pub company_name: Option<String>,
    pub high: Option<f64>,
    pub last_trade: Option<f64>,
    pub low: Option<f64>,
    pub total_volume: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionQuoteDetails {
    pub ask: Option<f64>,
    pub ask_size: Option<i64>,
    pub bid: Option<f64>,
    pub bid_size: Option<i64>,
    pub company_name: Option<String>,
    pub days_to_expiration: Option<i64>,
    pub last_trade: Option<f64>,
    pub open_interest: Option<i64>,
    pub option_previous_bid_price: Option<f64>,
    pub option_previous_ask_price: Option<f64>,
    pub osi_key: Option<String>,
    pub intrinsic_value: Option<f64>,
    pub time_premium: Option<f64>,
    pub option_multiplier: Option<f64>,
    pub contract_size: Option<f64>,
    pub symbol_description: Option<String>,
    #[serde(rename = "OptionGreeks", skip_serializing_if = "Option::is_none")]
    pub option_greeks: Option<OptionGreeks>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionGreeks {
    pub rho: Option<f64>,
    pub vega: Option<f64>,
    pub theta: Option<f64>,
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub iv: Option<f64>,
    pub current_value: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Week52QuoteDetails {
    pub annual_dividend: Option<f64>,
    pub company_name: Option<String>,
    pub high52: Option<f64>,
    pub last_trade: Option<f64>,
    pub low52: Option<f64>,
    pub perf_12_months: Option<f64>,
    pub previous_close: Option<f64>,
    pub symbol_description: Option<String>,
    pub total_volume: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct MutualFund {
    pub symbol_description: Option<String>,
    pub cusip: Option<String>,
    pub change_close: Option<f64>,
    pub previous_close: Option<f64>,
    pub transaction_fee: Option<f64>,
    pub early_redemption_fee: Option<String>,
    pub availability: Option<String>,
    pub initial_investment: Option<f64>,
    pub subsequent_investment: Option<f64>,
    pub fund_family: Option<String>,
    pub fund_name: Option<String>,
    pub change_close_percentage: Option<f64>,
    pub time_of_last_trade: Option<i64>,
    pub net_asset_value: Option<f64>,
    pub public_offer_price: Option<f64>,
    pub net_expense_ratio: Option<f64>,
    pub gross_expense_ratio: Option<f64>,
    pub order_cutoff_time: Option<i64>,
    pub sales_charge: Option<String>,
    pub initial_ira_investment: Option<f64>,
    pub subsequent_ira_investment: Option<f64>,
    pub net_assets: Option<NetAsset>,
    pub fund_inception_date: Option<i64>,
    pub average_annual_returns: Option<f64>,
    pub seven_day_current_yield: Option<f64>,
    pub annual_total_return: Option<f64>,
    pub weighted_average_maturity: Option<f64>,
    pub average_annual_returns_1_yr: Option<f64>,
    pub average_annual_returns_3_yr: Option<f64>,
    pub average_annual_returns_5_yr: Option<f64>,
    pub average_annual_returns_10_yr: Option<f64>,
    pub high52: Option<f64>,
    pub low52: Option<f64>,
    pub week_52_low_date: Option<i64>,
    pub week_52_hi_date: Option<i64>,
    pub exchange_name: Option<String>,
    pub since_inception: Option<f64>,
    pub quarterly_since_inception: Option<f64>,
    pub last_trade: Option<f64>,
    #[serde(rename = "actual12B1Fee")]
    pub actual_12b1_fee: Option<f64>,
    pub performance_as_of_date: Option<String>,
    pub qtrly_performance_as_of_date: Option<String>,
    pub redemption: Option<Redemption>,
    pub morning_star_category: Option<String>,
    #[serde(rename = "monthlyTrailingReturn1Y")]
    pub monthly_trailing_return_1y: Option<f64>,
    #[serde(rename = "monthlyTrailingReturn3Y")]
    pub monthly_trailing_return_3y: Option<f64>,
    #[serde(rename = "monthlyTrailingReturn5Y")]
    pub monthly_trailing_return_5y: Option<f64>,
    #[serde(rename = "monthlyTrailingReturn10Y")]
    pub monthly_trailing_return_10y: Option<f64>,
    pub etrade_early_redemption_fee: Option<String>,
    pub max_sales_load: Option<f64>,
    #[serde(rename = "monthlyTrailingReturnYTD")]
    pub monthly_trailing_return_ytd: Option<f64>,
    #[serde(rename = "monthlyTrailingReturn1M")]
    pub monthly_trailing_return_1m: Option<f64>,
    #[serde(rename = "monthlyTrailingReturn3M")]
    pub monthly_trailing_return_3m: Option<f64>,
    #[serde(rename = "monthlyTrailingReturn6M")]
    pub monthly_trailing_return_6m: Option<f64>,
    #[serde(rename = "qtrlyTrailingReturnYTD")]
    pub qtrly_trailing_return_ytd: Option<f64>,
    #[serde(rename = "qtrlyTrailingReturn1M")]
    pub qtrly_trailing_return_1m: Option<f64>,
    #[serde(rename = "qtrlyTrailingReturn3M")]
    pub qtrly_trailing_return_3m: Option<f64>,
    #[serde(rename = "qtrlyTrailingReturn6M")]
    pub qtrly_trailing_return_6m: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deferred_sales_changes: Vec<SaleChargeValues>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub frontend_sales_changes: Vec<SaleChargeValues>,
    pub exchange_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NetAsset {
    pub value: Option<f64>,
    pub as_of_date: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Redemption {
    pub min_month: Option<String>,
    pub fee_percent: Option<String>,
    pub is_front_end: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub front_end_values: Vec<Values>,
    pub redemption_duration_type: Option<String>,
    pub is_sales: Option<String>,
    pub sales_duration_type: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sales_values: Vec<Values>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Values {
    pub low: Option<String>,
    pub high: Option<String>,
    pub percent: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SaleChargeValues {
    pub lowhigh: Option<String>,
    pub percent: Option<String>,
} 

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_quote_response_deserialization() {
        // Read the test data file
        let test_data_path = Path::new("tests/quote.json");
        let test_data = fs::read_to_string(test_data_path)
            .expect("Failed to read test data file");
        
        // Parse the JSON into a serde_json::Value first
        let json_value: serde_json::Value = serde_json::from_str(&test_data)
            .expect("Failed to parse test JSON");
        
        // Extract the QuoteResponse field
        let quote_response_value = json_value.get("QuoteResponse")
            .expect("No QuoteResponse field in test data")
            .clone();
        
        // Try to deserialize into our QuoteResponse struct
        let _: QuoteResponse = serde_json::from_value(quote_response_value)
            .expect("Failed to deserialize QuoteResponse");
    }

    #[test]
    fn test_quote_response_with_missing_fields() {
        // Test with minimal data to ensure optional fields work correctly
        let minimal_json = r#"{
            "QuoteResponse": {
                "QuoteData": [
                    {
                        "All": {
                            "companyName": "TEST COMPANY",
                            "lastTrade": 100.0
                        },
                        "dateTime": "12:00:00 EDT 01-01-2024",
                        "dateTimeUTC": 1704067200
                    }
                ]
            }
        }"#;
        
        let json_value: serde_json::Value = serde_json::from_str(minimal_json)
            .expect("Failed to parse minimal JSON");
        
        let quote_response_value = json_value.get("QuoteResponse")
            .expect("No QuoteResponse field in test data")
            .clone();
        
        let quote_response: QuoteResponse = serde_json::from_value(quote_response_value)
            .expect("Failed to deserialize minimal QuoteResponse");
        
        assert_eq!(quote_response.quote_data.len(), 1);
        
        let quote_data = &quote_response.quote_data[0];
        let all_details = quote_data.all.as_ref().expect("Should have All quote details");
        
        assert_eq!(all_details.company_name.as_deref(), Some("TEST COMPANY"));
        assert_eq!(all_details.last_trade, Some(100.0));
        assert_eq!(all_details.ask, None); // Should be None for missing field
        assert_eq!(all_details.bid, None); // Should be None for missing field
    }

    #[test]
    fn test_quote_response_empty_messages() {
        // Test that empty messages are handled correctly
        let json_with_messages = r#"{
            "QuoteResponse": {
                "QuoteData": [
                    {
                        "All": {
                            "companyName": "TEST COMPANY",
                            "lastTrade": 100.0
                        }
                    }
                ],
                "Messages": {
                    "Message": []
                }
            }
        }"#;
        
        let json_value: serde_json::Value = serde_json::from_str(json_with_messages)
            .expect("Failed to parse JSON with messages");
        
        let quote_response_value = json_value.get("QuoteResponse")
            .expect("No QuoteResponse field in test data")
            .clone();
        
        let quote_response: QuoteResponse = serde_json::from_value(quote_response_value)
            .expect("Failed to deserialize QuoteResponse with messages");
        
        assert_eq!(quote_response.quote_data.len(), 1);
        assert!(quote_response.messages.is_empty());
    }
}
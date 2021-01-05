use super::{session, Session, Store};
use crate::{empty_body, qs_params, MarketSession};
use crate::{Product, SortOrder};
use anyhow::Result;
use http::Method;
use session::CallbackProvider;
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

  pub async fn list(&self, callbacks: impl CallbackProvider) -> Result<Vec<Account>> {
    let resp: AccountListResponse = self
      .session
      .send(Method::GET, "/v1/accounts/list", empty_body(), callbacks)
      .await?;
    debug!("balance json: {}", serde_json::to_string_pretty(&resp)?);
    Ok(resp.response.accounts.account)
  }

  pub async fn balance(
    &self,
    account_id_key: &str,
    balance_request: BalanceRequest<'_>,
    callbacks: impl CallbackProvider,
  ) -> Result<BalanceResponse> {
    let balance: serde_json::Value = self
      .session
      .send(
        Method::GET,
        format!("/v1/accounts/{}/balance", account_id_key),
        qs_params(&balance_request)?,
        callbacks,
      )
      .await?;
    debug!("balance json: {}", serde_json::to_string_pretty(&balance)?);
    Ok(serde_json::from_value(balance.get("BalanceResponse").unwrap().clone())?)
  }

  pub async fn portfolio(
    &self,
    account_id_key: &str,
    params: PortfolioRequest,
    callbacks: impl CallbackProvider,
  ) -> Result<PortfolioResponse> {
    let portfolio: serde_json::Value = self
      .session
      .send(
        Method::GET,
        format!("/v1/accounts/{}/portfolio", account_id_key),
        qs_params(&params)?,
        callbacks,
      )
      .await?;
    debug!("portfolio json: {}", serde_json::to_string_pretty(&portfolio)?);
    Ok(serde_json::from_value(
      portfolio.get("PortfolioResponse").unwrap().clone(),
    )?)
  }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PortfolioRequest {
  pub count: Option<usize>,
  pub sort_by: Option<PortfolioColumn>,
  pub sort_order: Option<SortOrder>,
  pub market_session: Option<MarketSession>,
  pub totals_required: Option<bool>,
  pub lots_required: Option<bool>,
  pub view: Option<PortfolioView>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BalanceRequest<'a> {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub account_type: Option<AccountType>,
  pub inst_type: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub real_time_nav: Option<bool>,
}

impl<'a> Default for BalanceRequest<'a> {
  fn default() -> Self {
    Self {
      inst_type: "BROKERAGE",
      account_type: None,
      real_time_nav: None,
    }
  }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum AccountType {
  #[serde(rename = "AMMCHK")]
  Ammchk,
  #[serde(rename = "ARO")]
  Aro,
  #[serde(rename = "BCHK")]
  Bchk,
  #[serde(rename = "BENFIRA")]
  Benfira,
  #[serde(rename = "BENFROTHIRA")]
  Benfrothira,
  #[serde(rename = "BENF_ESTATE_IRA")]
  BenfEstateIra,
  #[serde(rename = "BENF_MINOR_IRA")]
  BenfMinorIra,
  #[serde(rename = "BENF_ROTH_ESTATE_IRA")]
  BenfRothEstateIra,
  #[serde(rename = "BENF_ROTH_MINOR_IRA")]
  BenfRothMinorIra,
  #[serde(rename = "BENF_ROTH_TRUST_IRA")]
  BenfRothTrustIra,
  #[serde(rename = "BENF_TRUST_IRA")]
  BenfTrustIra,
  #[serde(rename = "BRKCD")]
  Brkcd,
  #[serde(rename = "BROKER")]
  Broker,
  #[serde(rename = "CASH")]
  Cash,
  #[serde(rename = "C_CORP")]
  CCorp,
  #[serde(rename = "CONTRIBUTORY")]
  Contributory,
  #[serde(rename = "COVERDELL_ESA")]
  CoverdellEsa,
  #[serde(rename = "CONVERSION_ROTH_IRA")]
  ConversionRothIra,
  #[serde(rename = "CREDITCARD")]
  Creditcard,
  #[serde(rename = "COMM_PROP")]
  CommProp,
  #[serde(rename = "CONSERVATOR")]
  Conservator,
  #[serde(rename = "CORPORATION")]
  Corporation,
  #[serde(rename = "CSA")]
  Csa,
  #[serde(rename = "CUSTODIAL")]
  Custodial,
  #[serde(rename = "DVP")]
  Dvp,
  #[serde(rename = "ESTATE")]
  Estate,
  #[serde(rename = "EMPCHK")]
  Empchk,
  #[serde(rename = "EMPMMCA")]
  Empmmca,
  #[serde(rename = "ETCHK")]
  Etchk,
  #[serde(rename = "ETMMCHK")]
  Etmmchk,
  #[serde(rename = "HEIL")]
  Heil,
  #[serde(rename = "HELOC")]
  Heloc,
  #[serde(rename = "INDCHK")]
  Indchk,
  #[serde(rename = "INDIVIDUAL")]
  Individual,
  #[serde(rename = "INDIVIDUAL_K")]
  IndividualK,
  #[serde(rename = "INVCLUB")]
  Invclub,
  #[serde(rename = "INVCLUB_C_CORP")]
  InvclubCCorp,
  #[serde(rename = "INVCLUB_LLC_C_CORP")]
  InvclubLlcCCorp,
  #[serde(rename = "INVCLUB_LLC_PARTNERSHIP")]
  InvclubLlcPartnership,
  #[serde(rename = "INVCLUB_LLC_S_CORP")]
  InvclubLlcSCorp,
  #[serde(rename = "INVCLUB_PARTNERSHIP")]
  InvclubPartnership,
  #[serde(rename = "INVCLUB_S_CORP")]
  InvclubSCorp,
  #[serde(rename = "INVCLUB_TRUST")]
  InvclubTrust,
  #[serde(rename = "IRA_ROLLOVER")]
  IraRollover,
  #[serde(rename = "JOINT")]
  Joint,
  #[serde(rename = "JTTEN")]
  Jtten,
  #[serde(rename = "JTWROS")]
  Jtwros,
  #[serde(rename = "LLC_C_CORP")]
  LlcCCorp,
  #[serde(rename = "LLC_PARTNERSHIP")]
  LlcPartnership,
  #[serde(rename = "LLC_S_CORP")]
  LlcSCorp,
  #[serde(rename = "LLP")]
  Llp,
  #[serde(rename = "LLP_C_CORP")]
  LlpCCorp,
  #[serde(rename = "LLP_S_CORP")]
  LlpSCorp,
  #[serde(rename = "IRA")]
  Ira,
  #[serde(rename = "IRACD")]
  Iracd,
  #[serde(rename = "MONEY_PURCHASE")]
  MoneyPurchase,
  #[serde(rename = "MARGIN")]
  Margin,
  #[serde(rename = "MRCHK")]
  Mrchk,
  #[serde(rename = "MUTUAL_FUND")]
  MutualFund,
  #[serde(rename = "NONCUSTODIAL")]
  Noncustodial,
  #[serde(rename = "NON_PROFIT")]
  NonProfit,
  #[serde(rename = "OTHER")]
  Other,
  #[serde(rename = "PARTNER")]
  Partner,
  #[serde(rename = "PARTNERSHIP")]
  Partnership,
  #[serde(rename = "PARTNERSHIP_C_CORP")]
  PartnershipCCorp,
  #[serde(rename = "PARTNERSHIP_S_CORP")]
  PartnershipSCorp,
  #[serde(rename = "PDT_ACCOUNT")]
  PdtAccount,
  #[serde(rename = "PM_ACCOUNT")]
  PmAccount,
  #[serde(rename = "PREFCD")]
  Prefcd,
  #[serde(rename = "PREFIRACD")]
  Prefiracd,
  #[serde(rename = "PROFIT_SHARING")]
  ProfitSharing,
  #[serde(rename = "PROPRIETARY")]
  Proprietary,
  #[serde(rename = "REGCD")]
  Regcd,
  #[serde(rename = "ROTHIRA")]
  Rothira,
  #[serde(rename = "ROTH_INDIVIDUAL_K")]
  RothIndividualK,
  #[serde(rename = "ROTH_IRA_MINORS")]
  RothIraMinors,
  #[serde(rename = "SARSEPIRA")]
  Sarsepira,
  #[serde(rename = "S_CORP")]
  SCorp,
  #[serde(rename = "SEPIRA")]
  Sepira,
  #[serde(rename = "SIMPLE_IRA")]
  SimpleIra,
  #[serde(rename = "TIC")]
  Tic,
  #[serde(rename = "TRD_IRA_MINORS")]
  TrdIraMinors,
  #[serde(rename = "TRUST")]
  Trust,
  #[serde(rename = "VARCD")]
  Varcd,
  #[serde(rename = "VARIRACD")]
  Variracd,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum PortfolioView {
  #[serde(rename = "PERFORMANCE")]
  Performance,
  #[serde(rename = "FUNDAMENTAL")]
  Fundamental,
  #[serde(rename = "OPTIONSWATCH")]
  Optionswatch,
  #[serde(rename = "QUICK")]
  Quick,
  #[serde(rename = "COMPLETE")]
  Complete,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum PortfolioColumn {
  #[serde(rename = "SYMBOL")]
  Symbol,
  #[serde(rename = "TYPE_NAME")]
  TypeName,
  #[serde(rename = "EXCHANGE_NAME")]
  ExchangeName,
  #[serde(rename = "CURRENCY")]
  Currency,
  #[serde(rename = "QUANTITY")]
  Quantity,
  #[serde(rename = "LONG_OR_SHORT")]
  LongOrShort,
  #[serde(rename = "DATE_ACQUIRED")]
  DateAcquired,
  #[serde(rename = "PRICEPAID")]
  Pricepaid,
  #[serde(rename = "TOTAL_GAIN")]
  TotalGain,
  #[serde(rename = "TOTAL_GAIN_PCT")]
  TotalGainPct,
  #[serde(rename = "MARKET_VALUE")]
  MarketValue,
  #[serde(rename = "BI")]
  Bi,
  #[serde(rename = "ASK")]
  Ask,
  #[serde(rename = "PRICE_CHANGE")]
  PriceChange,
  #[serde(rename = "PRICE_CHANGE_PCT")]
  PriceChangePct,
  #[serde(rename = "VOLUME")]
  Volume,
  #[serde(rename = "WEEK_52_HIGH")]
  Week52High,
  #[serde(rename = "WEEK_52_LOW")]
  Week52Low,
  #[serde(rename = "EPS")]
  Eps,
  #[serde(rename = "PE_RATIO")]
  PeRatio,
  #[serde(rename = "OPTION_TYPE")]
  OptionType,
  #[serde(rename = "STRIKE_PRICE")]
  StrikePrice,
  #[serde(rename = "PREMIUM")]
  Premium,
  #[serde(rename = "EXPIRATION")]
  Expiration,
  #[serde(rename = "DAYS_GAIN")]
  DaysGain,
  #[serde(rename = "COMMISSION")]
  Commission,
  #[serde(rename = "MARKETCAP")]
  Marketcap,
  #[serde(rename = "PREV_CLOSE")]
  PrevClose,
  #[serde(rename = "OPEN")]
  Open,
  #[serde(rename = "DAYS_RANGE")]
  DaysRange,
  #[serde(rename = "TOTAL_COST")]
  TotalCost,
  #[serde(rename = "DAYS_GAIN_PCT")]
  DaysGainPct,
  #[serde(rename = "PCT_OF_PORTFOLIO")]
  PctOfPortfolio,
  #[serde(rename = "LAST_TRADE_TIME")]
  LastTradeTime,
  #[serde(rename = "BASE_SYMBOL_PRICE")]
  BaseSymbolPrice,
  #[serde(rename = "WEEK_52_RANGE")]
  Week52Range,
  #[serde(rename = "LAST_TRADE")]
  LastTrade,
  #[serde(rename = "SYMBOL_DESC")]
  SymbolDesc,
  #[serde(rename = "BID_SIZE")]
  BidSize,
  #[serde(rename = "ASK_SIZE")]
  AskSize,
  #[serde(rename = "OTHER_FEES")]
  OtherFees,
  #[serde(rename = "HELD_AS")]
  HeldAs,
  #[serde(rename = "OPTION_MULTIPLIER")]
  OptionMultiplier,
  #[serde(rename = "DELIVERABLES")]
  Deliverables,
  #[serde(rename = "COST_PERSHARE")]
  CostPershare,
  #[serde(rename = "DIVIDEND")]
  Dividend,
  #[serde(rename = "DIV_YIELD")]
  DivYield,
  #[serde(rename = "DIV_PAY_DATE")]
  DivPayDate,
  #[serde(rename = "EST_EARN")]
  EstEarn,
  #[serde(rename = "EX_DIV_DATE")]
  ExDivDate,
  #[serde(rename = "TEN_DAY_AVG_VOL")]
  TenDayAvgVol,
  #[serde(rename = "BETA")]
  Beta,
  #[serde(rename = "BID_ASK_SPREAD")]
  BidAskSpread,
  #[serde(rename = "MARGINABLE")]
  Marginable,
  #[serde(rename = "DELTA_52WK_HI")]
  Delta52WkHi,
  #[serde(rename = "DELTA_52WK_LOW")]
  Delta52WkLow,
  #[serde(rename = "PERF_1MON")]
  Perf1Mon,
  #[serde(rename = "ANNUAL_DIV")]
  AnnualDiv,
  #[serde(rename = "PERF_12MON")]
  Perf12Mon,
  #[serde(rename = "PERF_3MON")]
  Perf3Mon,
  #[serde(rename = "PERF_6MON")]
  Perf6Mon,
  #[serde(rename = "PRE_DAY_VOL")]
  PreDayVol,
  #[serde(rename = "SV_1MON_AVG")]
  Sv1MonAvg,
  #[serde(rename = "SV_10DAY_AVG")]
  Sv10DayAvg,
  #[serde(rename = "SV_20DAY_AVG")]
  Sv20DayAvg,
  #[serde(rename = "SV_2MON_AVG")]
  Sv2MonAvg,
  #[serde(rename = "SV_3MON_AVG")]
  Sv3MonAvg,
  #[serde(rename = "SV_4MON_AVG")]
  Sv4MonAvg,
  #[serde(rename = "SV_6MON_AVG")]
  Sv6MonAvg,
  #[serde(rename = "DELTA")]
  Delta,
  #[serde(rename = "GAMMA")]
  Gamma,
  #[serde(rename = "IV_PCT")]
  IvPct,
  #[serde(rename = "THETA")]
  Theta,
  #[serde(rename = "VEGA")]
  Vega,
  #[serde(rename = "ADJ_NONADJ_FLAG")]
  AdjNonadjFlag,
  #[serde(rename = "DAYS_EXPIRATION")]
  DaysExpiration,
  #[serde(rename = "OPEN_INTEREST")]
  OpenInterest,
  #[serde(rename = "INSTRINIC_VALUE")]
  InstrinicValue,
  #[serde(rename = "RHO")]
  Rho,
  #[serde(rename = "TYPE_CODE")]
  TypeCode,
  #[serde(rename = "DISPLAY_SYMBOL")]
  DisplaySymbol,
  #[serde(rename = "AFTER_HOURS_PCTCHANGE")]
  AfterHoursPctchange,
  #[serde(rename = "PRE_MARKET_PCTCHANGE")]
  PreMarketPctchange,
  #[serde(rename = "EXPAND_COLLAPSE_FLAG")]
  ExpandCollapseFlag,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct AccountListResponse {
  #[serde(rename = "AccountListResponse")]
  response: AccountList,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct AccountList {
  #[serde(rename = "Accounts")]
  accounts: AccountHolder,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct AccountHolder {
  #[serde(rename = "Account", skip_serializing_if = "Vec::is_empty")]
  account: Vec<Account>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Account {
  #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct BalanceResponse {
  pub account_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub institution_type: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub as_of_date: Option<i64>,
  pub account_type: String,
  pub option_level: String,
  pub account_description: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quote_mode: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub day_trader_status: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub account_mode: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub account_desc: Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub open_calls: Vec<OpenCalls>,
  #[serde(rename = "Cash", skip_serializing_if = "Option::is_none")]
  pub cash: Option<Cash>,
  #[serde(rename = "Margin", skip_serializing_if = "Option::is_none")]
  pub margin: Option<Margin>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lending: Option<Lending>,
  #[serde(rename = "Computed")]
  pub computed_balance: ComputedBalance,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OpenCalls {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub min_equity_call: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fed_call: Option<f64>,
  pub cash_call: f64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub house_call: Option<f64>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Cash {
  pub funds_for_open_orders_cash: f64,
  pub money_mkt_balance: f64,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Margin {
  pub dt_cash_open_order_reserve: f64,
  pub dt_margin_open_order_reserve: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ComputedBalance {
  pub cash_available_for_investment: f64,
  pub cash_available_for_withdrawal: f64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total_available_for_withdrawal: Option<f64>,
  pub net_cash: f64,
  pub cash_balance: f64,
  pub settled_cash_for_investment: f64,
  pub un_settled_cash_for_investment: f64,
  pub funds_withheld_from_purchase_power: f64,
  pub funds_withheld_from_withdrawal: f64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub margin_buying_power: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cash_buying_power: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dt_margin_buying_power: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dt_cash_buying_power: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub margin_balance: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub short_adjust_balance: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub regt_equity: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub regt_equity_percent: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub account_balance: Option<f64>,
  #[serde(rename = "OpenCalls")]
  pub open_calls: OpenCalls,
  #[serde(rename = "RealTimeValues")]
  pub real_time_values: RealTimeValues,
  #[serde(rename = "PortfolioMargin")]
  pub portfolio_margin: Option<PortfolioMargin>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RealTimeValues {
  pub total_account_value: f64,
  pub net_mv: f64,
  pub net_mv_long: f64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub net_mv_short: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total_long_value: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PortfolioResponse {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub totals: Option<PortfolioTotals>,
  #[serde(rename = "AccountPortfolio", skip_serializing_if = "Vec::is_empty")]
  pub account_portfolio: Vec<AccountPortfolio>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PortfolioTotals {
  pub todays_gain_loss: f64,
  pub todays_gain_loss_pct: f64,
  pub total_market_value: f64,
  pub total_gain_loss: f64,
  pub total_gain_loss_pct: f64,
  pub total_price_paid: f64,
  pub cash_balance: f64,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AccountPortfolio {
  pub account_id: String,
  pub next: String,
  pub total_no_of_pages: i32,
  pub next_page_no: String,
  #[serde(rename = "Position", skip_serializing_if = "Vec::is_empty")]
  pub position: Vec<PortfolioPosition>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PortfolioPosition {
  pub position_id: i64,
  pub account_id: String,
  #[serde(rename = "Product")]
  pub product: Product,
  pub osi_key: String,
  pub symbol_description: String,
  pub date_acquired: i64,
  pub price_paid: f64,
  pub price: f64,
  pub commissions: f64,
  pub other_fees: f64,
  pub quantity: f64,
  pub position_indicator: String,
  pub position_type: String,
  pub change: f64,
  pub change_pct: f64,
  pub days_gain: f64,
  pub days_gain_pct: f64,
  pub market_value: f64,
  pub total_cost: f64,
  pub total_gain: f64,
  pub total_gain_pct: f64,
  pub pct_of_portfolio: f64,
  pub cost_per_share: f64,
  pub today_commissions: f64,
  pub today_fees: f64,
  pub today_price_paid: f64,
  pub today_quantity: f64,
  pub quotestatus: String,
  #[serde(rename = "dateTimeUTC")]
  pub date_time_utc: i64,
  pub adj_prev_close: f64,
  #[serde(rename = "Performance", skip_serializing_if = "Option::is_none")]
  pub performance: Option<PerformanceView>,
  #[serde(rename = "Fundamental", skip_serializing_if = "Option::is_none")]
  pub fundamental: Option<FundamentalView>,
  #[serde(rename = "OptionsWatch", skip_serializing_if = "Option::is_none")]
  pub options_watch: Option<OptionsWatchView>,
  #[serde(rename = "Quick", skip_serializing_if = "Option::is_none")]
  pub quick: Option<QuickView>,
  #[serde(rename = "Complete", skip_serializing_if = "Option::is_none")]
  pub complete: Option<CompleteView>,
  pub lots_details: String,
  pub quote_details: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub position_lot: Vec<PositionLot>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PerformanceView {
  pub change: f64,
  pub change_pct: f64,
  pub last_trade: f64,
  pub days_gain: f64,
  pub total_gain: f64,
  pub total_gain_pct: f64,
  pub market_value: f64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quote_status: Option<QuoteStatus>,
  pub last_trade_time: i64,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum QuoteStatus {
  #[serde(rename = "REALTIME")]
  Realtime,
  #[serde(rename = "DELAYED")]
  Delayed,
  #[serde(rename = "CLOSING")]
  Closing,
  #[serde(rename = "EH_REALTIME")]
  EhRealtime,
  #[serde(rename = "EH_BEFORE_OPEN")]
  EhBeforeOpen,
  #[serde(rename = "EH_CLOSED")]
  EhClosed,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct FundamentalView {
  pub last_trade: f64,
  pub last_trade_time: i64,
  pub change: f64,
  pub change_pct: f64,
  pub pe_ratio: f64,
  pub eps: f64,
  pub dividend: f64,
  pub div_yield: f64,
  pub market_cap: f64,
  pub week_52_range: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quote_status: Option<QuoteStatus>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionsWatchView {
  pub last_trade: f64,
  pub last_trade_time: i64,
  pub base_symbol_and_price: String,
  pub premium: f64,
  pub bid: f64,
  pub ask: f64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quote_status: Option<QuoteStatus>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct QuickView {
  pub last_trade: f64,
  pub last_trade_time: i64,
  pub change: f64,
  pub change_pct: f64,
  pub volume: i64,
  pub seven_day_current_yield: f64,
  pub annual_total_return: f64,
  pub weighted_average_maturity: f64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quote_status: Option<QuoteStatus>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CompleteView {
  pub price_adjusted_flag: bool,
  pub price: f64,
  pub adj_price: f64,
  pub change: f64,
  pub change_pct: f64,
  pub prev_close: f64,
  pub adj_prev_close: f64,
  pub last_trade: f64,
  pub last_trade_time: i64,
  pub adj_last_trade: f64,
  pub symbol_description: String,
  pub perform_1_month: f64,
  pub perform_3_month: f64,
  pub perform_6_month: f64,
  pub perform_12_month: f64,
  pub prev_day_volume: i64,
  pub ten_day_volume: i64,
  pub beta: f64,
  pub sv_10_days_avg: f64,
  pub sv_20_days_avg: f64,
  pub sv_1_mon_avg: f64,
  pub sv_2_mon_avg: f64,
  pub sv_3_mon_avg: f64,
  pub sv_4_mon_avg: f64,
  pub sv_6_mon_avg: f64,
  pub week_52_high: f64,
  pub week_52_low: f64,
  pub week_52_range: String,
  pub market_cap: f64,
  pub days_range: String,
  pub delta_52_wk_high: f64,
  pub delta_52_wk_low: f64,
  pub currency: String,
  pub exchange: String,
  pub marginable: bool,
  pub bid: f64,
  pub ask: f64,
  pub bid_ask_spread: f64,
  pub bid_size: i64,
  pub ask_size: i64,
  pub open: f64,
  pub delta: f64,
  pub gamma: f64,
  pub iv_pct: f64,
  pub rho: f64,
  pub theta: f64,
  pub vega: f64,
  pub base_symbol_and_price: String,
  pub premium: f64,
  pub days_to_expiration: i32,
  pub intrinsic_value: f64,
  pub open_interest: f64,
  pub options_adjusted_flag: bool,
  pub deliverables_str: String,
  pub option_multiplier: f64,
  pub est_earnings: f64,
  pub eps: f64,
  pub pe_ratio: f64,
  pub annual_dividend: f64,
  pub dividend: f64,
  pub div_yield: f64,
  pub div_pay_date: i64,
  pub ex_dividend_date: i64,
  pub cusip: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quote_status: Option<QuoteStatus>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PositionLot {
  pub position_id: i64,
  pub position_log_id: i64,
  pub price: f64,
  pub term_code: i32,
  pub days_gain: f64,
  pub day_gain_pct: f64,
  pub market_value: f64,
  pub total_cost: f64,
  pub total_cost_for_gain_pct: f64,
  pub total_gain: f64,
  pub lot_source_code: i32,
  pub original_qty: f64,
  pub remaining_qty: f64,
  pub available_qty: f64,
  pub order_no: i64,
  pub leg_no: i32,
  pub acquired_date: i64,
  pub location_code: i32,
  pub exchange_rate: f64,
  pub settlement_currency: String,
  pub payment_currency: String,
  pub adj_price: f64,
  pub comm_per_share: f64,
  pub fees_per_share: f64,
  pub premium_adj: f64,
  pub short_type: i32,
}

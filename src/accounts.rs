use std::collections::BTreeSet;

use super::{session, Session, Store};
use anyhow::Result;
use http::Method;
use session::CallbackProvider;
use strum::EnumString;
use crate::Product;
use crate::MarketSession;
use std::sync::Arc;

fn no_body() -> Option<()> {
    None
}

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

    pub async fn account_list(&self, callbacks: impl CallbackProvider) -> Result<Vec<Account>> {
        let resp: AccountListResponse = self
            .session
            .send(Method::GET, "/v1/accounts/list", no_body(), callbacks)
            .await?;
        Ok(resp.response.accounts.account)
    }

    pub async fn account_balance(
        &self,
        account: &Account,
        real_time: bool,
        callbacks: impl CallbackProvider,
    ) -> Result<BalanceResponse> {
        let balance: serde_json::Value = self
            .session
            .send(
                Method::GET,
                format!("/v1/accounts/{}/balance", account.account_id_key),
                Some(&[
                    ("instType", &account.institution_type),
                    ("realTimeNAV", &real_time.to_string()),
                ]),
                callbacks,
            )
            .await?;
        Ok(serde_json::from_value(
            balance.get("BalanceResponse").unwrap().clone(),
        )?)
    }

    pub async fn portfolio(
        &self,
        account: &Account,
        params: PortfolioRequest,
        callbacks: impl CallbackProvider,
    ) -> Result<PortfolioResponse> {
        let qss = serde_urlencoded::to_string(&params)?;
        let qs: BTreeSet<(String, String)> = serde_urlencoded::from_str(&qss)?;
        let qsv = if qs.is_empty() { None } else { Some(qs) };

        let portfolio: serde_json::Value = self
            .session
            .send(
                Method::GET,
                format!("/v1/accounts/{}/portfolio", account.account_id_key),
                qsv,
                callbacks,
            )
            .await?;
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum SortOrder {
    #[serde(rename = "ASC")]
    Asc,
    #[serde(rename = "DESC")]
    Desc,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
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

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
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
#[serde(rename_all = "camelCase", default)]
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
#[serde(rename_all = "camelCase", default)]
pub struct OpenCalls {
    pub min_equity_call: Option<f64>,
    pub fed_call: Option<f64>,
    pub cash_call: f64,
    pub house_call: Option<f64>,
}
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Cash {
    pub funds_for_open_orders_cash: f64,
    pub money_mkt_balance: f64,
}
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Margin {
    pub dt_cash_open_order_reserve: f64,
    pub dt_margin_open_order_reserve: f64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
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

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
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

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RealTimeValues {
    pub total_account_value: f64,
    pub net_mv: f64,
    pub net_mv_long: f64,
    pub net_mv_short: Option<f64>,
    pub total_long_value: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PortfolioResponse {
    pub totals: Option<PortfolioTotals>,
    #[serde(rename = "AccountPortfolio")]
    pub account_portfolio: Vec<AccountPortfolio>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
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
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AccountPortfolio {
    pub account_id: String,
    pub next: String,
    pub total_no_of_pages: i32,
    pub next_page_no: String,
    #[serde(rename = "Position")]
    pub position: Vec<PortfolioPosition>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
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
    #[serde(rename = "Performance")]
    pub performance: Option<PerformanceView>,
    #[serde(rename = "Fundamental")]
    pub fundamental: Option<FundamentalView>,
    #[serde(rename = "OptionsWatch")]
    pub options_watch: Option<OptionsWatchView>,
    #[serde(rename = "Quick")]
    pub quick: Option<QuickView>,
    #[serde(rename = "Complete")]
    pub complete: Option<CompleteView>,
    pub lots_details: String,
    pub quote_details: String,
    pub position_lot: Vec<PositionLot>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PerformanceView {
    pub change: f64,
    pub change_pct: f64,
    pub last_trade: f64,
    pub days_gain: f64,
    pub total_gain: f64,
    pub total_gain_pct: f64,
    pub market_value: f64,
    pub quote_status: Option<QuoteStatus>,
    pub last_trade_time: i64,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize, Default)]
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
    pub quote_status: Option<QuoteStatus>,
}
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionsWatchView {
    pub last_trade: f64,
    pub last_trade_time: i64,
    pub base_symbol_and_price: String,
    pub premium: f64,
    pub bid: f64,
    pub ask: f64,
    pub quote_status: Option<QuoteStatus>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
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
    pub quote_status: Option<QuoteStatus>,
}
#[derive(Debug, Deserialize, Serialize, Default)]
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
    pub quote_status: Option<QuoteStatus>,
}
#[derive(Debug, Deserialize, Serialize, Default)]
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

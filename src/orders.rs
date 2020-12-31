use crate::{qs_params, session::CallbackProvider};
use crate::{MarketSession, Product, SecurityType, Session, Store};
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

    pub async fn list(
        &self,
        account_id_key: &str,
        params: ListOrdersRequest,
        callbacks: impl CallbackProvider,
    ) -> Result<OrdersResponse> {
        let orders: serde_json::Value = self
            .session
            .send(
                Method::GET,
                format!("/v1/accounts/{}/orders", account_id_key),
                qs_params(&params)?,
                callbacks,
            )
            .await?;
        debug!("orders json: {}", serde_json::to_string_pretty(&orders)?);
        Ok(serde_json::from_value(
            orders.get("OrdersResponse").unwrap().clone(),
        )?)
    }

    pub async fn preview(
        &self,
        account_id_key: &str,
        params: PreviewOrderRequest,
        callbacks: impl CallbackProvider,
    ) -> Result<PreviewOrderResponse> {
        let preview: serde_json::Value = self
            .session
            .clone()
            .send(
                Method::POST,
                format!("/v1/accounts/{}/orders/preview", account_id_key),
                Some(params),
                callbacks,
            )
            .await?;
        debug!("preview json: {}", serde_json::to_string_pretty(&preview)?);
        Ok(serde_json::from_value(
            preview.get("PreviewOrderResponse").unwrap().clone(),
        )?)
    }

    pub async fn place(
        &self,
        account_id_key: &str,
        params: PlaceOrderRequest,
        callbacks: impl CallbackProvider,
    ) -> Result<PlaceOrderResponse> {
        let place: serde_json::Value = self
            .session
            .clone()
            .send(
                Method::POST,
                format!("/v1/accounts/{}/orders/place", account_id_key),
                Some(params),
                callbacks,
            )
            .await?;
        debug!(
            "placed order json: {}",
            serde_json::to_string_pretty(&place)?
        );
        Ok(serde_json::from_value(
            place.get("PlaceOrderResponse").unwrap().clone(),
        )?)
    }

    pub async fn cancel(
        &self,
        account_id_key: &str,
        params: CancelOrderRequest,
        callback: impl CallbackProvider,
    ) -> Result<CancelOrderResponse> {
        let cancellation: serde_json::Value = self
            .session
            .clone()
            .send(
                Method::PUT,
                format!("/v1/accounts/{}/orders/cancel", account_id_key),
                Some(params),
                callback,
            )
            .await?;
        debug!(
            "cancellation json: {}",
            serde_json::to_string_pretty(&cancellation)?
        );
        Ok(serde_json::from_value(
            cancellation.get("CancelOrderResponse").unwrap().clone(),
        )?)
    }

    pub async fn change_preview(
        &self,
        account_id_key: &str,
        order_id: &str,
        params: PreviewOrderRequest,
        callback: impl CallbackProvider,
    ) -> Result<PreviewOrderResponse> {
        let preview: serde_json::Value = self
            .session
            .clone()
            .send(
                Method::PUT,
                format!(
                    "/v1/accounts/{}/orders/{}/change/preview",
                    account_id_key, order_id
                ),
                Some(params),
                callback,
            )
            .await?;
        debug!(
            "changed preview json: {}",
            serde_json::to_string_pretty(&preview)?
        );
        Ok(serde_json::from_value(
            preview.get("PreviewOrderResponse").unwrap().clone(),
        )?)
    }

    pub async fn change_order(
        &self,
        account_id_key: &str,
        order_id: &str,
        params: PlaceOrderRequest,
        callback: impl CallbackProvider,
    ) -> Result<PlaceOrderResponse> {
        let place: serde_json::Value = self
            .session
            .clone()
            .send(
                Method::PUT,
                format!(
                    "/v1/accounts/{}/orders/{}/change/place",
                    account_id_key, order_id
                ),
                Some(params),
                callback,
            )
            .await?;
        debug!(
            "changed placed order json: {}",
            serde_json::to_string_pretty(&place)?
        );
        Ok(serde_json::from_value(
            place.get("PlaceOrderResponse").unwrap().clone(),
        )?)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CancelOrderRequest {
    pub order_id: i64,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CancelOrderResponse {
    pub account_id: String,
    pub order_id: i64,
    pub cancel_time: i64,
    #[serde(rename = "Messages", skip_serializing_if = "Messages::is_empty")]
    pub messages: Messages,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PlaceOrderRequest {
    pub order_type: Option<OrderType>,
    pub client_order_id: String,
    #[serde(rename = "Order")]
    pub order: Vec<OrderDetail>,
    #[serde(rename = "PreviewIds", skip_serializing_if = "Vec::is_empty")]
    pub preview_ids: Vec<PreviewId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PlaceOrderResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
    #[serde(skip_serializing_if = "Messages::is_empty")]
    pub message_list: Messages,
    pub total_order_value: f64,
    pub total_commission: f64,
    pub order_id: i64,
    #[serde(rename = "Order", skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<OrderDetail>,
    pub dst_flag: bool,
    pub account_id: String,
    pub option_level_cd: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_level_cd: Option<MarginLevelCd>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portfolio_margin: Option<PortfolioMargin>,
    pub is_employee: bool,
    pub commission_msg: String,
    #[serde(rename = "OrderIds", skip_serializing_if = "Vec::is_empty")]
    pub order_ids: Vec<OrderId>,
    #[serde(rename = "Disclosure", skip_serializing_if = "Option::is_none")]
    pub disclosure: Option<Disclosure>,
    pub placed_time: i64,
    pub client_order_id: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OrderId {
    pub order_id: i64,
    pub cash_margin: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PreviewOrderRequest {
    pub order_type: Option<OrderType>,
    pub client_order_id: String,
    #[serde(rename = "Order")]
    pub order: Vec<OrderDetail>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PreviewOrderResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<OrderType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_list: Option<Messages>,
    pub total_order_value: f64,
    pub total_commission: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<OrderDetail>,
    #[serde(rename = "PreviewIds", skip_serializing_if = "Vec::is_empty")]
    pub preview_ids: Vec<PreviewId>,
    pub preview_time: i64,
    pub dst_flag: bool,
    pub account_id: String,
    pub option_level_cd: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_level_cd: Option<MarginLevelCd>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portfolio_margin: Option<PortfolioMargin>,
    pub is_employee: bool,
    pub commission_message: String,
    #[serde(rename = "Disclosure", skip_serializing_if = "Option::is_none")]
    pub disclosure: Option<Disclosure>,
    pub client_order_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bp_details: Option<MarginBuyingPowerDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cash_bp_details: Option<CashBuyingPowerDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dt_bp_details: Option<DtBuyingPowerDetails>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OrderBuyPowerEffect {
    pub current_bp: f64,
    pub current_oor: f64,
    pub current_net_bp: f64,
    pub current_order_impact: f64,
    pub net_bp: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CashBuyingPowerDetails {
    #[serde(rename = "settledUnsettled", skip_serializing_if = "Option::is_none")]
    pub settled_unsettled: Option<OrderBuyPowerEffect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled: Option<OrderBuyPowerEffect>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct MarginBuyingPowerDetails {
    #[serde(rename = "nonMarginable", skip_serializing_if = "Option::is_none")]
    pub non_marginable: Option<OrderBuyPowerEffect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marginable: Option<OrderBuyPowerEffect>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct DtBuyingPowerDetails {
    #[serde(rename = "nonMarginable", skip_serializing_if = "Option::is_none")]
    pub non_marginable: Option<OrderBuyPowerEffect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marginable: Option<OrderBuyPowerEffect>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Disclosure {
    pub eh_disclosure_flag: bool,
    pub ah_disclosure_flag: bool,
    pub conditional_disclosure_flag: bool,
    pub ao_disclosure_flag: bool,
    #[serde(rename = "mfFLConsent")]
    pub mf_fl_consent: bool,
    #[serde(rename = "mfEOConsent")]
    pub mf_eo_consent: bool,
}

#[derive(Debug, Deserialize, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PortfolioMargin {
    pub house_excess_equity_new: f64,
    pub pm_eligible: f64,
    pub house_excess_equity_curr: f64,
    pub house_excess_equity_change: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PreviewId {
    pub preview_id: i64,
    pub cash_margin: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ListOrdersRequest {
    pub marker: Option<usize>,
    pub count: Option<usize>,
    pub status: Option<OrderStatus>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub symbol: Option<Vec<String>>,
    pub security_type: Option<SecurityType>,
    pub transaction_type: Option<TransactionType>,
    pub market_session: Option<MarketSession>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OrdersResponse {
    pub marker: String,
    pub next: String,
    #[serde(rename = "Order", skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<Order>,
    #[serde(rename = "Messages", skip_serializing_if = "Messages::is_empty")]
    pub messages: Messages,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Order {
    pub order_id: i64,
    pub details: String,
    pub order_type: String,
    pub total_order_value: f64,
    pub total_commission: f64,
    #[serde(rename = "OrderDetail")]
    pub order_detail: Vec<OrderDetail>,
    #[serde(rename = "Events")]
    pub events: Events,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct OrderDetail {
    pub order_number: isize,
    pub account_id: String,
    pub preview_time: i64,
    pub placed_time: i64,
    pub executed_time: i64,
    pub order_value: f64,
    pub status: Option<OrderStatus>,
    pub order_type: Option<OrderType>,
    pub order_term: Option<OrderTerm>,
    pub price_type: Option<PriceType>,
    pub price_value: String,
    pub limit_price: f64,
    pub stop_price: f64,
    pub stop_limit_price: f64,
    pub offset_type: Option<OffsetType>,
    pub offset_value: f64,
    pub market_session: Option<MarketSession>,
    pub routing_destination: RoutingDestination,
    pub bracketed_limit_price: f64,
    pub initial_stop_price: f64,
    pub trail_price: f64,
    pub trigger_price: f64,
    pub condition_price: f64,
    pub condition_type: Option<ConditionType>,
    pub condition_follow_price: Option<ConditionFollowPrice>,
    pub condition_security_type: String,
    pub replaced_by_order_id: isize,
    pub replaces_order_id: isize,
    pub all_or_none: bool,
    pub preview_id: i64,
    #[serde(rename = "Instrument")]
    pub instrument: Vec<Instrument>,
    #[serde(rename = "messages", skip_serializing_if = "Option::is_none")]
    pub messages: Option<Messages>,
    pub pre_clearance_code: String,
    pub override_restricted_cd: i32,
    pub investment_amount: f64,
    pub position_quantity: Option<PositionQuantity>,
    pub aip_flag: bool,
    pub eq_qual: Option<EgQual>,
    pub re_invest_option: Option<ReInvestOption>,
    pub estimated_commission: f64,
    pub estimated_fees: f64,
    pub estimated_total_amount: f64,
    pub net_price: f64,
    pub net_bid: f64,
    pub net_ask: f64,
    pub gcd: i32,
    pub ratio: String,
    pub mfprice_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Event {
    pub name: EventName,
    pub date_time: i64,
    pub order_number: isize,
    pub instrument: Vec<Instrument>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Instrument {
    pub product: Product,
    pub symbol_description: String,
    pub order_action: Option<OrderAction>,
    pub quantity_type: Option<QuantityType>,
    pub quantity: f64,
    pub cancel_quantity: f64,
    pub ordered_quantity: f64,
    pub filled_quantity: f64,
    pub average_execution_price: f64,
    pub estimated_commission: f64,
    pub estimated_fees: f64,
    pub bid: f64,
    pub ask: f64,
    pub lastprice: f64,
    pub currency: Currency,
    #[serde(rename = "Lots")]
    pub lots: Lots,
    #[serde(rename = "MFQuantity")]
    pub mf_quantity: MFQuantity,
    pub osi_key: String,
    pub mf_transaction: Option<MFTransaction>,
    pub reserve_order: bool,
    pub reserve_quantity: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct MFQuantity {
    pub cash: f64,
    pub margin: f64,
    pub cusip: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Lots {
    #[serde(rename = "Lot")]
    pub lot: Vec<Lot>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Lot {
    pub id: i64,
    pub size: f64,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Currency {
    #[serde(rename = "USD")]
    Usd,
    #[serde(rename = "EUR")]
    Eur,
    #[serde(rename = "GBP")]
    Gbp,
    #[serde(rename = "HKD")]
    Hkd,
    #[serde(rename = "JPY")]
    Jpy,
    #[serde(rename = "CAD")]
    Cad,
}

impl Default for Currency {
    fn default() -> Self {
        Currency::Usd
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum QuantityType {
    #[serde(rename = "QUANTITY")]
    Quantity,
    #[serde(rename = "DOLLAR")]
    Dollar,
    #[serde(rename = "ALL_I_OWN")]
    AllIOwn,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum MFTransaction {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum OrderAction {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
    #[serde(rename = "BUY_TO_COVER")]
    BuyToCover,
    #[serde(rename = "SELL_SHORT")]
    SellShort,
    #[serde(rename = "BUY_OPEN")]
    BuyOpen,
    #[serde(rename = "BUY_CLOSE")]
    BuyClose,
    #[serde(rename = "SELL_OPEN")]
    SellOpen,
    #[serde(rename = "SELL_CLOSE")]
    SellClose,
    #[serde(rename = "EXCHANGE")]
    Exchange,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum EventName {
    #[serde(rename = "UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "ORDER_PLACED")]
    OrderPlaced,
    #[serde(rename = "SENT_TO_CMS")]
    SentToCms,
    #[serde(rename = "SENT_TO_MARKET")]
    SentToMarket,
    #[serde(rename = "MARKET_SENT_ACKNOWLEDGED")]
    MarketSentAcknowledged,
    #[serde(rename = "CANCEL_REQUESTED")]
    CancelRequested,
    #[serde(rename = "ORDER_MODIFIED")]
    OrderModified,
    #[serde(rename = "ORDER_SENT_TO_BROKER_REVIEW")]
    OrderSentToBrokerReview,
    #[serde(rename = "SYSTEM_REJECTED")]
    SystemRejected,
    #[serde(rename = "ORDER_REJECTED")]
    OrderRejected,
    #[serde(rename = "ORDER_CANCELLED")]
    OrderCancelled,
    #[serde(rename = "CANCEL_REJECTED")]
    CancelRejected,
    #[serde(rename = "ORDER_EXPIRED")]
    OrderExpired,
    #[serde(rename = "ORDER_EXECUTED")]
    OrderExecuted,
    #[serde(rename = "ORDER_ADJUSTED")]
    OrderAdjusted,
    #[serde(rename = "ORDER_REVERSED")]
    OrderReversed,
    #[serde(rename = "REVERSE_CANCELLATION")]
    ReverseCancellation,
    #[serde(rename = "REVERSE_EXPIRATION")]
    ReverseExpiration,
    #[serde(rename = "OPTION_POSITION_ASSIGNED")]
    OptionPositionAssigned,
    #[serde(rename = "OPEN_ORDER_ADJUSTED")]
    OpenOrderAdjusted,
    #[serde(rename = "CA_CANCELLED")]
    CaCancelled,
    #[serde(rename = "CA_BOOKED")]
    CaBooked,
    #[serde(rename = "IPO_ALLOCATED")]
    IpoAllocated,
    #[serde(rename = "DONE_TRADE_EXECUTED")]
    DoneTradeExecuted,
    #[serde(rename = "REJECTION_REVERSAL")]
    RejectionReversal,
}

impl Default for EventName {
    fn default() -> Self {
        EventName::Unspecified
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Events {
    #[serde(rename = "Events")]
    pub event: Vec<Event>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Messages {
    #[serde(rename = "Message")]
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum OrderStatus {
    #[serde(rename = "OPEN")]
    Open,
    #[serde(rename = "EXECUTED")]
    Executed,
    #[serde(rename = "CANCELLED")]
    Cancelled,
    #[serde(rename = "INDIVIDUAL_FILLS")]
    IndividualFills,
    #[serde(rename = "CANCEL_REQUESTED")]
    CancelRequested,
    #[serde(rename = "EXPIRED")]
    Expired,
    #[serde(rename = "REJECTED")]
    Rejected,
    #[serde(rename = "PARTIAL")]
    Partial,
    #[serde(rename = "DO_NOT_EXERCISE")]
    DoNotExercise,
    #[serde(rename = "DONE_TRADE_EXECUTED")]
    DoneTradeExecuted,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum TransactionType {
    #[serde(rename = "ATNM")]
    Atnm,
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
    #[serde(rename = "SELL_SHORT")]
    SellShort,
    #[serde(rename = "BUY_TO_COVER")]
    BuyToCover,
    #[serde(rename = "MF_EXCHANGE")]
    MfExchange,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum OrderTerm {
    #[serde(rename = "GOOD_UNTIL_CANCEL")]
    GoodUntilCancel,
    #[serde(rename = "GOOD_FOR_DAY")]
    GoodForDay,
    #[serde(rename = "GOOD_TILL_DATE")]
    GoodTillDate,
    #[serde(rename = "IMMEDIATE_OR_CANCEL")]
    ImmediateOrCancel,
    #[serde(rename = "FILL_OR_KILL")]
    FillOrKill,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum OrderType {
    #[serde(rename = "EQ")]
    Eq,
    #[serde(rename = "OPTN")]
    Optn,
    #[serde(rename = "SPREADS")]
    Spreads,
    #[serde(rename = "BUY_WRITES")]
    BuyWrites,
    #[serde(rename = "BUTTERFLY")]
    Butterfly,
    #[serde(rename = "IRON_BUTTERFLY")]
    IronButterfly,
    #[serde(rename = "CONDOR")]
    Condor,
    #[serde(rename = "IRON_CONDOR")]
    IronCondor,
    #[serde(rename = "MF")]
    Mf,
    #[serde(rename = "MMF")]
    Mmf,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum PriceType {
    #[serde(rename = "MARKET")]
    Market,
    #[serde(rename = "LIMIT")]
    Limit,
    #[serde(rename = "STOP")]
    Stop,
    #[serde(rename = "STOP_LIMIT")]
    StopLimit,
    #[serde(rename = "TRAILING_STOP_CNST_BY_LOWER_TRIGGER")]
    TrailingStopCnstByLowerTrigger,
    #[serde(rename = "UPPER_TRIGGER_BY_TRAILING_STOP_CNST")]
    UpperTriggerByTrailingStopCnst,
    #[serde(rename = "TRAILING_STOP_PRCT_BY_LOWER_TRIGGER")]
    TrailingStopPrctByLowerTrigger,
    #[serde(rename = "UPPER_TRIGGER_BY_TRAILING_STOP_PRCT")]
    UpperTriggerByTrailingStopPrct,
    #[serde(rename = "TRAILING_STOP_CNST")]
    TrailingStopCnst,
    #[serde(rename = "TRAILING_STOP_PRCT")]
    TrailingStopPrct,
    #[serde(rename = "HIDDEN_STOP")]
    HiddenStop,
    #[serde(rename = "HIDDEN_STOP_BY_LOWER_TRIGGER")]
    HiddenStopByLowerTrigger,
    #[serde(rename = "UPPER_TRIGGER_BY_HIDDEN_STOP")]
    UpperTriggerByHiddenStop,
    #[serde(rename = "NET_DEBIT")]
    NetDebit,
    #[serde(rename = "NET_CREDIT")]
    NetCredit,
    #[serde(rename = "NET_EVEN")]
    NetEven,
    #[serde(rename = "MARKET_ON_OPEN")]
    MarketOnOpen,
    #[serde(rename = "MARKET_ON_CLOSE")]
    MarketOnClose,
    #[serde(rename = "LIMIT_ON_OPEN")]
    LimitOnOpen,
    #[serde(rename = "LIMIT_ON_CLOSE")]
    LimitOnClose,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum OffsetType {
    #[serde(rename = "TRAILING_STOP_CNST")]
    TrailingStopCnst,
    #[serde(rename = "TRAILING_STOP_PRCT")]
    TrailingStopPrct,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum RoutingDestination {
    #[serde(rename = "AUTO")]
    Auto,
    #[serde(rename = "AMEX")]
    Amex,
    #[serde(rename = "BOX")]
    Box,
    #[serde(rename = "CBOE")]
    Cboe,
    #[serde(rename = "ISE")]
    Ise,
    #[serde(rename = "NOM")]
    Nom,
    #[serde(rename = "NYSE")]
    Nyse,
    #[serde(rename = "PHX")]
    Phx,
}

impl Default for RoutingDestination {
    fn default() -> Self {
        RoutingDestination::Auto
    }
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum ConditionType {
    #[serde(rename = "CONTINGENT_GTE")]
    Gte,
    #[serde(rename = "CONTINGENT_LTE")]
    Lte,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum ConditionFollowPrice {
    #[serde(rename = "ASK")]
    Ask,
    #[serde(rename = "BID")]
    Bid,
    #[serde(rename = "LAST")]
    Last,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum PositionQuantity {
    #[serde(rename = "ENTIRE_POSITION")]
    EntirePosition,
    #[serde(rename = "CASH")]
    Cash,
    #[serde(rename = "MARGIN")]
    Margin,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum EgQual {
    #[serde(rename = "EG_QUAL_UNSPECIFIED")]
    EgQualUnspecified,
    #[serde(rename = "EG_QUAL_QUALIFIED")]
    EgQualQualified,
    #[serde(rename = "EG_QUAL_NOT_IN_FORCE")]
    EgQualNotInForce,
    #[serde(rename = "EG_QUAL_NOT_A_MARKET_ORDER")]
    EgQualNotAMarketOrder,
    #[serde(rename = "EG_QUAL_NOT_AN_ELIGIBLE_SECURITY")]
    EgQualNotAnEligibleSecurity,
    #[serde(rename = "EG_QUAL_INVALID_ORDER_TYPE")]
    EgQualInvalidOrderType,
    #[serde(rename = "EG_QUAL_SIZE_NOT_QUALIFIED")]
    EgQualSizeNotQualified,
    #[serde(rename = "EG_QUAL_OUTSIDE_GUARANTEED_PERIOD")]
    EgQualOutsideGuaranteedPeriod,
    #[serde(rename = "EG_QUAL_INELIGIBLE_GATEWAY")]
    EgQualIneligibleGateway,
    #[serde(rename = "EG_QUAL_INELIGIBLE_DUE_TO_IPO")]
    EgQualIneligibleDueToIpo,
    #[serde(rename = "EG_QUAL_INELIGIBLE_DUE_TO_SELF_DIRECTED")]
    EgQualIneligibleDueToSelfDirected,
    #[serde(rename = "EG_QUAL_INELIGIBLE_DUE_TO_CHANGEORDER")]
    EgQualIneligibleDueToChangeorder,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum ReInvestOption {
    #[serde(rename = "REINVEST")]
    Reinvest,
    #[serde(rename = "DEPOSIT")]
    Deposit,
    #[serde(rename = "CURRENT_HOLDING")]
    CurrentHolding,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumString)]
pub enum MarginLevelCd {
    #[serde(rename = "UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "MARGIN_TRADING_NOT_ALLOWED")]
    MarginTradingNotAllowed,
    #[serde(rename = "MARGIN_TRADING_ALLOWED")]
    MarginTradingAllowed,
    #[serde(rename = "MARGIN_TRADING_ALLOWED_ON_OPTIONS")]
    MarginTradingAllowedOnOptions,
    #[serde(rename = "MARGIN_TRADING_ALLOWED_ON_PM")]
    MarginTradingAllowedOnPm,
}

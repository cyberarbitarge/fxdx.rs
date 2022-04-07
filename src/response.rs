use anyhow::Result;
use bigdecimal::BigDecimal;
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::cmp::PartialEq;

pub trait Success {
    fn is_success(&self) -> bool;
}

impl Success for i32 {
    fn is_success(&self) -> bool {
        *self == 200
    }
}

#[derive(Debug, PartialEq, Deserialize_repr)]
#[repr(u8)]
pub enum Direction {
    Ask = 0,
    Bid = 1,
}

#[derive(Debug, Deserialize)]
pub struct NonceResponse {
    pub code: i32,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub code: i32,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PendingOrderResponse {
    pub code: i32,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchPendingOrdersResponse {
    pub code: i32,
    pub data: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderResponse {
    pub code: i32,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchCancelOrdersResponse {
    pub code: i32,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Trade {
    pub base: i32,
    pub quote: i32,
    pub ask_or_bid: Direction,
    pub price: BigDecimal,
    pub amount: BigDecimal,
    pub quote_amount: BigDecimal,
    pub quote_fee: BigDecimal,
    pub base_fee: BigDecimal,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct QueryOrder {
    pub symbol: String,
    pub order_id: String,
    pub order_type: crate::request::OrderType,
    pub direction: Direction,
    pub amount: BigDecimal,
    pub price: BigDecimal,
    pub filled_base: BigDecimal,
    pub filled_quote: BigDecimal,
    pub avg_price: BigDecimal,
    pub status: crate::request::OrderStatus,
    pub trades: Vec<Trade>,
}

#[derive(Debug, Deserialize)]
pub struct QueryByIdResponse {
    pub code: i32,
    pub data: Option<QueryOrder>,
}

#[derive(Debug, Deserialize)]
pub struct QueryByPageResponse {
    pub code: i32,
    pub data: Option<Vec<QueryOrder>>,
}

#[derive(Debug, Deserialize)]
pub struct Balance {
    pub code: i32,
    pub name: String,
    pub available: BigDecimal,
    pub frozen: BigDecimal,
}

#[derive(Debug, Deserialize)]
pub struct BalancesResposne {
    pub code: i32,
    pub data: Option<Balance>,
}

#[derive(Debug, Deserialize)]
pub struct Depth {
    pub depth: i32,
    pub bids: Vec<Vec<BigDecimal>>,
    pub asks: Vec<Vec<BigDecimal>>,
}

#[derive(Debug, Deserialize)]
pub struct DepthResponse {
    pub code: i32,
    pub data: Option<Depth>,
}

#[derive(Debug, Deserialize)]
pub struct Kline {
    pub id: i64,
    pub open: BigDecimal,
    pub close: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub vol: BigDecimal,
}

#[derive(Debug, Deserialize)]
pub struct KlineResponse {
    pub code: i32,
    pub data: Option<Vec<Kline>>,
}

#[derive(Debug, Deserialize)]
pub struct Symbol {
    pub base: i32,
    pub quote: i32,
    pub base_name: String,
    pub quote_name: String,
    pub base_scale: i32,
    pub quote_scale: i32,
    pub taker_fee: BigDecimal,
    pub make_fee: BigDecimal,
    pub min_amount: BigDecimal,
    pub min_vol: BigDecimal,
    pub enable_marker_order: bool,
}

#[derive(Debug, Deserialize)]
pub struct SymbolsResponse {
    pub code: i32,
    pub data: Option<Vec<Symbol>>,
}

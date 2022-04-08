use bigdecimal::BigDecimal;
use serde::ser::Serializer;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use std::cmp::PartialEq;
use std::marker::PhantomData;

pub trait Prefix {
    fn prefix() -> &'static str;
}

pub struct PrivPub;
pub struct Sr25519;

impl Prefix for PrivPub {
    #[inline]
    fn prefix() -> &'static str {
        "/maker"
    }
}

impl Prefix for Sr25519 {
    #[inline]
    fn prefix() -> &'static str {
        "/api"
    }
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq)]
#[repr(u8)]
pub enum OrderType {
    Ask = 0,
    Bid = 1,
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq)]
#[repr(u8)]
pub enum OrderStatus {
    Undeal = 1,
    Cancel = 2,
    Dealed = 3,
    PartialDealed = 4,
}

#[derive(Debug, Copy, Clone)]
pub enum Scale {
    Minute,
    Minute5,
    Minute15,
    Minute30,
    Hour,
    Hour4,
    Day,
    Week,
}

impl Serialize for Scale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Scale::Minute => serializer.serialize_str("MINUTE"),
            Scale::Minute5 => serializer.serialize_str("MINUTE_5"),
            Scale::Minute15 => serializer.serialize_str("MINUTE_15"),
            Scale::Minute30 => serializer.serialize_str("MINUTE_30"),
            Scale::Hour => serializer.serialize_str("HOUR"),
            Scale::Hour4 => serializer.serialize_str("HOUR4"),
            Scale::Day => serializer.serialize_str("DAY"),
            Scale::Week => serializer.serialize_str("WEEK"),
        }
    }
}

impl std::string::ToString for Scale {
    fn to_string(&self) -> String {
        match self {
            Scale::Minute => String::from("MINUTE"),
            Scale::Minute5 => String::from("MINUTE_5"),
            Scale::Minute15 => String::from("MINUTE_15"),
            Scale::Minute30 => String::from("MINUTE_30"),
            Scale::Hour => String::from("HOUR"),
            Scale::Hour4 => String::from("HOUR4"),
            Scale::Day => String::from("DAY"),
            Scale::Week => String::from("WEEK"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum Request {
    Nonce,
    Token {
        nonce: String,
        pubkey: String,
        signature: String,
    },
    PendingOrder {
        r#type: String,
        symbol: String,
        price: BigDecimal,
        amount: BigDecimal,
    },
    BatchPendingOrders(Vec<Self>),
    CancelOrder {
        symbol: String,
        order_id: String,
    },
    BatchCancelOrders {
        symbol: String,
        order_ids: Vec<String>,
    },
    OrderById {
        symbol: String,
        order_id: String,
    },
    OrderByPage {
        symbol: String,
        page: i32,
        size: i32,
        pending: bool,
    },
    Balances,
    Depth {
        symbol: String,
    },
    Kline {
        symbol: String,
        scale: Scale,
    },
    Symbols,
}

// FIXME: for more effective profermance we have to change the implemation into generic type
impl Request {
    pub fn uri<P: Prefix>(&self) -> String {
        match self {
            Request::Nonce => format!("/maker/nonce"),
            Request::Token { .. } => format!("/{}/token", P::prefix()),
            Request::PendingOrder { .. } => format!("/{}/order", P::prefix()),
            Request::BatchPendingOrders { .. } => format!("/{}/orders", P::prefix()),
            Request::CancelOrder { symbol, order_id } => {
                format!("/{}/order/{}/{}", P::prefix(), symbol, order_id)
            }
            Request::BatchCancelOrders { symbol, order_ids } => {
                format!("/{}/order/{}/{}", P::prefix(), symbol, order_ids.join("|"))
            }
            Request::OrderById { symbol, order_id } => {
                format!("/{}/order/{}/{}", P::prefix(), symbol, order_id)
            }
            Request::OrderByPage {
                symbol,
                page,
                size,
                pending,
            } => format!(
                "/{}/orders/{}/{}/{}/{}",
                P::prefix(),
                symbol,
                page,
                size,
                pending
            ),
            Request::Balances => format!("/{}/balances", P::prefix()),
            Request::Depth { symbol } => format!("/{}/depth/{}", P::prefix(), symbol),
            Request::Kline { symbol, scale } => {
                format!("/{}/kline/{}/{}", P::prefix(), symbol, scale.to_string())
            }
            Request::Symbols => format!("/{}/symbols", P::prefix()),
        }
    }
    pub fn method(&self) -> reqwest::Method {
        match self {
            Request::Nonce => reqwest::Method::POST,
            Request::Token { .. } => reqwest::Method::POST,
            Request::PendingOrder { .. } => reqwest::Method::POST,
            Request::BatchPendingOrders { .. } => reqwest::Method::POST,
            Request::CancelOrder { .. } => reqwest::Method::DELETE,
            Request::BatchCancelOrders { .. } => reqwest::Method::DELETE,
            Request::OrderById { .. } => reqwest::Method::GET,
            Request::OrderByPage { .. } => reqwest::Method::GET,
            Request::Balances => reqwest::Method::GET,
            Request::Depth { .. } => reqwest::Method::GET,
            Request::Kline { .. } => reqwest::Method::GET,
            Request::Symbols => reqwest::Method::GET,
        }
    }

    pub fn formalize(&self) -> Option<String> {
        match self {
            Request::PendingOrder {
                r#type,
                symbol,
                price,
                amount,
            } => Some(format!("{},{},{},{}", amount, price, symbol, r#type)),
            Request::BatchPendingOrders(orders) => Some(format!(
                "{}",
                orders
                    .iter()
                    .map(|o| o.formalize().unwrap()) // have to use the Request::PendingOrder varints else panic
                    .collect::<Vec<String>>()
                    .join(",")
            )),
            Request::CancelOrder { symbol, order_id } => Some(format!("{},{}", order_id, symbol)),
            Request::BatchCancelOrders { symbol, order_ids } => {
                Some(format!("{},{}", order_ids.join("|"), symbol))
            }
            Request::OrderById { symbol, order_id } => Some(format!("{},{}", order_id, symbol)),
            Request::OrderByPage {
                symbol,
                page,
                size,
                pending,
            } => Some(format!("{},{},{},{}", page, pending, size, symbol)),
            Request::Depth { symbol } => Some(format!("{}", symbol)),
            Request::Kline { symbol, scale } => Some(format!("{},{}", scale.to_string(), symbol)),
            _ => None,
        }
    }

    pub fn payload(&self) -> anyhow::Result<Option<String>> {
        Ok(match self {
           Request::Token{..} | Request::PendingOrder{..} => Some(serde_json::to_string(self)?),
           Request::BatchPendingOrders(orders) => Some(serde_json::to_string(self)?),
           _ => Ok(None),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockClient<T> {
        _marker: std::marker::PhantomData<T>,
    }

    impl<T> MockClient<T>
    where
        T: Prefix,
    {
        fn new() -> Self {
            MockClient {
                _marker: Default::default(),
            }
        }

        fn prefix(&self) -> &'static str {
            T::prefix()
        }
    }

    #[test]
    fn test_generic_type() {
        let client = MockClient::<PrivPub>::new();
        assert_eq!(client.prefix(), "/maker")
    }
}

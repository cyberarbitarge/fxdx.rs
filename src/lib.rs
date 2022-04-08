mod request;
mod response;

use anyhow::Result;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Signer as OpensslSigner;
use reqwest::header::HeaderValue;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid request {0}")]
    InvalidRequest(String),

    #[error("Invalid wait to gernerate signature {0:?}")]
    InvalidSignature(request::Request),
}

struct Signer {
    secret_key: String,
}

impl Signer {
    pub fn new(secret: String) -> Self {
        Signer { secret_key: secret }
    }

    pub fn sign(&self, formalized: String) -> Result<Vec<u8>> {
        let secret = PKey::hmac(self.secret_key.as_bytes())?;
        let mut signer = OpensslSigner::new(MessageDigest::sha1(), &secret)?;
        signer.update(formalized.as_bytes())?;
        Ok(signer.sign_to_vec()?)
    }
}

pub struct FxdxClient<P> {
    client: reqwest::Client,
    endpoint: String,
    address: String,
    signer: Signer,
    _marker: std::marker::PhantomData<P>,
}

impl<P> FxdxClient<P>
where
    P: request::Prefix,
{
    async fn send(&self, req: request::Request) -> Result<reqwest::Response> {
        let mut builder = self
            .client
            .request(req.method(), format!("{}{}", self.endpoint, req.uri::<P>()));
        let timestamp = {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
            now.as_secs().to_string()
        };
        let mut signature = format!(
            "{},{},{}",
            self.signer.secret_key,
            &timestamp,
            req.uri::<P>()
        );
        if let Some(suffix) = req.formalize() {
            signature = format!("{},{}", signature, suffix);
        }
        if let Some(payload) = req.payload()? {
            builder = builder.body(payload);
        }
        Ok(builder
            .header("X-Timestamp", HeaderValue::from_str(&timestamp)?)
            .header("X-Address", HeaderValue::from_str(&self.address)?)
            .header("X-Signature", HeaderValue::from_str(&signature)?)
            .send()
            .await?)
    }

    /// fresh the inner signer using sr25519
    pub async fn fresh(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// send a pending order to fxdx
    pub async fn pending_order(
        &self,
        req: request::Request,
    ) -> Result<response::PendingOrderResponse> {
        Ok(self
            .send(req)
            .await?
            .json::<response::PendingOrderResponse>()
            .await?)
    }

    /// batch pending orders
    pub async fn batch_pending_orders(
        &self,
        req: request::Request,
    ) -> Result<response::BatchPendingOrdersResponse> {
        Ok(self
            .send(req)
            .await?
            .json::<response::BatchPendingOrdersResponse>()
            .await?)
    }

    pub async fn cancel_order(
        &self,
        req: request::Request,
    ) -> Result<response::CancelOrderResponse> {
        Ok(self
            .send(req)
            .await?
            .json::<response::CancelOrderResponse>()
            .await?)
    }

    pub async fn batch_cancel_orders(
        &self,
        req: request::Request,
    ) -> Result<response::BatchCancelOrdersResponse> {
        Ok(self
            .send(req)
            .await?
            .json::<response::BatchCancelOrdersResponse>()
            .await?)
    }

    pub async fn query_order_by_id(
        &self,
        req: request::Request,
    ) -> Result<response::QueryByIdResponse> {
        Ok(self
            .send(req)
            .await?
            .json::<response::QueryByIdResponse>()
            .await?)
    }

    pub async fn query_orders_by_page(
        &self,
        req: request::Request,
    ) -> Result<response::QueryByPageResponse> {
        Ok(self
            .send(req)
            .await?
            .json::<response::QueryByPageResponse>()
            .await?)
    }

    pub async fn query_account_balance(
        &self,
        req: request::Request,
    ) -> Result<response::BalancesResposne> {
        Ok(self
            .send(req)
            .await?
            .json::<response::BalancesResposne>()
            .await?)
    }

    pub async fn query_depth(&self, req: request::Request) -> Result<response::DepthResponse> {
        Ok(self
            .send(req)
            .await?
            .json::<response::DepthResponse>()
            .await?)
    }

    pub async fn query_kline(&self, req: request::Request) -> Result<response::KlineResponse> {
        Ok(self
            .send(req)
            .await?
            .json::<response::KlineResponse>()
            .await?)
    }

    pub async fn query_symbols(&self, req: request::Request) -> Result<response::SymbolsResponse> {
        Ok(self
            .send(req)
            .await?
            .json::<response::SymbolsResponse>()
            .await?)
    }
}

#[derive(Default)]
pub struct FxdxBuilder<P> {
    endpoint: String,
    secret_key: String,
    address: String,
    is_sr25519: bool,
    _marker: std::marker::PhantomData<P>,
}

impl<P> FxdxBuilder<P>
where
    P: request::Prefix,
{
    pub fn endpoint(endpoint: String) -> Self {
        FxdxBuilder {
            endpoint,
            secret_key: Default::default(),
            address: Default::default(),
            is_sr25519: false,
            _marker: Default::default(),
        }
    }

    pub fn sr25519(mut self, address: String, private_key: String) -> Self {
        self.address = address;
        self.secret_key = private_key; // FIXME: use the sr25519 handshake
        self.is_sr25519 = true;
        self
    }

    pub fn secret(mut self, secret_key: String) -> Self {
        if self.is_sr25519 {
            panic!("could not set registered secret in sr25519 mode");
        }
        self.secret_key = secret_key;
        self
    }

    pub async fn build(mut self) -> Result<FxdxClient<P>> {
        if self.is_sr25519 {
            let client = reqwest::Client::new();
            // if sr25519 handshake else panic and set the default headers
            let nonce = client
                .post(format!("{}/maker/nonce", &self.endpoint))
                .send()
                .await?
                .json::<response::NonceResponse>()
                .await?;
            // TODO: impl the Schnorrkel signature for this mode
            unimplemented!()
        } else {
            let builder = reqwest::Client::builder();
            Ok(FxdxClient {
                client: builder.build()?,
                endpoint: self.endpoint,
                address: self.address,
                signer: Signer::new(self.secret_key),
                _marker: Default::default(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

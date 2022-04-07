mod request;
mod response;

use anyhow::bail;
use anyhow::Result;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Signer as OpensslSigner;
use reqwest::header::HeaderValue;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid request {0}")]
    InvalidRequest(String),
}

struct Signer {
    secret_key: Vec<u8>,
}

impl Signer {
    pub fn new(secret: Vec<u8>) -> Self {
        Signer { secret_key: secret }
    }

    pub fn sign(&self, formalized: String) -> Result<Vec<u8>> {
        let secret = PKey::hmac(&self.secret_key)?;
        let mut signer = OpensslSigner::new(MessageDigest::sha1(), &secret)?;
        signer.update(formalized.as_bytes())?;
        Ok(signer.sign_to_vec()?)
    }
}

// TODO: use the const generics when the String generics const stable
pub struct FxdxClient<P> {
    client: reqwest::Client,
    endpoint: String,
    address: String,
    token: Option<String>,
    signer: Option<Signer>,
    _marker: std::marker::PhantomData<P>,
}

impl<P> FxdxClient<P>
where
    P: request::Prefix,
{
    async fn send(&self, builder: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let timestamp = {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
            now.as_secs().to_string()
        };
        Ok(builder
            .header("X-Timestamp", HeaderValue::from_str(&timestamp)?)
            .header("X-Address", HeaderValue::from_str(&self.address)?)
            .send()
            .await?)
    }

    /// send a pending order to fxdx
    pub async fn pending_order(
        &self,
        req: request::Request,
    ) -> Result<response::PendingOrderResponse> {
        if let request::Request::PendingOrder { .. } = &req {
            let builder = self.client.request(
                req.method(),
                format!("{}{}", self.endpoint, &req.uri::<P>()),
            );
            if let Some(ref token) = self.token {
                // TODO: for the logic of this using sr25519
                unimplemented!()
            } else {
                return Ok(self
                    .send({
                        if let Some(formalized) = req.formalize() {
                            builder.header(
                                "X-Signature",
                                HeaderValue::from_bytes(&self.signer.unwrap().sign(formalized)?)?,
                            )
                        } else {
                            builder
                        }
                    })
                    .await?
                    .json::<response::PendingOrderResponse>()
                    .await?);
            }
        } else {
            Err(bail!(Error::InvalidRequest(format!("{:?}", req))))
        }
    }
}

#[derive(Default)]
pub struct FxdxBuilder<P> {
    endpoint: String,
    secret_key: Vec<u8>,
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

    pub fn sr25519(mut self, address: String, private_key: Vec<u8>) -> Self {
        self.address = address;
        self.secret_key = private_key;
        self.is_sr25519 = true;
        self
    }

    pub fn secret(mut self, secret_key: Vec<u8>) -> Self {
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
                token: None,
                signer: Some(Signer::new(self.secret_key)),
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

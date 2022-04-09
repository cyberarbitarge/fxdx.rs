# fxdx.rs

fusotao fxdx rust connector for maker 

Sample example 

```rust 
let client = FxdxBuilder::<fxdx_rs::request::PrivPub>::endpoint(String::from("https://test-api.fxdx.finance"))
                                                        .address(String::from("your polkadot.js address"))
                                                        .secret(String::from("your maker key"))
                                                        .build()
                                                        .await?;

let symbols = client.query_symbols(fxdx_rs::request::Symbols).await?;
```

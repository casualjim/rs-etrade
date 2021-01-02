# Rust etrade

Wraps the etrade API and implements the required oauth1 flow.

## State storage

The default feature for the crate includes a thread safe in-memory store for the oauth tokens.
There is an optional feature `secretservice` which will the keychain on linux to store the token information.

## Usage

```rust
use anyhow::{anyhow, Result};
use etrade::orders::{ListOrdersRequest, OrderStatus, TransactionType};
use etrade::KeychainStore;
use etrade::{self, SortOrder};
use etrade::{accounts, MarketSession, SecurityType};
use accounts::BalanceRequest;

#[tokio::main]
async fn main() -> Result<()> {
  let mode: etrade::Mode = etrade::Mode::Live;
  let session = Arc::new(etrade::Session::new(mode, KeychainStore));
  let accounts = etrade::accounts::Api::new(session.clone());

  let msg1 = "Consumer key:\n";
  io::stderr().write_all(msg1.as_bytes()).await?;

  let mut consumer_token = String::new();
  io::BufReader::new(io::stdin()).read_line(&mut consumer_token).await?;

  let msg2 = "Consumer secret:\n";
  io::stderr().write_all(msg2.as_bytes()).await?;

  let mut consumer_secret = String::new();
  io::BufReader::new(io::stdin()).read_line(&mut consumer_secret).await?;

  session
    .initialize(consumer_token.trim().to_string(), consumer_secret.trim().to_string())
    .await?;
  println!("updated the {} consumer token and key", mode);

  let account_list = accounts.list(etrade::OOB).await?;

  for account in &account_list {
    let balance = accounts
        .balance(
          &account.account_id_key,
          BalanceRequest {
            real_time_nav: if real_time { Some(real_time) } else { None },
            ..Default::default()
          },
          oob,
        )
        .await?;
    println!("{:?}", balance);
  }

  Ok(())
}
```
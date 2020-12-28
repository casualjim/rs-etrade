#[macro_use]
extern crate log;

mod credentials;
mod etrade;

use colored_json::to_colored_json_auto;

use anyhow::{anyhow, bail, Result};
// use credentials::SecretServiceStore;
// use etrade::Store;
// use etrade::{Account, AuthenticatedClient};

const NAMESPACE: &str = "etradesandbox";
const API_KEY: &str = "apikey";
const SECRET_KEY: &str = "secret";
const ACCESS_TOKEN_KEY: &str = "access_token_key";
const ACCESS_TOKEN_SECRET: &str = "access_token_secret";
const REQUEST_TOKEN_KEY: &str = "request_token_key";
const REQUEST_TOKEN_SECRET: &str = "request_token_secret";
// const

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "info,etrade=debug,playground=debug");
    pretty_env_logger::init();
    // let mut store = SecretServiceStore::new()?;

    // // prepare authorization info
    // let consumer_key = store
    //     .get(NAMESPACE, API_KEY)
    //     .and_then(|r| r.ok_or(anyhow!("secret {}@{} not found.", API_KEY, NAMESPACE)))?;
    // let consumer_secret = store
    //     .get(NAMESPACE, SECRET_KEY)
    //     .and_then(|r| r.ok_or(anyhow!("secret {}@{} not found.", SECRET_KEY, NAMESPACE)))?;

    // let request_token_key = store.get(NAMESPACE, REQUEST_TOKEN_KEY)?;
    // let request_token_secret = store.get(NAMESPACE, REQUEST_TOKEN_SECRET)?;
    // let access_token_key = store.get(NAMESPACE, ACCESS_TOKEN_KEY)?;
    // let access_token_secret = store.get(NAMESPACE, ACCESS_TOKEN_SECRET)?;

    // let mut client = match (access_token_key, access_token_secret) {
    //     (Some(key), Some(secret)) => etrade::AuthenticatedClient::sandbox(
    //         consumer_key.clone(),
    //         consumer_secret.clone(),
    //         request_token_key,
    //         request_token_secret,
    //         key,
    //         secret,
    //     ),
    //     _ => authenticate_client(&mut store, consumer_key.clone(), consumer_secret.clone()).await?,
    // };

    // let mut accounts: Option<Vec<Account>> = None;
    // let mut cnt: usize = 0;
    // loop {
    //     if cnt > 2 {
    //         bail!("exhausted all retries for refreshing the credentials");
    //     }
    //     cnt += 1;
    //     match client.account_list().await {
    //         Ok(accts) => {
    //             accounts.replace(accts);
    //             break;
    //         }
    //         Err(etrade::Error::Reqwest(e))
    //         | Err(etrade::Error::Oauth(reqwest_oauth1::Error::Reqwest(e))) => {
    //             if let Some(sc) = e.status() {
    //                 if sc != 401 {
    //                     bail!("{}", e);
    //                 }
    //             } else {
    //                 bail!("{}", e);
    //             }

    //             info!("refreshing the authentication");
    //             client = renew_access_token(
    //                 &mut store,
    //                 client,
    //                 consumer_key.clone(),
    //                 consumer_secret.clone(),
    //             )
    //             .await?;
    //         }
    //         Err(e) => bail!("{}", e),
    //     }
    // }

    // for account in accounts.unwrap_or_default() {
    //     let balance = client.account_balance(&account, false).await?;
    //     println!(
    //         "{}",
    //         to_colored_json_auto(&serde_json::to_value(&balance)?)?
    //     );
    // }

    Ok(())
}

// async fn renew_access_token<'a>(
//     store: &mut impl Store,
//     ucl: AuthenticatedClient,
//     consumer_key: SecUtf8,
//     consumer_secret: SecUtf8,
// ) -> Result<etrade::AuthenticatedClient> {
//     match ucl.renew_access_token().await {
//         Ok(access_token) => {
//             store.put(NAMESPACE, ACCESS_TOKEN_KEY, access_token.key.clone())?;
//             store.put(NAMESPACE, ACCESS_TOKEN_SECRET, access_token.secret.clone())?;
//             Ok(ucl.with_access_token(access_token))
//         }
//         Err(etrade::Error::Reqwest(e))
//         | Err(etrade::Error::Oauth(reqwest_oauth1::Error::Reqwest(e))) => {
//             if let Some(sc) = e.status() {
//                 if sc == 401 {
//                     return authenticate_client(store, consumer_key, consumer_secret).await;
//                 }
//             }
//             Err(e.into())
//         }
//         Err(e) => Err(e.into()),
//     }
// }

// async fn authenticate_client(
//     store: &mut impl Store,
//     consumer_key: SecUtf8,
//     consumer_secret: SecUtf8,
// ) -> Result<etrade::AuthenticatedClient> {
//     let ss = SecretService::new(EncryptionType::Dh).expect("failed to initialize secret service");
//     let _collection = ss
//         .get_default_collection()
//         .expect("failed to get default collection");
//     let ucl = etrade::Client::new(consumer_key.clone(), consumer_secret.clone());

//     let request_token = (&ucl).request_token().await?;
//     println!(
//         "please visit: {}",
//         (&ucl).verifier_url(&request_token).await?,
//     );
//     println!("input pin: ");

//     let mut user_input = String::new();
//     io::stdin().read_line(&mut user_input)?;
//     let pin = user_input.trim();

//     let access_token = ucl.access_token(request_token.clone(), pin).await?;
//     store.put(NAMESPACE, ACCESS_TOKEN_KEY, access_token.key.clone())?;
//     store.put(NAMESPACE, ACCESS_TOKEN_SECRET, access_token.secret.clone())?;
//     store.put(NAMESPACE, REQUEST_TOKEN_KEY, request_token.key.clone())?;
//     store.put(
//         NAMESPACE,
//         REQUEST_TOKEN_SECRET,
//         request_token.secret.clone(),
//     )?;

//     Ok(etrade::AuthenticatedClient::sandbox(
//         consumer_key,
//         consumer_secret,
//         Some(request_token.clone().key),
//         Some(request_token.clone().secret),
//         access_token.key,
//         access_token.secret,
//     ))
// }

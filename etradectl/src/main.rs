mod credentials;
use etrade;

use anyhow::Result;
use colored_json::to_colored_json_auto;
use credentials::SecretServiceStore;
use std::io;
// use etrade::{Account, AuthenticatedClient};

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "info,etrade=debug,playground=debug");
    pretty_env_logger::init();
    let store = SecretServiceStore::new()?;

    let session = etrade::Session::new(etrade::Mode::Sandbox, store);
    let accounts = etrade::Accounts::new(session);
    let oob = etrade::OOB;
    let account_list = accounts.account_list(oob).await?;
    for account in account_list.as_slice() {
        // println!(
        //     "{}: {} | {}",
        //     account.account_name, account.account_type, account.account_status
        // );
        let balance = accounts.account_balance(&account, false, oob).await?;
        println!(
            "{}",
            to_colored_json_auto(&serde_json::to_value(&balance)?)?
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    pub fn init() {
        std::env::set_var("RUST_LOG", "debug");
        let _ = pretty_env_logger::try_init();
    }
}

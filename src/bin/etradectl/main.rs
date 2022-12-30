use std::env;

use accounts::BalanceRequest;
use anyhow::{anyhow, Result};
use bat::{Input, PrettyPrinter};
use etrade::orders::{ListOrdersRequest, OrderStatus, TransactionType};
use etrade::KeychainStore;
use etrade::{self, SortOrder};
use etrade::{accounts, MarketSession, SecurityType};
use serde::Serialize;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::io::{self, *};

#[tokio::main]
async fn main() -> Result<()> {
  pretty_env_logger::init();

  let mode: etrade::Mode = etrade::Mode::Live;
  let session = Arc::new(etrade::Session::new(mode, KeychainStore));
  let accounts = etrade::accounts::Api::new(session.clone());
  let orders = etrade::orders::Api::new(session.clone());
  let _transactions = etrade::transactions::Api::new(session.clone());
  let oob = etrade::OOB;

  match Cmd::from_args() {
    Cmd::Init => {
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
    }
    Cmd::Accounts { cmd: AccountCmd::List } => {
      let account_list = accounts.list(oob).await?;
      pretty_print(&account_list)?;
    }
    Cmd::Accounts {
      cmd: AccountCmd::Balance { account_id, real_time },
    } => {
      let balance = accounts
        .balance(
          &account_id,
          BalanceRequest {
            real_time_nav: if real_time { Some(real_time) } else { None },
            ..Default::default()
          },
          oob,
        )
        .await?;
      pretty_print(&balance)?;
    }
    Cmd::Accounts {
      cmd:
        AccountCmd::Portfolio {
          account_id,
          count,
          sort_by,
          sort_order,
          market_session,
          totals_required,
          lots_required,
          view,
        },
    } => {
      let portfolio = accounts
        .portfolio(
          &account_id,
          accounts::PortfolioRequest {
            count,
            sort_by,
            sort_order: Some(sort_order),
            market_session: Some(market_session),
            totals_required,
            lots_required,
            view: Some(view),
          },
          oob,
        )
        .await?;
      pretty_print(&portfolio)?;
    }
    Cmd::Orders {
      cmd:
        OrdersCmd::List {
          account_id,
          marker,
          count,
          status,
          from_date,
          to_date,
          symbol,
          security_type,
          transaction_type,
          market_session,
        },
    } => {
      let results = orders
        .list(
          &account_id,
          ListOrdersRequest {
            marker,
            count,
            status,
            from_date,
            to_date,
            symbol,
            security_type,
            transaction_type,
            market_session,
          },
          oob,
        )
        .await?;
      pretty_print(&results)?;
    }
  };
  Ok(())
}

fn pretty_print<T: Serialize>(data: &T) -> Result<()> {
  // let bytes = serde_json::to_vec_pretty(&data)?;
  let bytes = serde_yaml::to_string(&data)?;

  PrettyPrinter::new()
    .language("yaml")
    .line_numbers(false)
    .grid(false)
    .header(false)
    .input(Input::from_bytes(bytes.as_bytes()))
    .true_color(true)
    .theme(env::var("BAT_THEME").unwrap_or_default())
    .print()
    .map_err(|e| anyhow!("{}", e))?;
  Ok(())
}
#[derive(Debug, StructOpt)]
/// Exposes the E*Trade API methods for the CLI.
///
/// This command mostly serves to manage the oauth1 tokens via the keychain.
enum Cmd {
  Init,
  /// List accounts, balances, transactions and portfolios
  Accounts {
    #[structopt(subcommand)]
    cmd: AccountCmd,
  },
  /// List, place, cancel and preview orders
  Orders {
    #[structopt(subcommand)]
    cmd: OrdersCmd,
  },
}

#[derive(Debug, StructOpt)]
enum OrdersCmd {
  /// List the orders
  List {
    #[structopt(long)]
    /// The account id to fetch the orders for
    account_id: String,
    #[structopt(long)]
    /// Specifies the desired starting point of the set of items to return.
    marker: Option<String>,
    #[structopt(long)]
    /// Number of transactions to return in the response. If not specified, defaults to 25 and maximum count is 100.
    count: Option<usize>,
    #[structopt(long)]
    /// The status
    status: Option<OrderStatus>,
    #[structopt(long)]
    /// The earliest date to include in the date range, formatted as MMDDYYYY. History is available for two years. Both fromDate and toDate should be provided, toDate should be greater than fromDate.
    from_date: Option<String>,
    #[structopt(long)]
    /// The latest date to include in the date range, formatted as MMDDYYYY. Both fromDate and toDate should be provided, toDate should be greater than fromDate.
    to_date: Option<String>,
    #[structopt(long)]
    /// The market symbol for the security being bought or sold. API supports only 25 symbols seperated by delimiter " , ".
    symbol: Option<Vec<String>>,
    #[structopt(long)]
    /// The security type
    security_type: Option<SecurityType>,
    #[structopt(long)]
    /// Type of transaction
    transaction_type: Option<TransactionType>,
    #[structopt(long)]
    /// Session in which the equity order will be place
    market_session: Option<MarketSession>,
  },
}

#[derive(Debug, StructOpt)]
enum AccountCmd {
  /// List the accounts
  List,
  /// Show the balance for an account
  Balance {
    #[structopt(long)]
    /// The account id to fetch the portfolio for
    account_id: String,
    #[structopt(long)]
    /// Get real time balance info
    real_time: bool,
  },
  /// Show the portfolio for an account
  Portfolio {
    #[structopt(long)]
    /// The account id to fetch the portfolio for
    account_id: String,

    #[structopt(long)]
    /// The number of positions to return in the response. If not specified, defaults to 50.
    ///
    /// To page through a large number of items, use the count property to specify how many items to return in a group (the default is 25),
    /// and the marker property to specify the starting point (the default is the newest).
    /// For instance, a request with no count and no marker retrieves the newest 25 items. Each response includes a marker that points to
    /// the beginning of the next group. To page through all the items, repeat the request with the marker from each previous response until
    /// you receive a response with an empty marker, indicating that there are no more items.
    count: Option<usize>,

    #[structopt(long)]
    /// The sort by query. Sorting done based on the column specified in the query paramater.
    sort_by: Option<accounts::PortfolioColumn>,

    #[structopt(long, default_value = "desc")]
    /// The sort order query. Default: desc.
    sort_order: SortOrder,

    #[structopt(long, default_value = "regular")]
    /// The market session. Default: regular
    market_session: MarketSession,

    #[structopt(long)]
    /// It gives the total values of the portfolio.
    totals_required: Option<bool>,

    #[structopt(long)]
    /// It gives position lots for positions.
    lots_required: Option<bool>,

    #[structopt(long, default_value = "quick")]
    /// The view query.
    view: accounts::PortfolioView,
  },
}

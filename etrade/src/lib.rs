#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use secstr::SecUtf8;
use std::sync::Mutex;

mod accounts;
mod session;

pub use accounts::Api as Accounts;
pub use accounts::PortfolioRequest;
pub use session::Session;
pub use session::OOB;

// The sandbox url to use as base url for the etrade api
const SANDBOX_URL: &str = "https://apisb.etrade.com";

// The production url to use as base url for the etrade api
const LIVE_URL: &str = "https://api.etrade.com";

pub enum Mode {
    Sandbox,
    Live,
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub key: SecUtf8,
    pub secret: SecUtf8,
}

impl Credentials {
    pub fn new(key: SecUtf8, secret: SecUtf8) -> Credentials {
        Credentials { key, secret }
    }
}

impl Into<oauth::Credentials> for Credentials {
    fn into(self) -> oauth::Credentials {
        oauth::Credentials::new(self.key.into_unsecure(), self.secret.into_unsecure())
    }
}

impl<T> From<oauth::Credentials<T>> for Credentials
where
    T: Into<SecUtf8>,
{
    fn from(input: oauth::Credentials<T>) -> Self {
        Credentials {
            key: input.identifier.into(),
            secret: input.secret.into(),
        }
    }
}

pub trait Store {
    // type KeyType: Into<String> + Send;
    // type SecretType: Into<SecUtf8> + Send;

    fn put(
        &self,
        namespace: impl Into<String> + Send,
        key: impl Into<String> + Send,
        value: impl Into<SecUtf8> + Send,
    ) -> Result<()>;
    fn del(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<()>;
    fn get(
        &self,
        namespace: impl AsRef<str> + Send,
        key: impl AsRef<str> + Send,
    ) -> Result<Option<SecUtf8>>;
}

#[derive(Debug)]
pub struct Memstore {
    data: Arc<Mutex<HashMap<String, HashMap<String, SecUtf8>>>>,
}

impl Memstore {
    pub fn new() -> Self {
        Memstore {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Store for Memstore {
    fn put(
        &self,
        namespace: impl Into<String> + Send,
        key: impl Into<String> + Send,
        value: impl Into<SecUtf8> + Send,
    ) -> Result<()> {
        let mut data = self.data.lock().unwrap();

        let svc_state = data
            .entry(namespace.into())
            .or_insert_with(|| HashMap::new());
        svc_state.insert(key.into(), value.into());
        Ok(())
    }

    fn del(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<()> {
        let mut data = self.data.lock().unwrap();

        if let Some(st) = data.get_mut(namespace.as_ref()) {
            st.remove(key.as_ref());
        }
        Ok(())
    }

    fn get(
        &self,
        namespace: impl AsRef<str> + Send,
        key: impl AsRef<str> + Send,
    ) -> Result<Option<SecUtf8>> {
        let data = self.data.lock().unwrap();
        Ok(data
            .get(namespace.as_ref())
            .and_then(|r| r.get(key.as_ref()).map(|v| v.clone())))
    }
}

#[cfg(test)]
pub mod tests {

    use super::{Memstore, Store};
    use anyhow::Result;
    use secstr::SecUtf8;
    pub(crate) fn init() {
        std::env::set_var("RUST_LOG", "debug");
        let _ = pretty_env_logger::try_init();
    }
    #[test]
    fn test_mem_store() {
        verify_token_store(Memstore::new());
    }

    pub fn verify_token_store(token_store: impl Store) {
        let expected: Result<SecUtf8> = Ok("hello".into());
        token_store.put("my_svc", "api_key", "hello").unwrap();
        assert_eq!(
            token_store.get("my_svc", "api_key").ok(),
            Some(expected.ok())
        );
        assert!(token_store.del("my_svc", "api_key").is_ok());
        assert!(token_store.get("my_svc", "api_key").unwrap().is_none());
    }
}

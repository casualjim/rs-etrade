use async_trait::async_trait;
use secstr::SecUtf8;
use std::borrow::{Borrow, Cow};

use anyhow::{anyhow, Result};
use secret_service::{Collection, EncryptionType, SecretService};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait Store {
    fn put<T: Into<String> + Send, S: Into<SecUtf8> + Send>(
        &mut self,
        namespace: T,
        key: T,
        value: S,
    ) -> Result<()>;
    fn del<T: AsRef<str> + Send>(&mut self, namespace: T, key: T) -> Result<()>;
    fn get<T: AsRef<str> + Send>(&self, namespace: T, key: T) -> Result<Option<SecUtf8>>;
}

#[derive(Debug)]
pub struct SecretServiceStore {
    svc: SecretService,
}

impl SecretServiceStore {
    pub fn new() -> Result<Self> {
        let svc = SecretService::new(EncryptionType::Dh)
            .map_err(|e| anyhow!("failed to acquire secret service: {}", e))?;
        Ok(Self { svc })
    }
}

impl Store for SecretServiceStore {
    fn put<T: Into<String> + Send, S: Into<SecUtf8> + Send>(
        &mut self,
        namespace: T,
        key: T,
        value: S,
    ) -> Result<()> {
        let ns = namespace.into();
        let k = key.into();
        let label = format!("secret for etradectl {}@{}", &k, &ns);
        let svc = &self.svc;
        let coll = svc
            .get_default_collection()
            .map_err(|e| anyhow!("failed to acquire secret service collection: {}", e))?;
        coll.create_item(
            &label,
            vec![(&ns, &k)],
            value.into().unsecure().as_bytes(),
            true,
            "text/plain",
        )
        .map(|_| ())
        .map_err(|e| anyhow!("failed to create secret: {}", e))
    }

    fn del<T: AsRef<str> + Send>(&mut self, namespace: T, key: T) -> Result<()> {
        let svc = &self.svc;
        let coll = svc
            .get_default_collection()
            .map_err(|e| anyhow!("failed to acquire secret service collection: {}", e))?;
        let results = coll
            .search_items(vec![(namespace.as_ref(), key.as_ref())])
            .map_err(|e| anyhow!("failed to find secret ({}:{}) "))?;

        match results.get(0) {
            Some(item) => item
                .delete()
                .map_err(|e| anyhow!("failed to delete secret {}", e)),
            _ => Ok(()),
        }
    }

    fn get<T: AsRef<str> + Send>(&self, namespace: T, key: T) -> Result<Option<SecUtf8>> {
        let svc = &self.svc;
        let coll = svc
            .get_default_collection()
            .map_err(|e| anyhow!("failed to acquire secret service collection: {}", e))?;
        let results = coll
            .search_items(vec![(namespace.as_ref(), key.as_ref())])
            .map_err(|e| {
                anyhow!(
                    "failed to find secret ({}:{}): {}",
                    namespace.as_ref(),
                    key.as_ref(),
                    e
                )
            })?;

        match results.get(0) {
            Some(item) => {
                let secret = item
                    .get_secret()
                    .map_err(|e| anyhow!("failed to get secret: {}", e))?;

                if secret.is_empty() {
                    return Ok(None);
                }
                Ok(Some(String::from_utf8(secret)?.into()))
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug)]
pub struct Memstore {
    data: HashMap<String, HashMap<String, SecUtf8>>,
}

impl Memstore {
    pub fn new() -> Self {
        Memstore {
            data: HashMap::new(),
        }
    }
}

impl Store for Memstore {
    fn put<T: Into<String> + Send, S: Into<SecUtf8> + Send>(
        &mut self,
        namespace: T,
        key: T,
        value: S,
    ) -> Result<()> {
        let svc_state = self
            .data
            .entry(namespace.into())
            .or_insert_with(|| HashMap::new());
        svc_state.insert(key.into(), value.into());
        Ok(())
    }

    fn del<T: AsRef<str> + Send>(&mut self, namespace: T, key: T) -> Result<()> {
        if let Some(st) = self.data.get_mut(namespace.as_ref()) {
            st.remove(key.as_ref());
        }
        Ok(())
    }

    fn get<T: AsRef<str> + Send>(&self, namespace: T, key: T) -> Result<Option<SecUtf8>> {
        Ok(self
            .data
            .get(namespace.as_ref())
            .and_then(|r| r.get(key.as_ref()).map(|v| v.clone())))
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::{Memstore, SecretServiceStore, Store};
    use anyhow::Result;
    use secstr::SecUtf8;
    use std::collections::HashMap;

    #[test]
    fn test_mem_store() {
        verify_token_store(Memstore::new());
    }

    fn verify_token_store(mut token_store: impl Store) {
        let expected: Result<SecUtf8> = Ok("hello".into());
        token_store.put("my_svc", "api_key", "hello").unwrap();
        assert_eq!(
            token_store.get("my_svc", "api_key").ok(),
            Some(expected.ok())
        );
        assert!(token_store.del("my_svc", "api_key").is_ok());
        assert!(token_store.get("my_svc", "api_key").unwrap().is_none());
    }

    #[test]
    fn test_secret_service_store() {
        verify_token_store(SecretServiceStore::new().unwrap())
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

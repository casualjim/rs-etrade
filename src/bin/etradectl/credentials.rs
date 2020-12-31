use secstr::SecUtf8;

use anyhow::{anyhow, Result};
use etrade::Store;
use secret_service::{EncryptionType, SecretService};

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
    fn put(
        &self,
        namespace: impl Into<String> + Send,
        key: impl Into<String> + Send,
        value: impl Into<SecUtf8> + Send,
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

    fn del(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<()> {
        let svc = &self.svc;
        let coll = svc
            .get_default_collection()
            .map_err(|e| anyhow!("failed to acquire secret service collection: {}", e))?;
        let results = coll
            .search_items(vec![(namespace.as_ref(), key.as_ref())])
            .map_err(|_e| anyhow!("failed to find secret ({}:{}) "))?;

        match results.get(0) {
            Some(item) => item
                .delete()
                .map_err(|e| anyhow!("failed to delete secret {}", e)),
            _ => Ok(()),
        }
    }

    fn get(
        &self,
        namespace: impl AsRef<str> + Send,
        key: impl AsRef<str> + Send,
    ) -> Result<Option<SecUtf8>> {
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

#[cfg(test)]
mod tests {
    use crate::{credentials::SecretServiceStore, etrade};
    use anyhow::Result;
    use secstr::SecUtf8;

    #[test]
    fn test_secret_service_store() {
        verify_token_store(SecretServiceStore::new().unwrap())
    }

    fn verify_token_store(token_store: impl etrade::Store) {
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

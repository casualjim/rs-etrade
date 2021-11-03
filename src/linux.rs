use secstr::SecUtf8;

use crate::Store;
use anyhow::{anyhow, Result};
use secret_service::{EncryptionType, SecretService};

#[derive(Debug)]
pub struct KeychainStore;

impl KeychainStore {
  pub async fn new() -> Result<Self> {
    Ok(Self)
  }
}

impl Store for KeychainStore {
  fn put(
    &self,
    namespace: impl Into<String> + Send,
    key: impl Into<String> + Send,
    value: impl Into<SecUtf8> + Send,
  ) -> Result<()> {
    let ns = namespace.into();
    let k = key.into();
    let label = format!("secret for etradectl {}@{}", &k, &ns);
    let svc = SecretService::new(EncryptionType::Dh).map_err(|e| anyhow!("failed to acquire secret service: {}", e))?;
    let coll = svc
      .get_default_collection()
      .map_err(|e| anyhow!("failed to acquire secret service collection: {}", e))?;
    coll
      .create_item(
        &label,
        vec![(&ns as &str, &k as &str)].into_iter().collect(),
        value.into().unsecure().as_bytes(),
        true,
        "text/plain",
      )
      .map(|_| ())
      .map_err(|e| anyhow!("failed to create secret: {}", e))
  }

  fn del(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<()> {
    let svc = SecretService::new(EncryptionType::Dh).map_err(|e| anyhow!("failed to acquire secret service: {}", e))?;
    let coll = svc
      .get_default_collection()
      .map_err(|e| anyhow!("failed to acquire secret service collection: {}", e))?;
    let results = coll
      .search_items(vec![(namespace.as_ref(), key.as_ref())].into_iter().collect())
      .map_err(|e| anyhow!("failed to find secret ({}:{}): {}", namespace.as_ref(), key.as_ref(), e))?;

    match results.get(0) {
      Some(item) => item.delete().map_err(|e| anyhow!("failed to delete secret {}", e)),
      _ => Ok(()),
    }
  }

  fn get(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<Option<SecUtf8>> {
    let svc = SecretService::new(EncryptionType::Dh).map_err(|e| anyhow!("failed to acquire secret service: {}", e))?;
    let coll = svc
      .get_default_collection()
      .map_err(|e| anyhow!("failed to acquire secret service collection: {}", e))?;
    let results = coll
      .search_items(vec![(namespace.as_ref(), key.as_ref())].into_iter().collect())
      .map_err(|e| anyhow!("failed to find secret ({}:{}): {}", namespace.as_ref(), key.as_ref(), e))?;

    match results.get(0) {
      Some(item) => {
        let secret = item.get_secret().map_err(|e| anyhow!("failed to get secret: {}", e))?;

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
  use super::KeychainStore;
  use anyhow::Result;
  use secstr::SecUtf8;

  #[tokio::test]
  async fn test_secret_service_store() {
    verify_token_store(KeychainStore::new().await.unwrap())
  }

  fn verify_token_store(token_store: impl crate::Store) {
    let expected: Result<SecUtf8> = Ok("hello".into());
    token_store.put("my_svc", "api_key", "hello").unwrap();
    assert_eq!(token_store.get("my_svc", "api_key").ok(), Some(expected.ok()));
    assert!(token_store.del("my_svc", "api_key").is_ok());
    assert!(token_store.get("my_svc", "api_key").unwrap().is_none());
  }
}

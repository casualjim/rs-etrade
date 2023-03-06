use crate::Store;
use anyhow::{anyhow, Result};
use secstr::SecUtf8;
use security_framework::os::macos::keychain::SecKeychain;
use security_framework::os::macos::passwords::find_generic_password;

#[derive(Debug)]
pub struct KeychainStore;

#[async_trait]
impl Store for KeychainStore {
  async fn put(
    &self,
    namespace: impl Into<String> + Send,
    key: impl Into<String> + Send,
    value: impl Into<SecUtf8> + Send,
  ) -> Result<()> {
    let ns = namespace.into();
    let k = key.into();

    SecKeychain::default()
      .map_err(|e| anyhow!("{}", e))?
      .set_generic_password(&ns, &k, value.into().unsecure().as_bytes())
      .map_err(|e| anyhow!("{}", e))?;

    Ok(())
  }

  async fn del(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<()> {
    let (_, item) = find_generic_password(None, namespace.as_ref(), key.as_ref())?;
    item.delete();
    Ok(())
  }

  async fn get(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<Option<SecUtf8>> {
    let (secret, _) = find_generic_password(None, namespace.as_ref(), key.as_ref())?;
    Ok(Some(String::from_utf8(secret.to_vec())?.into()))
  }
}

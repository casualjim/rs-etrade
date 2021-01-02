use anyhow::Result;
use security_framework::os::macos::keychain::SecKeychain;
use security_framework::os::macos::passwords::find_generic_password;

#[derive(Debug)]
pub struct KeychainStore;

impl Store for KeychainStore {
  fn put(
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

  fn del(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<()> {
    let (_, item) = find_generic_password(None, namespace.as_ref(), key.as_ref())?;

    item.delete();

    Ok(())

    // let svc = &self.svc;
    // let coll = svc
    //   .get_default_collection()
    //   .map_err(|e| anyhow!("failed to acquire secret service collection: {}", e))?;
    // let results = coll
    //   .search_items(vec![(namespace.as_ref(), key.as_ref())])
    //   .map_err(|_e| anyhow!("failed to find secret ({}:{}) "))?;

    // match results.get(0) {
    //   Some(item) => item.delete().map_err(|e| anyhow!("failed to delete secret {}", e)),
    //   _ => Ok(()),
    // }
  }

  fn get(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<Option<SecUtf8>> {
    let (secret, _) = find_generic_password(None, namespace.as_ref(), key.as_ref())?;
    Ok(Some(String::from_utf8(secret)?.into()))
    // let svc = &self.svc;
    // let coll = svc
    //   .get_default_collection()
    //   .map_err(|e| anyhow!("failed to acquire secret service collection: {}", e))?;
    // let results = coll
    //   .search_items(vec![(namespace.as_ref(), key.as_ref())])
    //   .map_err(|e| anyhow!("failed to find secret ({}:{}): {}", namespace.as_ref(), key.as_ref(), e))?;

    // match results.get(0) {
    //   Some(item) => {
    //     let secret = item.get_secret().map_err(|e| anyhow!("failed to get secret: {}", e))?;

    //     if secret.is_empty() {
    //       return Ok(None);
    //     }
    //     Ok(Some(String::from_utf8(secret)?.into()))
    //   }
    //   None => Ok(None),
    // }
  }
}

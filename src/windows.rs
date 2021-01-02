use anyhow::{anyhow, Result};
use byteorder::{ByteOrder, LittleEndian};
use secstr::SecUtf8;
use std::ffi::OsStr;
use std::iter::once;
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStrExt;
use std::slice;
use std::str;
use winapi::shared::minwindef::FILETIME;
use winapi::um::wincred::{
  CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_ENTERPRISE, CRED_TYPE_GENERIC, PCREDENTIALW,
  PCREDENTIAL_ATTRIBUTEW,
};

use crate::Store;

#[derive(Debug)]
pub struct KeychainStore;

impl Store for KeychainStore {
  fn put(
    &self,
    namespace: impl Into<String> + Send,
    key: impl Into<String> + Send,
    value: impl Into<SecUtf8> + Send,
  ) -> Result<()> {
    let ns: String = namespace.into();
    let k: String = key.into();

    // Setting values of credential

    let flags = 0;
    let cred_type = CRED_TYPE_GENERIC;
    let target_name: String = [k.as_str(), ns.as_str()].join(".");
    let mut target_name = to_wstr(&target_name);

    // empty string for comments, and target alias,
    // I don't use here
    let label = format!("secret for etradectl {}@{}", &k, &ns);
    let mut empty_str = to_wstr(&label);

    // Ignored by CredWriteW
    let last_written = FILETIME {
      dwLowDateTime: 0,
      dwHighDateTime: 0,
    };

    // In order to allow editing of the password
    // from within Windows, the password must be
    // transformed into utf16. (but because it's a
    // blob, it then needs to be passed to windows
    // as an array of bytes).
    let blob_u16 = to_wstr_no_null(value.into().unsecure());
    let mut blob = vec![0; blob_u16.len() * 2];
    LittleEndian::write_u16_into(&blob_u16, &mut blob);

    let blob_len = blob.len() as u32;
    let persist = CRED_PERSIST_ENTERPRISE;
    let attribute_count = 0;
    let attributes: PCREDENTIAL_ATTRIBUTEW = std::ptr::null_mut();
    let mut username = to_wstr(&k);

    let mut credential = CREDENTIALW {
      Flags: flags,
      Type: cred_type,
      TargetName: target_name.as_mut_ptr(),
      Comment: empty_str.as_mut_ptr(),
      LastWritten: last_written,
      CredentialBlobSize: blob_len,
      CredentialBlob: blob.as_mut_ptr(),
      Persist: persist,
      AttributeCount: attribute_count,
      Attributes: attributes,
      TargetAlias: empty_str.as_mut_ptr(),
      UserName: username.as_mut_ptr(),
    };
    // raw pointer to credential, is coerced from &mut
    let pcredential: PCREDENTIALW = &mut credential;

    // Call windows API
    match unsafe { CredWriteW(pcredential, 0) } {
      0 => Err(anyhow!("windows vault error")),
      _ => Ok(()),
    }
  }

  fn del(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<()> {
    let target_name: String = [key.as_ref(), namespace.as_ref()].join(".");

    let cred_type = CRED_TYPE_GENERIC;
    let target_name = to_wstr(&target_name);

    match unsafe { CredDeleteW(target_name.as_ptr(), cred_type, 0) } {
      0 => Err(anyhow!("windows vault error")),
      _ => Ok(()),
    }
  }

  fn get(&self, namespace: impl AsRef<str> + Send, key: impl AsRef<str> + Send) -> Result<Option<SecUtf8>> {
    // passing uninitialized pcredential.
    // Should be ok; it's freed by a windows api
    // call CredFree.
    let mut pcredential = MaybeUninit::uninit();

    let target_name: String = [key.as_ref(), namespace.as_ref()].join(".");
    let target_name = to_wstr(&target_name);

    let cred_type = CRED_TYPE_GENERIC;

    // Windows api call
    match unsafe { CredReadW(target_name.as_ptr(), cred_type, 0, pcredential.as_mut_ptr()) } {
      0 => Err(anyhow!("windows vault error")),
      _ => {
        let pcredential = unsafe { pcredential.assume_init() };
        // Dereferencing pointer to credential
        let credential: CREDENTIALW = unsafe { *pcredential };

        // get blob by creating an array from the pointer
        // and the length reported back from the credential
        let blob_pointer: *const u8 = credential.CredentialBlob;
        let blob_len: usize = credential.CredentialBlobSize as usize;

        // blob needs to be transformed from bytes to an
        // array of u16, which will then be transformed into
        // a utf8 string. As noted above, this is to allow
        // editing of the password from within the vault order
        // or other windows programs, which operate in utf16
        let blob: &[u8] = unsafe { slice::from_raw_parts(blob_pointer, blob_len) };
        let mut blob_u16 = vec![0; blob_len / 2];
        LittleEndian::read_u16_into(&blob, &mut blob_u16);

        // Now can get utf8 string from the array
        let password = String::from_utf16(&blob_u16)
          .map(|pass| Some(pass.to_string().into()))
          .map_err(|_| anyhow!("windows vault error"));

        // Free the credential
        unsafe {
          CredFree(pcredential as *mut _);
        }

        password
      }
    }
  }
}
// helper function for turning utf8 strings to windows
// utf16
fn to_wstr(s: &str) -> Vec<u16> {
  OsStr::new(s).encode_wide().chain(once(0)).collect()
}

fn to_wstr_no_null(s: &str) -> Vec<u16> {
  OsStr::new(s).encode_wide().collect()
}

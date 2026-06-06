use std::{ffi::c_void, fmt, ptr};

use git2::{Cred, CredentialType, RemoteCallbacks};

use super::AuthMode;

pub trait CredentialStore: Send + Sync {
    fn read(&self, key: &str) -> Result<Option<SyncCredential>, String>;
    fn write(&self, key: &str, credential: &SyncCredential) -> Result<(), String>;
    fn delete(&self, key: &str) -> Result<(), String>;
}

#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncCredential {
    pub username: String,
    pub secret: Secret,
}

pub fn credential_key(host: &str) -> String {
    format!("openmacro/sync/{host}")
}

#[derive(Default)]
pub struct WindowsCredentialStore;

impl CredentialStore for WindowsCredentialStore {
    fn read(&self, key: &str) -> Result<Option<SyncCredential>, String> {
        tracing::debug!(key = %key, "reading sync credential");
        #[cfg(windows)]
        unsafe {
            use windows::core::PCWSTR;
            use windows::Win32::Security::Credentials::{
                CredFree, CredReadW, CREDENTIALW, CRED_TYPE_GENERIC,
            };

            let key_utf16 = to_utf16(key);
            let mut credential_ptr: *mut CREDENTIALW = ptr::null_mut();
            let read = CredReadW(
                PCWSTR::from_raw(key_utf16.as_ptr()),
                CRED_TYPE_GENERIC,
                0,
                &mut credential_ptr,
            );
            if let Err(err) = read {
                let code = err.code().0 as u32;
                if code == windows::Win32::Foundation::ERROR_NOT_FOUND.0 || code == 0x8007_0490 {
                    return Ok(None);
                }
                return Err(err.to_string());
            }

            let credential = &*credential_ptr;
            let username = from_pcwstr(credential.UserName);
            let secret_bytes = std::slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            );
            let secret =
                String::from_utf8(secret_bytes.to_vec()).map_err(|error| error.to_string())?;
            CredFree(credential_ptr.cast::<c_void>());
            Ok(Some(SyncCredential {
                username,
                secret: Secret::new(secret),
            }))
        }

        #[cfg(not(windows))]
        {
            let _ = key;
            Err("windows credential manager unavailable on this platform".to_string())
        }
    }

    fn write(&self, key: &str, credential: &SyncCredential) -> Result<(), String> {
        // SECURITY: credential secret is never logged; only key and username metadata are recorded.
        tracing::debug!(key = %key, username = %credential.username, "writing sync credential");
        #[cfg(windows)]
        unsafe {
            use windows::core::PWSTR;
            use windows::Win32::Security::Credentials::{
                CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
            };

            let key_utf16 = to_utf16(key);
            let user_utf16 = to_utf16(&credential.username);
            let blob = credential.secret.expose().as_bytes().to_vec();
            let cred = CREDENTIALW {
                Type: CRED_TYPE_GENERIC,
                TargetName: PWSTR(key_utf16.as_ptr() as *mut _),
                UserName: PWSTR(user_utf16.as_ptr() as *mut _),
                CredentialBlobSize: blob.len() as u32,
                CredentialBlob: blob.as_ptr() as *mut u8,
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                Comment: PWSTR(ptr::null_mut()),
                TargetAlias: PWSTR(ptr::null_mut()),
                Attributes: ptr::null_mut(),
                AttributeCount: 0,
                ..Default::default()
            };

            CredWriteW(&cred, 0).map_err(|error| error.to_string())
        }

        #[cfg(not(windows))]
        {
            let _ = (key, credential);
            Err("windows credential manager unavailable on this platform".to_string())
        }
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        tracing::debug!(key = %key, "deleting sync credential");
        #[cfg(windows)]
        unsafe {
            use windows::core::PCWSTR;
            use windows::Win32::Security::Credentials::{CredDeleteW, CRED_TYPE_GENERIC};

            let key_utf16 = to_utf16(key);
            CredDeleteW(PCWSTR::from_raw(key_utf16.as_ptr()), CRED_TYPE_GENERIC, 0)
                .map_err(|error| error.to_string())
        }

        #[cfg(not(windows))]
        {
            let _ = key;
            Err("windows credential manager unavailable on this platform".to_string())
        }
    }
}

pub fn git_remote_callbacks<'a>(
    auth: &'a AuthMode,
    store: &'a dyn CredentialStore,
) -> RemoteCallbacks<'a> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, allowed| match auth {
        AuthMode::HttpsPat { host, username } => {
            tracing::debug!(
                host = %host,
                username = %username_from_url.unwrap_or(username),
                "providing https sync credential"
            );
            if !allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
                return Err(git2::Error::from_str("https credentials not allowed"));
            }
            let key = credential_key(host);
            let credential = store
                .read(&key)
                .map_err(|error| git2::Error::from_str(&error))?
                .ok_or_else(|| git2::Error::from_str("missing https credential"))?;
            let user = if credential.username.is_empty() {
                username_from_url.unwrap_or(username)
            } else {
                credential.username.as_str()
            };
            // PATs in-process only, deleted from stack as soon as git2 consumes them.
            Cred::userpass_plaintext(user, credential.secret.expose())
        }
        AuthMode::Ssh => {
            if !allowed.contains(CredentialType::SSH_KEY) {
                return Err(git2::Error::from_str("ssh credentials not allowed"));
            }
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        }
    });
    callbacks
}

#[cfg(windows)]
fn to_utf16(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

#[cfg(windows)]
fn from_pcwstr(value: windows::core::PWSTR) -> String {
    if value.is_null() {
        return String::new();
    }
    unsafe {
        let mut len = 0;
        while *value.0.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(value.0, len))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use super::{credential_key, CredentialStore, Secret, SyncCredential};

    #[derive(Default)]
    struct MockStore {
        entries: Mutex<HashMap<String, SyncCredential>>,
    }

    impl CredentialStore for MockStore {
        fn read(&self, key: &str) -> Result<Option<SyncCredential>, String> {
            Ok(self.entries.lock().unwrap().get(key).cloned())
        }

        fn write(&self, key: &str, credential: &SyncCredential) -> Result<(), String> {
            self.entries
                .lock()
                .unwrap()
                .insert(key.to_string(), credential.clone());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), String> {
            self.entries.lock().unwrap().remove(key);
            Ok(())
        }
    }

    #[test]
    fn secret_debug_is_redacted() {
        let secret = Secret::new("super-secret".to_string());

        assert_eq!(format!("{secret:?}"), "<redacted>");
    }

    #[test]
    fn mock_store_roundtrip_and_delete() {
        let store = MockStore::default();
        let key = credential_key("github.com");
        let credential = SyncCredential {
            username: "alice".to_string(),
            secret: Secret::new("pat-123".to_string()),
        };

        store.write(&key, &credential).unwrap();
        let loaded = store.read(&key).unwrap().unwrap();
        store.delete(&key).unwrap();

        assert_eq!(loaded.username, "alice");
        assert_eq!(loaded.secret.expose(), "pat-123");
        assert!(store.read(&key).unwrap().is_none());
    }

    #[cfg(windows)]
    #[test]
    fn windows_store_roundtrip_and_delete() {
        let store = super::WindowsCredentialStore;
        let key = credential_key("phase-09-test.local");
        let credential = SyncCredential {
            username: "alice".to_string(),
            secret: Secret::new("pat-credential-test".to_string()),
        };

        store.write(&key, &credential).unwrap();
        let loaded = store.read(&key).unwrap().unwrap();
        store.delete(&key).unwrap();

        assert_eq!(loaded.username, "alice");
        assert_eq!(loaded.secret.expose(), "pat-credential-test");
        assert!(store.read(&key).unwrap().is_none());
    }
}

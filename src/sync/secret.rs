use keyring::Entry;

const SERVICE: &str = "dev.edfloreshz.Tasks.caldav";

fn entry(username: &str) -> keyring::Result<Entry> {
    Entry::new(SERVICE, username)
}

/// Returns Ok(None) if the entry exists but has no value, or no entry exists.
pub fn load(username: &str) -> Option<String> {
    if username.is_empty() {
        return None;
    }
    match entry(username).and_then(|e| e.get_password()) {
        Ok(s) => Some(s),
        Err(keyring::Error::NoEntry) => None,
        Err(e) => {
            tracing::warn!("keyring load for {username}: {e}");
            None
        }
    }
}

pub fn store(username: &str, password: &str) -> Result<(), String> {
    if username.is_empty() {
        return Err("username is empty".to_string());
    }
    entry(username)
        .and_then(|e| e.set_password(password))
        .map_err(|e| e.to_string())
}

pub fn delete(username: &str) {
    if username.is_empty() {
        return;
    }
    if let Ok(e) = entry(username) {
        let _ = e.delete_credential();
    }
}

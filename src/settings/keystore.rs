use crate::APP_ID;
use keyring::{Entry, Error};

pub const KEY_NAME: &str = "soniox_api_key";

pub enum KeyStorage {
    Keyring,
    PlainFile { reason: String },
}

fn entry() -> Result<Entry, Error> {
    Entry::new(APP_ID, KEY_NAME)
}

pub fn load() -> Result<Option<String>, Error> {
    match entry()?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(Error::NoEntry) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn store(key: &str) -> Result<(), Error> {
    entry()?.set_password(key)
}

pub fn delete() -> Result<(), Error> {
    match entry()?.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(e) => Err(e),
    }
}
use std::ops::Deref;

use serde::{de::DeserializeOwned, Serialize};
use sled::Db;

#[derive(Debug, Clone)]
pub struct TypedDb(Db);

impl TypedDb {
    pub fn new(db: Db) -> Self {
        Self(db)
    }

    pub fn insert<T: Serialize>(&self, key: &str, value: &T) -> anyhow::Result<()> {
        let bytes = bincode::serialize(value)?;
        self.0.insert(key, bytes)?;
        Ok(())
    }
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let bytes = self.0.get(key)?;
        let Some(bytes) = bytes else {
            return Ok(None);
        };
        let value = bincode::deserialize_from(bytes.deref())?;
        Ok(Some(value))
    }
}

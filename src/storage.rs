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
        let type_name = std::any::type_name::<T>().to_string();

        let bytes = bincode::serialize(value)?;
        self.0.insert(type_name + key, bytes)?;
        Ok(())
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let type_name = std::any::type_name::<T>().to_string();
        let bytes = self.0.get(type_name + key)?;
        let Some(bytes) = bytes else {
            return Ok(None);
        };
        let value = bincode::deserialize_from(bytes.deref())?;
        Ok(Some(value))
    }

    pub fn get_all<T: DeserializeOwned>(&self) -> anyhow::Result<Vec<T>> {
        let type_name = std::any::type_name::<T>();

        let all: anyhow::Result<Vec<_>> = self
            .0
            .scan_prefix(type_name)
            .map(|res| {
                let (_key, bytes) = res?;
                let value = bincode::deserialize_from(bytes.deref())?;
                Ok(value)
            })
            // expected to be in order of insertion, we want the most recent(last) first
            .rev()
            .collect();
        all
    }
}

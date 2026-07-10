use serde::Serialize;

use crate::{HorizonError, HorizonResult};

pub fn canonical_bytes<T: Serialize>(value: &T) -> HorizonResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| HorizonError::Serialization(error.to_string()))
}

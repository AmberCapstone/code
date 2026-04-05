use std::collections::HashMap;

use proto::sensor::nvm::Parameters;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Creates a more user-friendly structure than the packed proto
#[derive(Serialize, Deserialize)]
pub struct UserParameters {
    name: String,
    supercapacitor_uf: u32,
    camera_settings: HashMap<sccb::Reg, String>,
}

#[derive(Error, Clone, Debug)]
pub enum ConversionError {
    #[error("Name is too long. Max 7 characters")]
    NameTooLong,

    #[error("Invalid hex string \"{0}\". Expected form is \"0x1A\"")]
    InvalidHex(String),
}

impl TryFrom<UserParameters> for Parameters {
    type Error = ConversionError;

    fn try_from(value: UserParameters) -> Result<Self, Self::Error> {
        if value.name.len() > 7 {
            return Err(ConversionError::NameTooLong);
        }

        Ok(Self {
            name: value.name,
            supercapacitor_uf: value.supercapacitor_uf,
            camera_settings: pack_camera_settings(&value.camera_settings)?,
        })
    }
}

impl From<Parameters> for UserParameters {
    fn from(value: Parameters) -> Self {
        Self {
            name: value.name,
            supercapacitor_uf: value.supercapacitor_uf,
            camera_settings: unpack_camera_settings(&value.camera_settings),
        }
    }
}

fn pack_camera_settings(settings: &HashMap<sccb::Reg, String>) -> Result<Vec<u32>, ConversionError> {
    fn from_hex(s: &str) -> Result<u8, ConversionError> {
        let trimmed = s.strip_prefix("0x").ok_or(ConversionError::InvalidHex(s.to_string()))?;
        u8::from_str_radix(trimmed, 16).map_err(|_| ConversionError::InvalidHex(s.to_string()))
    }
    settings
        .iter()
        .map(|(reg, val)| {
            let val = from_hex(val)?;
            Ok(u32::from_le_bytes([*reg as u8, val, 0x00, 0x00]))
        })
        .collect()
}

fn unpack_camera_settings(packed: &[u32]) -> HashMap<sccb::Reg, String> {
    packed
        .iter()
        .filter_map(|u| {
            let [reg, val, _, _] = u.to_le_bytes(); // destructure
            sccb::Reg::from_repr(reg).map(|r| (r, format!("0x{val:02x}"))) // append converted value if conversion succeeds
        })
        .collect()
}

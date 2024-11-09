use anyhow::Result;
use crm_metadata::AppConfig;

// pub mod pb {
//     use std::time::SystemTime;

//     use prost_types::Timestamp;

//     include!(concat!(env!("OUT_DIR"), "/crm.rs"));

// }
fn main() -> Result<()> {
    AppConfig::try_load()?;

    Ok(())
}

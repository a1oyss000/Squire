use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct IncludeRef {
    pub use_path: String,
    pub with: Option<HashMap<String, serde_yaml::Value>>,
}

impl<'de> Deserialize<'de> for IncludeRef {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(rename = "use")]
            use_path: String,
            with: Option<HashMap<String, serde_yaml::Value>>,
        }
        let raw = Raw::deserialize(d)?;
        Ok(IncludeRef { use_path: raw.use_path, with: raw.with })
    }
}

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct EnchantJson {
    pub bash: BashConfig,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BashConfig {
    pub allow: Vec<String>,
}

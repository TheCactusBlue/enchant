use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct EnchantJson {
    pub permissions: Permissions,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Permissions {
    pub bash: BashConfig,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BashConfig {
    pub allow: Vec<String>,
}

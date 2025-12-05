use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::tools::{EnchantTool, EnchantToolInfo};

pub struct Grep;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GrepInput {
    pub query: String,
}

impl EnchantTool for Grep {
    type Input = GrepInput;

    fn get_info() -> EnchantToolInfo {
        EnchantToolInfo::new("Grep")
            .with_description("Uses ripgrep to search through the directory.")
    }

    async fn execute(_input: Self::Input) {}
}

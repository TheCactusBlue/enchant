use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ModelInfo {
    pub name: Option<String>,
    pub max_context: Option<u64>,
}

pub type ModelInfoMap = HashMap<String, ModelInfo>;

static MODEL_INFO: OnceLock<ModelInfoMap> = OnceLock::new();

pub fn get_model_info(name: &str) -> ModelInfo {
    MODEL_INFO
        .get_or_init(|| serde_json::from_str(include_str!("./model_info.json")).unwrap())
        .get(name)
        .cloned()
        .unwrap_or_default()
}

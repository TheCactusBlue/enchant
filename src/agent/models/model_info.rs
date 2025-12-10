use std::{collections::HashMap, sync::OnceLock};

use iocraft::Color;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ModelInfo {
    pub name: Option<String>,
    pub max_context: Option<u64>,
}

impl ModelInfo {
    pub fn get_color(&self) -> Color {
        let name = if let Some(name) = &self.name {
            name
        } else {
            return Color::White;
        };
        match name {
            name if name.starts_with("Claude") => Color::Rgb {
                r: 204,
                g: 124,
                b: 94,
            },
            name if name.starts_with("GPT-") || name.starts_with("OpenAI") => Color::Rgb {
                r: 46,
                g: 255,
                b: 137,
            },
            _ => Color::White,
        }
    }
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

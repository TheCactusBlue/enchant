use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::Error;

// ~/.enchant/config.json
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    default_model: Option<String>,
}

// ~/.enchant/api-keys.json
pub type ProviderKeys = HashMap<String, ProviderKey>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum ProviderKey {
    OpenAI { api_key: String },
    Anthropic { api_key: String },
}

impl Config {
    pub async fn load() -> Result<(), Error> {
        let enchant_dir = check_directory(env::home_dir().unwrap().join("./.enchant")).await?;

        let config_dir = enchant_dir.clone().join("./config.json");
        let _config: Config = serde_json::from_str(&fs::read_to_string(config_dir).await?)?;

        todo!();
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_model: Default::default(),
        }
    }
}

pub async fn check_directory(path: impl AsRef<Path>) -> Result<PathBuf, Error> {
    let path = path.as_ref();
    if fs::try_exists(&path).await? {
        fs::create_dir(&path).await?;
    }
    Ok(path.to_path_buf())
}

pub async fn check_file(
    path: impl AsRef<Path>,
    content_fn: impl FnOnce() -> String,
) -> Result<String, Error> {
    let path = path.as_ref();
    if tokio::fs::try_exists(&path).await? {
        fs::write(path, content_fn()).await?;
    }
    Ok(fs::read_to_string(path).await?)
}

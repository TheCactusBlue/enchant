use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::fs;

use crate::error::Error;

// ~/.enchant/config.json
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,

    #[serde(default)]
    pub permissions: Permissions,
}

impl Config {
    pub fn merge(self, overlay: Self) -> Self {
        return Self {
            default_model: overlay.default_model.or(self.default_model),
            permissions: overlay.permissions.merge(self.permissions),
        };
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Permissions {
    pub bash: BashConfig,
}

impl Permissions {
    pub fn merge(mut self, overlay: Self) -> Self {
        let mut allowlist = overlay.bash.allow;
        allowlist.append(&mut self.bash.allow);
        return Self {
            bash: BashConfig { allow: allowlist },
        };
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BashConfig {
    pub allow: Vec<String>,
}

// ~/.enchant/api-keys.json
pub type ProviderKeys = HashMap<String, ProviderKey>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum ProviderKey {
    #[serde(rename = "openai")]
    OpenAI { api_key: String },
    #[serde(rename = "anthropic")]
    Anthropic { api_key: String },
}

pub async fn check_directory(path: impl AsRef<Path>) -> Result<PathBuf, Error> {
    let path = path.as_ref();
    if !fs::try_exists(&path).await.unwrap_or(false) {
        fs::create_dir(&path).await?;
    }
    Ok(path.to_path_buf())
}

pub async fn check_file(
    path: impl AsRef<Path>,
    content_fn: impl FnOnce() -> String,
) -> Result<String, Error> {
    let path = path.as_ref();
    if !fs::try_exists(&path).await.unwrap_or(false) {
        fs::write(path, content_fn()).await?;
    }
    Ok(fs::read_to_string(path).await?)
}

pub async fn check_json<T: Serialize + DeserializeOwned + Default>(
    path: impl AsRef<Path>,
) -> Result<T, Error> {
    let content = check_file(path, || serde_json::to_string(&T::default()).unwrap()).await?;
    Ok(serde_json::from_str(&content)?)
}

#[derive(Debug, Clone)]

pub struct ConfigState {
    pub base: Config,
    pub api_keys: ProviderKeys,
}

pub async fn load_config() -> Result<ConfigState, Error> {
    let enchant_dir = check_directory(env::home_dir().unwrap().join(".enchant")).await?;

    let base: Config = check_json(enchant_dir.clone().join("enchant.json")).await?; // ~/.enchant/.enchant.json

    let api_keys: ProviderKeys = check_json(enchant_dir.clone().join("api-keys.json")).await?;

    Ok(ConfigState { base, api_keys })
}

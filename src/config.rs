use color_eyre::eyre::{Context as _, Result};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Cli;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// Site <h1>
    pub name: String,
    /// Site <title>
    pub title: String,
    /// Site description
    pub desc: String,
    /// Use the bundled normalize.min.css?
    #[serde(default = "default_true")]
    pub normalize: bool,
    /// Site BaseURL
    #[serde(rename = "baseURL")]
    pub baseurl: String,
    /// Where to put artifacts
    #[serde(default = "default_outdir")]
    pub out_dir: PathBuf,
    /// HBS templates folder
    pub theme: PathBuf,
    /// Static assets folder copied directly to output.
    /// No assets folder means no static assets will be copied.
    pub assets: Option<PathBuf>,
    /// Extra values stored in the config for convenience lookup
    pub env: HashMap<String, serde_json::Value>,
}

fn default_outdir() -> PathBuf {
    "./html".into()
}

const fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Link {
    name: String,
    value: String,
    icon: Option<String>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        Self::from_str(
            fs::read_to_string(path.as_ref()).context(format!("Error opening: \"{}\"", path.as_ref().display()))?,
        )
    }

    pub fn from_str(s: impl AsRef<str>) -> Result<Config> {
        let config: Config = toml::from_str(s.as_ref())?;
        Ok(config)
    }

    pub fn update_from(&mut self, cli: &Cli) {
        if let Some(output_folder) = cli.output.clone() {
            self.out_dir = output_folder;
        }
        
        if let Some(theme_folder) = cli.theme.clone() {
            self.theme = theme_folder;
        }
    
        if let Some(assets_folder) = cli.assets.clone() {
            self.assets = Some(assets_folder);
        }
    
    }

    pub fn merge_with(mut self, cli: &Cli) -> Self {
        self.update_from(cli);
        self
    }
}

impl TryFrom<&Config> for serde_json::Value {
    type Error = color_eyre::eyre::Error;
    fn try_from(value: &Config) -> std::result::Result<Self, Self::Error> {
        Ok(serde_json::json!({
            "site": {
                "name": value.name,
                "title": value.title,
                "desc": value.desc,
                "baseurl": value.baseurl,
            },
            "normalize": include_str!("normalize.min.css"),
            "env": value.env,
        }))
    }
}

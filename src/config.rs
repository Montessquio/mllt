/*
    MLLT Simple Static Site Generator
    Copyright (C) 2025 Nicolas "Montessquio" Suarez

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use color_eyre::eyre::{Context as _, Result};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Cli;

/// A unified configuration struct, parsed from a `mllt.toml`
/// site configuration file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// Project-specific options.
    pub site: Site,

    /// Extra values stored in the config for convenience lookup
    pub params: HashMap<String, serde_json::Value>,
}

impl Default for Config {
    /// This default is used primarily for creating a [new project](crate::new).
    /// It doesn't describe anything useful besides an example site configuration.
    fn default() -> Self {
        Self {
            site: Site {
                baseurl: "example.com".to_owned(),
                out_dir: "./output".into(),
                content: "./content".into(),
                theme: Some("./theme".into()),
                assets: Some("./assets".into()),
                strict: false,
            },
            params: {
                let mut hm: HashMap<String, serde_json::Value> = HashMap::new();
                hm.insert("title".into(), "MLLT Example Site".into());
                hm.insert("desc".into(), "This is an example MLLT site.".into());
                hm.insert("some_nonstring_value".into(), 42.into());
                hm.insert("links".into(), serde_json::json!([
                    { "name": "My Social Media", "value": "@example.bsky.app", "iconuri": "./bsky_icon.png" },
                    { "name": "My Blog", "value": "https://blog.example.com" },
                    { "name": "My Github", "value": "https://github.com", "iconuri": "./gh_icon.png" },
                ]));
                hm.insert(
                    "made_with".into(),
                    serde_json::json!({
                        "name": "mllt",
                        "link": "https://github.com/Montessquio/mllt",
                    }),
                );
                hm
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Site {
    /// Site BaseURL
    #[serde(rename = "baseURL")]
    pub baseurl: String,

    /// Where to put artifacts
    #[serde(rename = "publishdir", default = "default_outdir")]
    pub out_dir: PathBuf,

    /// HBS templates folder
    pub content: PathBuf,

    /// HBS partials folder. Technically there doesn't need to be
    /// any partials in the simplest of outputs, so this is optional.
    pub theme: Option<PathBuf>,

    /// Static assets folder copied directly to output.
    /// No assets folder means no static assets will be copied.
    pub assets: Option<PathBuf>,

    /// Enable strict mode in the handlebars parser. This causes
    /// missing or unknown values to produce hard errors instead of
    /// empty strings.
    #[serde(default = "default_false")]
    pub strict: bool,
}

fn default_outdir() -> PathBuf {
    "./html".into()
}

const fn default_false() -> bool {
    false
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
            fs::read_to_string(path.as_ref())
                .context(format!("Error opening: \"{}\"", path.as_ref().display()))?,
        )
    }

    pub fn from_str(s: impl AsRef<str>) -> Result<Config> {
        let config: Config = toml::from_str(s.as_ref())?;
        Ok(config)
    }

    pub fn update_from(&mut self, cli: &Cli) {
        match &cli.command {
            crate::cli::Command::Build {
                strict,
                output,
                content,
                theme,
                assets,
                config: _config,
            } => {
                if let Some(is_strict) = strict {
                    self.site.strict = *is_strict;
                }

                if let Some(output_folder) = output.clone() {
                    self.site.out_dir = output_folder;
                }

                if let Some(content_folder) = content.clone() {
                    self.site.content = content_folder;
                }

                if let Some(theme_folder) = theme.clone() {
                    self.site.theme = Some(theme_folder);
                }

                if let Some(assets_folder) = assets.clone() {
                    self.site.assets = Some(assets_folder);
                }
            }
            crate::cli::Command::Serve { strict: Some(is_strict), .. } => self.site.strict = *is_strict,
            _ => {}
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
            "site": value.site,
            "params": value.params,
            "_bundled_normalize": include_str!("normalize.min.css"),
        }))
    }
}

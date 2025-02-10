use color_eyre::eyre::{eyre, Error, Result};
use handlebars::Handlebars;
use tracing::{debug, info};
use std::{io::Write, path::{Path, PathBuf}};
use walkdir::WalkDir;

use crate::config::Config;

pub struct Site<'a> {
    context: serde_json::Value,
    templates: Handlebars<'a>,
    assets: Option<PathBuf>,
    out_dir: PathBuf,
}
impl<'a> TryFrom<&Config> for Site<'a> {
    type Error = Error;
    fn try_from(value: &Config) -> std::result::Result<Self, Self::Error> {
        let mut handlebars = Handlebars::new();

        // Recursively scan the theme folder for .hbs files
        for entry in WalkDir::new(&value.theme) {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("hbs") {
                let template_name = entry
                    .path()
                    .strip_prefix(&value.theme)?
                    .to_str()
                    .ok_or(eyre!("Invalid template path"))?
                    .replace(".hbs", "")
                    .replace(std::path::MAIN_SEPARATOR, "/"); // Normalize path separators

                let template_content = std::fs::read_to_string(entry.path())?;

                // Register index.hbs as a regular template
                if template_name == "index" {
                    handlebars.register_template_string(&template_name, template_content)?;
                } else {
                    // Register all other .hbs files as partials
                    handlebars.register_partial(&template_name, template_content)?;
                }
            }
        }

        Ok(Self {
            context: value.try_into()?,
            templates: handlebars,
            assets: value.assets.clone(),
            out_dir: value.out_dir.clone(),
        })
    }
}

impl<'a> TryFrom<Config> for Site<'a> {
    type Error = Error;
    fn try_from(value: Config) -> std::result::Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl<'a> Site<'a> {
    pub fn new(config: &'a Config) -> Result<Self> {
        config.try_into()
    }

    pub fn render(&self) -> Result<()> {
        // Create the output folder if it doesn't exist
        std::fs::create_dir_all(&self.out_dir)?;

        // Render the main.hbs template into index.html
        info!("Rendering index page...");
        let rendered = self.templates.render("index", &self.context)?;
        let index_html_path = self.out_dir.join("index.html");
        let mut file = std::fs::File::create(index_html_path)?;
        file.write_all(rendered.as_bytes())?;

        // Copy the `assets` folder into the output folder
        if let Some(assets) = self.assets.as_deref() {
            info!("Copying static assets...");
            Self::copy_if_newer(assets, &self.out_dir)?;
        }
        else {
            info!("No assets folder specified! Skipping...");
        }
        Ok(())
    }

    fn copy_if_newer(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
        let src = src.as_ref();
        let dst = dst.as_ref();
        
        for entry in WalkDir::new(src) {
            let entry = entry?;
            let src_path = entry.path();
            let relative_path = src_path.strip_prefix(src)?;
            let dst_path = dst.join(relative_path);
    
            if src_path.is_file() {
                let should_copy = if dst_path.exists() {
                    let src_metadata = std::fs::metadata(src_path)?;
                    let dst_metadata = std::fs::metadata(&dst_path)?;
                    let src_modified = src_metadata.modified()?;
                    let dst_modified = dst_metadata.modified()?;
                    src_modified > dst_modified
                } else {
                    true
                };
    
                if should_copy {
                    if let Some(parent) = dst_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::copy(src_path, &dst_path)?;
                    debug!("Copied: {}", dst_path.display());
                } else {
                    debug!("Skipped (source not newer): {}", dst_path.display());
                }
            } else if src_path.is_dir() && !dst_path.exists() {
                std::fs::create_dir_all(&dst_path)?;
                debug!("Created directory: {}", dst_path.display());
            }
        }
    
        Ok(())
    }
}

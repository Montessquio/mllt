use color_eyre::eyre::{eyre, Result};
use handlebars::{
    BlockContext, BlockParamHolder, Context, Handlebars, Helper, Output, RenderContext, RenderErrorReason, Renderable,
};
use ignore::WalkBuilder;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use tracing::{debug, info};
use walkdir::WalkDir;

use crate::config::Config;

pub struct Site<'a> {
    config: &'a Config,
    context: serde_json::Value,
    templates: Handlebars<'a>,
    assets: Option<PathBuf>,
    out_dir: PathBuf,
}

impl<'a> Site<'a> {
    pub fn new(config: &'a Config) -> Result<Self> {
        let handlebars = {
            let mut handlebars = Handlebars::new();
            handlebars.set_strict_mode(config.site.strict);
            handlebars.register_helper("theme", Box::new(ThemeHelper));
            handlebars
        };

        Ok(Self {
            config,
            context: config.try_into()?,
            templates: handlebars,
            assets: config.site.assets.clone(),
            out_dir: config.site.out_dir.clone(),
        })
    }

    pub fn reload_templates(&mut self) -> Result<()> {
        self.templates.clear_templates();
        self.populate_templates()?;

        Ok(())
    }

    pub fn render(&self) -> Result<()> {
        // Create the output folder if it doesn't exist
        std::fs::create_dir_all(&self.out_dir)?;

        // Render the main.hbs template into index.html
        info!("Rendering content pages...");
        let w = WalkBuilder::new(&self.config.site.content)
            .git_global(false)
            .git_exclude(false)
            .git_ignore(false)
            .ignore(true)
            .parents(true)
            .build();

        // TODO: Parallelize
        for entry in w {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("hbs") {
                let template_name =
                    Self::path_to_template_name(entry.path(), &self.config.site.content)?;

                let final_output_path = self
                    .config
                    .site
                    .out_dir
                    .join(entry.path().strip_prefix(&self.config.site.content)?)
                    .with_extension("html");

                if let Some(parent) = final_output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut file = std::fs::File::create(final_output_path)?;

                let rendered = self
                    .templates
                    .render(template_name.as_str(), &self.context)?;

                file.write_all(rendered.as_bytes())?;
            }
        }

        // Copy the `assets` folder into the output folder
        if let Some(assets) = self.assets.as_deref() {
            info!("Copying static assets...");
            Self::copy_if_newer(assets, &self.out_dir)?;
        } else {
            info!("No assets folder specified! Skipping...");
        }

        Ok(())
    }

    fn populate_templates(&mut self) -> Result<()> {
        // Recursively scan the theme folder for .hbs partials
        // To support page transclusion, also add in .hbs templates
        // from the content directory, too.
        // TODO: Parallelize
        let scan_for_templates = |p: &Path, r: &mut Handlebars| -> Result<usize> {
            let w = WalkBuilder::new(p)
                .git_global(false)
                .git_exclude(false)
                .git_ignore(false)
                .ignore(true)
                .parents(true)
                .build();

            let mut dbg_entry_count = 0usize;
            for entry in w {
                let entry = entry?;
                if entry.path().extension().and_then(|s| s.to_str()) == Some("hbs") {
                    let template_name = Self::path_to_template_name(entry.path(), p)?;

                    r.register_partial(&template_name, std::fs::read_to_string(entry.path())?)?;

                    dbg_entry_count += 1;
                    debug!("Registered template: {template_name}.");
                }
            }

            Ok(dbg_entry_count)
        };

        // Recursively scan the content folder for templates to render.
        if let Some(tp) = &self.config.site.theme {
            let cnt = scan_for_templates(tp, &mut self.templates)?;
            info!(
                "Registered {cnt} theme template{}!",
                if cnt != 1 { "s" } else { "" }
            );
        }
        let cnt = scan_for_templates(&self.config.site.content, &mut self.templates)?;
        info!(
            "Registered {cnt} content template{}!",
            if cnt != 1 { "s" } else { "" }
        );

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

    fn path_to_template_name(
        template_path: impl AsRef<Path>,
        category_path: impl AsRef<Path>,
    ) -> Result<String> {
        let mut template_name = template_path.as_ref();

        if let Some(p) = category_path.as_ref().parent() {
            template_name = template_name.strip_prefix(p)?
        }

        let template_name = template_name
            .to_str()
            .ok_or(eyre!("Invalid template path"))?
            .replace(".hbs", "")
            // Normalize path separators
            .replace(std::path::MAIN_SEPARATOR, "/");

        Ok(template_name)
    }
}

#[derive(Clone, Copy)]
struct ThemeHelper;

impl handlebars::HelperDef for ThemeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        reg: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> handlebars::HelperResult {
        if !h.is_block() {
            return Err(RenderErrorReason::BlockContentRequired.into());
        }

        // Get the name of the partial from the helper's first parameter
        let enclosing_template_name = h
            .param(0)
            .ok_or_else(|| RenderErrorReason::ParamNotFoundForIndex("template", 0))?
            .value()
            .as_str()
            .ok_or_else(|| RenderErrorReason::Other("Template name must be a string".into()))?;

        // Capture the internal content of the block
        let mut internal_content = handlebars::StringOutput::new();
        if let Some(t) = h.template() {
            t.render(reg, ctx, rc, &mut internal_content)?;
        }
        let internal_content = internal_content.into_string()?;

        // Render the substituted context into the final output
        match reg.get_template(enclosing_template_name) {
            None => {
                return Err(RenderErrorReason::Other(format!(
                    "Template '{}' not found!",
                    enclosing_template_name
                ))
                .into())
            }
            Some(t) => {
                let mut bc = BlockContext::new();
                bc.set_block_param("content", BlockParamHolder::Value(handlebars::JsonValue::String(internal_content)));
                rc.push_block(bc);
                rc.set_disable_escape(true);
                let result = t.render(reg, ctx, rc, out);
                rc.set_disable_escape(false);
                rc.pop_block(); // Restore the previous context
                result?;
            }
        };

        Ok(())
    }
}

use color_eyre::eyre::{bail, eyre, Result};
use serde::Serialize;
use std::{fs::File, io::Write, path::Path};
use tracing::warn;

use crate::config::Config;

pub fn instantiate_site(base_path: impl AsRef<Path>, clobber: bool) -> Result<()> {
    let base_path = base_path.as_ref();
    create_project_dir(base_path, clobber)?;
    write_serde_default::<Config>(base_path.join("mllt.toml"), clobber)?;
    create_sample_theme(base_path.join("theme"), clobber)?;
    create_sample_content(base_path.join("content"), clobber)?;
    create_sample_assets(base_path.join("assets"), clobber)?;
    Ok(())
}

fn create_project_dir(project_dir: impl AsRef<Path>, clobber: bool) -> Result<()> {
    let project_dir = project_dir.as_ref();
    
    create_dir_all_checked(project_dir, clobber)?;

    Ok(())
}

fn write_serde_default<T: Default + Serialize>(
    path: impl AsRef<Path>,
    clobber: bool,
) -> Result<()> {
    let path = path.as_ref();

    write_file_checked(path, toml::to_string_pretty(&T::default())?, clobber)?;

    Ok(())
}

fn create_sample_theme(theme_dir: impl AsRef<Path>, clobber: bool) -> Result<()> {
    let theme_dir = theme_dir.as_ref();
    
    create_dir_all_checked(theme_dir, clobber)?;

    write_file_checked(theme_dir.join("style.hbs").as_path(), include_str!("./style.hbs"), clobber)?;
    write_file_checked(theme_dir.join("header.hbs").as_path(), include_str!("./header.hbs"), clobber)?;
    write_file_checked(theme_dir.join("head.hbs").as_path(), include_str!("./head.hbs"), clobber)?;
    write_file_checked(theme_dir.join("footer.hbs").as_path(), include_str!("./footer.hbs"), clobber)?;
    write_file_checked(theme_dir.join("page.hbs").as_path(), include_str!("./page.hbs"), clobber)?;

    Ok(())
}

fn create_sample_content(content_dir: impl AsRef<Path>, clobber: bool) -> Result<()> {
    let content_dir = content_dir.as_ref();
    create_dir_all_checked(content_dir, clobber)?;

    write_file_checked(content_dir.join("index.hbs").as_path(), include_str!("./index.hbs"), clobber)?;

    Ok(())
}

fn create_sample_assets(assets_dir: impl AsRef<Path>, clobber: bool) -> Result<()> {
    let assets_dir = assets_dir.as_ref();
    
    create_dir_all_checked(assets_dir, clobber)?;

    Ok(())
}

fn create_dir_all_checked(path: impl AsRef<Path>, clobber: bool) -> Result<()> {
    let path = path.as_ref();

    if path.is_file() {
        return Err(eyre!("'{}' is a file.", path.display()));
    }

    if path.is_dir() {
        let is_empty = std::fs::read_dir(path)?.next().is_none();

        match (is_empty, clobber) {
            // Non empty and clobber means proceed with file creation,
            // but skip dir creation
            (false, true) => return Ok(()),
            // Non empty and no clobber means error out.
            (false, false) => return Err(eyre!(
                "Project directory '{}' is non-empty. To clobber existing files, use `--force`.",
                path.display()
            )),
            // Empty and existing means proceed
            (true, _) => return Ok(())
        }
    }

    // Path is not a file nor a directory.
    if path.exists() {
        return Err(eyre!("'{}' exists, unidentified record type.", path.display()))
    }

    // Path does not exist at all.
    std::fs::create_dir_all(path)?;

    Ok(())
}

fn write_file_checked(
    path: impl AsRef<Path>,
    content: impl AsRef<str>,
    clobber: bool,
) -> Result<()> {
    let path = path.as_ref();

    // Check if the file already exists
    if path.exists() {
        if clobber {
            warn!(
                "File already exists and will be overwritten: {}",
                path.display()
            );
        } else {
            bail!("File already exists: {}", path.display());
        }
    }
    write!(File::create(path)?, "{}", content.as_ref())?;
    Ok(())
}

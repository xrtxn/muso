use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

#[inline]
pub fn default_config_path() -> PathBuf {
    dirs::config_dir().unwrap().join("musso/config.toml")
}

#[inline]
pub fn default_service_path() -> PathBuf {
    dirs::config_dir()
        .unwrap()
        .join("systemd/user/musso.service")
}

pub fn maybe_create_dir(path: impl AsRef<Path>) -> std::io::Result<()> {
    match fs::create_dir_all(path) {
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(e),
        },
        Ok(_) => Ok(()),
    }
}

pub enum Resource {
    Config,
    Service,
}

pub fn generate_resource(res: Resource, default: Option<&str>) -> Result<()> {
    let name = match res {
        Resource::Config => "config",
        Resource::Service => "service",
    };

    let dest = match res {
        Resource::Config => default_config_path(),
        Resource::Service => default_service_path(),
    };

    log::info!("Generating {} file", name);

    let shared = match res {
        Resource::Config => Path::new("/usr/share/musso/config.toml"),
        Resource::Service => Path::new("/usr/share/musso/musso.service"),
    };

    if !shared.exists() {
        if let Some(default) = default {
            let mut file = File::create(&dest)?;
            write!(file, "{}", default)?;
            log::info!("Successfully written to: \"{}\"", dest.to_string_lossy());
        } else {
            return Err(Error::ResourceNotFound {
                path: shared.to_string_lossy().into(),
            });
        }
    } else {
        log::info!("Copying {} file from shared assets", name);

        let parent = dest.parent().ok_or(Error::InvalidParent {
            child: dest.to_string_lossy().into(),
        })?;

        maybe_create_dir(parent)?;
        fs::copy(shared, &dest)?;

        log::info!("Successfully copied to: \"{}\"", dest.to_string_lossy());
    }

    log::info!("Successfully generated {} file", name);
    Ok(())
}

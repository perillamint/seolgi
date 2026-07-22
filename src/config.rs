use crate::backend::landlock::LandlockAccessEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(feature = "landlock")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LandlockRule {
    pub path: PathBuf,
    pub rules: Vec<LandlockAccessEnum>,
}

#[cfg(feature = "landlock")]
impl LandlockRule {
    fn expand_path(&mut self) -> anyhow::Result<()> {
        self.path = expand_path(&self.path)?;
        Ok(())
    }
}

#[cfg(feature = "landlock")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LandlockConfig {
    pub rules: Vec<LandlockRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[cfg(feature = "landlock")]
    pub landlock: LandlockConfig,
}

impl Config {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let resolved = expand_path(path)?;
        let content = std::fs::read_to_string(&resolved)?;
        let mut config: Config = yaml_serde::from_str(&content)?;

        #[cfg(feature = "landlock")]
        for rule in &mut config.landlock.rules {
            rule.expand_path()?;
        }

        Ok(config)
    }
}

fn expand_path(path: &PathBuf) -> anyhow::Result<PathBuf> {
    let mut path_new = PathBuf::from(path);
    if let Some(rest) = path_new.to_string_lossy().strip_prefix("~/") {
        let home = std::env::var("HOME")?;
        path_new = PathBuf::from(home).join(rest);
    }

    if path_new.to_string_lossy().find("${workspace}").is_some() {
        let workspace = std::env::current_dir()?;
        path_new = PathBuf::from(
            path_new
                .to_string_lossy()
                .replace("${workspace}", &workspace.to_string_lossy()),
        );
    }

    Ok(path_new)
}

use std::os::unix::prelude::*;
use std::process::Command;

use landlock::{
    AccessFs, BitFlags, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreated,
    RulesetCreatedAttr,
};
use serde::{Deserialize, Serialize};

use crate::config::LandlockConfig;

macro_rules! define_landlock_access {
    ($($variant:ident => $access:ident),* $(,)?) => {
        #[derive(Clone, Debug, Serialize, Deserialize, strum::Display)]
        #[serde(rename_all = "snake_case")]
        #[strum(serialize_all = "snake_case")]
        #[allow(clippy::enum_variant_names)]
        pub enum LandlockAccessEnum {
            $($variant),*
        }

        impl LandlockAccessEnum {
            pub fn to_access_fs(&self) -> AccessFs {
                match self {
                    $(LandlockAccessEnum::$variant => AccessFs::$access),*
                }
            }

            pub fn all_access_fs() -> BitFlags<AccessFs> {
                $(AccessFs::$access)|*
            }
        }
    };
}

define_landlock_access! {
    FsExecute => Execute,
    FsWriteFile => WriteFile,
    FsReadFile => ReadFile,
    FsTruncate => Truncate,
    FsReadDir => ReadDir,
    FsRemoveDir => RemoveDir,
    FsRemoveFile => RemoveFile,
    FsMakeChar => MakeChar,
    FsMakeDir => MakeDir,
    FsMakeReg => MakeReg,
    FsMakeSock => MakeSock,
    FsMakeFifo => MakeFifo,
    FsMakeSym => MakeSym,
    FsRefer => Refer,
    FsIoctlDev => IoctlDev,
}

pub struct LandlockBackend {
    config: LandlockConfig,
}

impl LandlockBackend {
    pub fn new(config: LandlockConfig) -> Self {
        Self { config }
    }

    fn build_ruleset(&self) -> anyhow::Result<RulesetCreated> {
        tracing::debug!(
            "Applying landlock restrictions on: {:?}",
            LandlockAccessEnum::all_access_fs()
        );
        let mut ruleset = Ruleset::default()
            .handle_access(LandlockAccessEnum::all_access_fs())
            .and_then(|ruleset| ruleset.create())?;

        for rule in &self.config.rules {
            match PathFd::new(&rule.path) {
                Ok(fd) => {
                    // Apply rules from configuration
                    let flags = rule
                        .rules
                        .iter()
                        .map(|e| BitFlags::from(e.to_access_fs()))
                        .reduce(|access_fs, rule| access_fs | rule)
                        .unwrap_or_default();

                    tracing::debug!("Adding rule for path: {:?}, flags: {:?}", rule.path, flags);

                    ruleset = ruleset.add_rule(PathBeneath::new(fd, flags))?;
                }
                Err(e) => {
                    tracing::warn!("Failed to open path: {:?}, error: {:?}", rule.path, e);
                }
            }
        }

        Ok(ruleset)
    }

    pub fn sandbox_cmd(&self, cmd: &mut Command) -> anyhow::Result<()> {
        let mut ruleset = Some(self.build_ruleset()?);

        unsafe {
            cmd.pre_exec(move || {
                let rs = ruleset
                    .take()
                    .expect("landlock: ruleset must not be consumed twice");
                rs.restrict_self()
                    .unwrap_or_else(|_| panic!("landlock: restrict_self failed"));
                Ok(())
            });
        }

        Ok(())
    }
}

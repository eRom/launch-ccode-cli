//! Proxy LiteLLM autonome — install, lifecycle, génération de config.
//!
//! Voir docs/superpowers/specs/2026-05-15-lcc-proxy-autonome-design.md

pub mod cli;
pub mod generators;
pub mod health;
pub mod install;
pub mod keychain;
pub mod launchctl;
pub mod lifecycle;
pub mod provider_map;
pub mod reload;
pub mod status;
pub mod uninstall;

use std::path::PathBuf;

/// Identifiant launchd du daemon.
pub const LAUNCHD_LABEL: &str = "com.lcc.litellm";

/// Slug Keychain pour la master_key.
pub const KEYCHAIN_SERVICE: &str = "lcc.litellm.master_key";

/// Port par défaut du proxy.
pub const DEFAULT_PORT: u16 = 4000;

/// Path du yaml généré.
pub fn yaml_path() -> PathBuf {
    dirs::config_dir()
        .expect("config_dir introuvable")
        .join("launch-claude-code")
        .join("litellm.yaml")
}

/// Path du plist LaunchAgent.
pub fn plist_path() -> PathBuf {
    dirs::home_dir()
        .expect("home_dir introuvable")
        .join("Library/LaunchAgents")
        .join(format!("{LAUNCHD_LABEL}.plist"))
}

/// Path du wrapper bash.
pub fn wrapper_path() -> PathBuf {
    dirs::data_local_dir()
        .expect("data_local_dir introuvable")
        .join("lcc")
        .join("lcc-litellm-launcher.sh")
}

/// Dossier de logs.
pub fn logs_dir() -> PathBuf {
    dirs::home_dir()
        .expect("home_dir introuvable")
        .join("Library/Logs/lcc")
}

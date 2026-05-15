//! Subcommand `lcc proxy *` — dispatch.

use clap::Subcommand;
use std::io;

use crate::proxy::{install, lifecycle, reload, status, uninstall};

#[derive(Subcommand, Debug)]
pub enum Proxy {
    /// Setup complet du proxy LiteLLM (uv install + plist + keychain + load).
    Install,
    /// Désinstalle le proxy (unload + supprime plist).
    Uninstall {
        /// Supprime aussi le yaml, le venv litellm et la master_key.
        #[arg(long)]
        purge: bool,
    },
    /// Régénère le yaml depuis settings.json + redémarre le daemon.
    Reload,
    /// Démarre le LaunchAgent.
    Start,
    /// Arrête le LaunchAgent.
    Stop,
    /// Redémarre le LaunchAgent (kickstart -k).
    Restart,
    /// Affiche l'état du proxy (PID, port, modèles routés).
    Status,
    /// Tail des logs du daemon.
    Logs {
        /// Suivre en continu (-f).
        #[arg(short = 'f', long)]
        follow: bool,
    },
    /// Diagnostic complet (uv ? plist ? daemon ? port ? yaml ?).
    Doctor,
}

pub fn dispatch(cmd: Proxy) -> io::Result<()> {
    match cmd {
        Proxy::Install => install::run_install(),
        Proxy::Uninstall { purge } => uninstall::run_uninstall(purge),
        Proxy::Reload => reload::run_reload(),
        Proxy::Start => lifecycle::start(),
        Proxy::Stop => lifecycle::stop(),
        Proxy::Restart => lifecycle::restart(),
        Proxy::Status => status::run_status(),
        Proxy::Logs { follow } => status::run_logs(follow),
        Proxy::Doctor => status::run_doctor(),
    }
}

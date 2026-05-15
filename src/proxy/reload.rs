//! `lcc proxy reload` — régénère le yaml depuis settings.json + restart.

use crate::proxy::{install, launchctl};
use std::io;

pub fn run_reload() -> io::Result<()> {
    println!("=== lcc proxy reload ===\n");
    install::write_yaml()?;
    println!("→ launchctl kickstart -k…");
    launchctl::kickstart()?;
    println!("✓ proxy rechargé.");
    Ok(())
}

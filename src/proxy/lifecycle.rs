//! Sous-commandes simples : start / stop / restart.

use crate::proxy::launchctl;
use std::io;

pub fn start() -> io::Result<()> {
    println!("→ launchctl load…");
    launchctl::load()?;
    println!("✓ proxy lancé.");
    Ok(())
}

pub fn stop() -> io::Result<()> {
    println!("→ launchctl unload…");
    launchctl::unload()?;
    println!("✓ proxy arrêté.");
    Ok(())
}

pub fn restart() -> io::Result<()> {
    println!("→ launchctl kickstart -k…");
    launchctl::kickstart()?;
    println!("✓ proxy redémarré.");
    Ok(())
}

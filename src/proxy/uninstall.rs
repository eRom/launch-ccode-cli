//! `lcc proxy uninstall [--purge]` — symétrique de `install`. (Task 5.1)

use std::io;

pub fn run_uninstall(_purge: bool) -> io::Result<()> {
    eprintln!("⚠ uninstall pas encore implémenté (Task 5.1)");
    Ok(())
}

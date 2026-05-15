//! `lcc proxy uninstall [--purge]` — symétrique de `install`.

use crate::proxy::{keychain, launchctl, plist_path, wrapper_path, yaml_path};
use std::fs;
use std::io;
use std::process::Command;

pub fn run_uninstall(purge: bool) -> io::Result<()> {
    println!("=== lcc proxy uninstall ===\n");

    println!("→ launchctl unload…");
    let _ = launchctl::unload();

    let plist = plist_path();
    if plist.exists() {
        fs::remove_file(&plist)?;
        println!("✓ plist supprimé : {}", plist.display());
    }

    let wrapper = wrapper_path();
    if wrapper.exists() {
        fs::remove_file(&wrapper)?;
        println!("✓ wrapper supprimé");
    }

    if purge {
        let yaml = yaml_path();
        if yaml.exists() {
            fs::remove_file(&yaml)?;
            println!("✓ litellm.yaml supprimé (--purge)");
        }
        let _ = keychain::delete_master_key();
        println!("✓ master_key supprimée du Keychain (--purge)");

        println!("→ uv tool uninstall litellm…");
        let _ = Command::new("uv")
            .args(["tool", "uninstall", "litellm"])
            .status();
        println!("✓ litellm désinstallé (--purge)");
    } else {
        println!(
            "ℹ litellm venv, yaml et master_key conservés. Utilise --purge pour tout effacer."
        );
    }

    println!("\n✓ uninstall terminé.");
    Ok(())
}

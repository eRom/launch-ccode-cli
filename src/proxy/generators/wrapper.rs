//! Génère le wrapper bash lancé par launchd. Le wrapper récupère la
//! master_key depuis le Keychain puis exec litellm avec la bonne config.

use crate::proxy::KEYCHAIN_SERVICE;

pub fn generate_wrapper(litellm_bin: &str, yaml_path: &str, port: u16) -> String {
    format!(
        r#"#!/bin/bash
# Wrapper généré par `lcc proxy install`. NE PAS ÉDITER À LA MAIN.
# Re-générer via `lcc proxy reload` après changement de config.

set -euo pipefail

# Récupération de la master_key depuis le Keychain macOS.
# Échoue net si la clé n'existe pas (lcc proxy install non lancé).
LCC_MASTER_KEY="$(security find-generic-password -s {KEYCHAIN_SERVICE} -w)"
export LCC_MASTER_KEY

# S'assure que le PYTHONPATH inclut le site-packages du venv pour le callback custom.
exec {litellm_bin} --config {yaml_path} --port {port}
"#
    )
}

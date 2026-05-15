//! Génère le plist LaunchAgent pour le daemon LiteLLM.
//!
//! Format Apple plist XML (cf `man launchd.plist`).

use crate::proxy::LAUNCHD_LABEL;
use std::collections::HashMap;

pub fn generate_plist(
    wrapper_path: &str,
    stdout_log: &str,
    stderr_log: &str,
    env_vars: &HashMap<String, String>,
) -> String {
    let mut env_block = String::new();
    // Sort for deterministic output
    let mut keys: Vec<&String> = env_vars.keys().collect();
    keys.sort();
    for k in keys {
        env_block.push_str(&format!(
            "        <key>{}</key>\n        <string>{}</string>\n",
            xml_escape(k),
            xml_escape(&env_vars[k]),
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LAUNCHD_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}</string>
    <key>StandardErrorPath</key>
    <string>{}</string>
    <key>EnvironmentVariables</key>
    <dict>
{}    </dict>
</dict>
</plist>
"#,
        xml_escape(wrapper_path),
        xml_escape(stdout_log),
        xml_escape(stderr_log),
        env_block,
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

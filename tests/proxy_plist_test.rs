use lcc::proxy::generators::plist::generate_plist;
use std::collections::HashMap;

#[test]
fn plist_contains_required_keys() {
    let mut env = HashMap::new();
    env.insert("OPENROUTER_API_KEY".to_string(), "sk-or-v1-xxx".to_string());

    let plist = generate_plist(
        "/Users/test/.local/share/lcc/lcc-litellm-launcher.sh",
        "/Users/test/Library/Logs/lcc/litellm.out.log",
        "/Users/test/Library/Logs/lcc/litellm.err.log",
        &env,
    );

    assert!(plist.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(plist.contains("<key>Label</key>"));
    assert!(plist.contains("<string>com.lcc.litellm</string>"));
    assert!(plist.contains("<key>ProgramArguments</key>"));
    assert!(plist.contains("/Users/test/.local/share/lcc/lcc-litellm-launcher.sh"));
    assert!(plist.contains("<key>RunAtLoad</key>"));
    assert!(plist.contains("<true/>"));
    assert!(plist.contains("<key>KeepAlive</key>"));
    assert!(plist.contains("<key>StandardOutPath</key>"));
    assert!(plist.contains("/litellm.out.log"));
    assert!(plist.contains("<key>StandardErrorPath</key>"));
    assert!(plist.contains("<key>EnvironmentVariables</key>"));
    assert!(plist.contains("<key>OPENROUTER_API_KEY</key>"));
    assert!(plist.contains("<string>sk-or-v1-xxx</string>"));
}

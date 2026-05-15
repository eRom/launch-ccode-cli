use lcc::proxy::generators::wrapper::generate_wrapper;

#[test]
fn wrapper_loads_master_key_then_execs_litellm() {
    let wrapper = generate_wrapper(
        "/Users/test/.local/share/lcc/litellm-venv/bin/litellm",
        "/Users/test/.config/launch-claude-code/litellm.yaml",
        4000,
    );

    assert!(wrapper.starts_with("#!/bin/bash"));
    assert!(wrapper.contains("set -euo pipefail"));
    assert!(wrapper.contains(
        r#"LCC_MASTER_KEY="$(security find-generic-password -s lcc.litellm.master_key -w)""#
    ));
    assert!(wrapper.contains("export LCC_MASTER_KEY"));
    assert!(wrapper.contains("unset DATABASE_URL"));
    assert!(wrapper.contains(
        "exec \"/Users/test/.local/share/lcc/litellm-venv/bin/litellm\""
    ));
    assert!(wrapper.contains(
        "--config \"/Users/test/.config/launch-claude-code/litellm.yaml\""
    ));
    assert!(wrapper.contains("--port 4000"));
}

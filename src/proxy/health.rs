//! Health check HTTP du daemon LiteLLM (`GET /health/liveness`).

use std::time::Duration;

/// Retourne `true` si le proxy répond 200 sur `/health/liveness`.
/// Utilise `ureq` (sync, léger).
pub fn is_alive(port: u16, timeout: Duration) -> bool {
    let url = format!("http://127.0.0.1:{port}/health/liveness");
    let agent = ureq::AgentBuilder::new()
        .timeout(timeout)
        .build();
    match agent.get(&url).call() {
        Ok(resp) => resp.status() == 200,
        Err(_) => false,
    }
}

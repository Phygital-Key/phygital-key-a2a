//! `phygital-key-a2a`: Phygital Key for A2A. Agents ask through the A2A
//! protocol; a person approves the exact action with a passkey, fingerprint or
//! hardware key; the service that acts checks the signed approval itself, offline.
//!
//! The gate is an A2A "approval agent". It can never approve: only a person's key
//! signs, on the approval page. Everything but the A2A side comes from the
//! shared base, `phygital-key-gate`.

#![forbid(unsafe_code)]

mod card;
mod intent;
mod proto;
mod rpc;
mod server;
mod store;
#[cfg(feature = "tck")]
mod tck;
mod time;

use phygital_key_gate::{Adapter, Product, Settings, Shared};

pub(crate) static PRODUCT: Product = Product {
    name: "Phygital Key A2A",
    bin: "phygital-key-a2a",
    id: "a2a",
    version: env!("CARGO_PKG_VERSION"),
    repo: "Phygital-Key/phygital-key-a2a",
    port: 7478,
    about: "A person approves exact AI actions with a passkey or hardware key. A2A edition.",
    release_keys: include_str!("../release-keys.pem"),
};

struct A2a;

impl Adapter for A2a {
    fn path(&self) -> &'static str {
        "/a2a"
    }

    fn router(&self, gate: Shared) -> axum::Router {
        server::router(gate)
    }

    fn public_router(&self, gate: Shared) -> Option<axum::Router> {
        Some(card::router(gate))
    }

    fn connect_hint(&self, s: &Settings) -> String {
        hint(s)
    }

    fn demo_hint(&self) -> &'static str {
        "Point any A2A client at the agent card above. Send a message with one data part \
         {\"request\": {\"request\": {\"type\": \"deploy\", \"params\": {\"service\": \"api\", \"env\": \"production\", \"version\": \"2.4.1\"}}, \"aud\": \"<demo service address>\", \"level\": \"high\"}}, \
         open the approveUrl from the task to decide, then POST the params to <demo service>/deploy with the header Phygital-Approval: <approval from the task's artifact>."
    }
}

fn hint(s: &Settings) -> String {
    let o = &s.origin;
    let card = format!(
        "  agent card      {o}{}  (public, no token)",
        card::CARD_PATH
    );
    #[cfg(feature = "tck")]
    let card = format!(
        "  WARNING         conformance-test build (--features tck): never use it for real approvals\n{card}"
    );
    match &s.access_token {
        Some(t) => format!(
            "{card}\n  A2A endpoint    {o}/a2a  (JSON-RPC, A2A 1.0; needs headers  Authorization: Bearer {t}  and  A2A-Version: 1.0)\n  Try it:         curl -s {o}/a2a -H \"Authorization: Bearer {t}\" -H \"A2A-Version: 1.0\" -H \"Content-Type: application/json\" -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ListTasks\",\"params\":{{}}}}'"
        ),
        None => format!(
            "{card}\n  A2A endpoint    {o}/a2a  (JSON-RPC, A2A 1.0, open: no access token; needs header  A2A-Version: 1.0)"
        ),
    }
}

/// Refused in a release: with no access token, any process on this machine
/// could list tasks and read approval strings and standing ids, and sign uses.
const NO_TOKEN_REFUSED: &str = "phygital-key-a2a: --no-token is not allowed: anyone on this machine could read approvals. Use the access token (phygital-key-a2a token).";

/// True when the arguments ask to serve without the access token.
fn asks_no_token<I: IntoIterator<Item = std::ffi::OsString>>(args: I) -> bool {
    args.into_iter().any(|a| {
        a.to_str()
            .is_some_and(|a| a == "--no-token" || a.starts_with("--no-token="))
    })
}

fn main() {
    // The conformance kit runs against a `tck` build with --no-token; a
    // release never serves without the token.
    if !cfg!(feature = "tck") && asks_no_token(std::env::args_os().skip(1)) {
        eprintln!("{NO_TOKEN_REFUSED}");
        std::process::exit(2);
    }
    phygital_key_gate::run(&PRODUCT, A2a)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)] // serde_json `v["key"]` in test asserts
mod tests {
    use super::*;

    fn settings(token: Option<&str>) -> Settings {
        Settings {
            home: std::path::PathBuf::from("/nonexistent"),
            bind: "127.0.0.1".into(),
            port: 7478,
            origin: "http://localhost:7478".into(),
            rp_id: "localhost".into(),
            lifetime: 60,
            access_token: token.map(String::from),
            agent_path: "/a2a",
        }
    }

    #[test]
    fn connect_hint_names_card_endpoint_and_token() {
        let t = "A".repeat(43);
        let h = hint(&settings(Some(&t)));
        assert!(h.contains("http://localhost:7478/.well-known/agent-card.json"));
        assert!(h.contains("http://localhost:7478/a2a"));
        assert!(h.contains(&format!("Authorization: Bearer {t}")));
        assert!(h.contains("A2A-Version: 1.0"));
        assert!(h.contains("curl -s http://localhost:7478/a2a"));
        let open = hint(&settings(None));
        assert!(!open.contains("Bearer"));
    }

    #[test]
    fn no_token_is_spotted() {
        let a = |v: &[&str]| asks_no_token(v.iter().map(std::ffi::OsString::from));
        assert!(a(&["serve", "--no-token"]));
        assert!(a(&["--no-token", "serve"]));
        assert!(a(&["--home", "/x", "demo", "--no-token=true"]));
        assert!(!a(&["serve"]));
        assert!(!a(&["serve", "--no-tokens", "--token"]));
        assert!(NO_TOKEN_REFUSED.contains("--no-token is not allowed"));
    }

    #[test]
    fn product_identity() {
        assert_eq!(PRODUCT.bin, "phygital-key-a2a");
        assert_eq!(PRODUCT.id, "a2a");
        assert_eq!(PRODUCT.port, 7478);
        assert_eq!(PRODUCT.env("HOME"), "PHYGITAL_KEY_A2A_HOME");
    }
}

//! The public agent card at `/.well-known/agent-card.json` (spec 8.2). No token
//! needed, no secrets in it.

use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use phygital_key_gate::Shared;
use serde_json::{Value, json};

use crate::proto::EXT;

/// Where the card is served.
pub const CARD_PATH: &str = "/.well-known/agent-card.json";

fn skill(id: &str, name: &str, description: &str, tags: &[&str], example: Value) -> Value {
    json!({
        "id": id,
        "name": name,
        "description": description,
        "tags": tags,
        "examples": [example.to_string()],
        "inputModes": ["application/json"],
        "outputModes": ["application/json", "text/plain"],
    })
}

/// The agent card for a gate whose page is at `origin`.
pub fn card(origin: &str) -> Value {
    let p = &crate::PRODUCT;
    json!({
        "name": p.name,
        "description": "An approval agent: before a risky action, it asks a person to approve that exact action with their passkey, fingerprint or hardware key. It can never approve by itself, and text never counts as approval. Tasks wait in TASK_STATE_AUTH_REQUIRED while the person decides on the approval page; an approved task completes with the signed approval as its artifact, which the service that acts checks itself, offline.",
        "supportedInterfaces": [
            {"url": format!("{origin}/a2a"), "protocolBinding": "JSONRPC", "protocolVersion": "1.0"}
        ],
        "provider": {"organization": "Phygital-Key", "url": "https://github.com/Phygital-Key"},
        "version": p.version,
        "documentationUrl": format!("https://github.com/{}", p.repo),
        "capabilities": {
            "streaming": true,
            "pushNotifications": false,
            "extendedAgentCard": false,
            "extensions": [{
                "uri": EXT,
                "description": "Phygital Key approvals: the message shapes, the task states, the approval artifact, and how to carry an approval to other agents in metadata under this URI.",
                "required": false,
            }],
        },
        "securitySchemes": {
            "bearer": {"httpAuthSecurityScheme": {"scheme": "Bearer", "description": "The gate's access token (run `phygital-key-a2a token`)."}}
        },
        "securityRequirements": [{"schemes": {"bearer": {"list": []}}}],
        "defaultInputModes": ["application/json", "text/plain"],
        "defaultOutputModes": ["application/json", "text/plain"],
        "skills": [
            skill(
                "request_approval",
                "Request approval",
                "Ask a person to approve one exact action. Send one data part {\"request\": {\"request\": {\"type\", \"params\"}, \"aud\", \"level\": \"medium\" or \"high\"}}. The task waits in TASK_STATE_AUTH_REQUIRED with the approval page link; approved, it completes with the signed approval (works once, for that service, for 60 seconds by default); denied, it is rejected.",
                &["approval", "human-in-the-loop", "passkey", "webauthn"],
                json!({"request": {"request": {"type": "deploy", "params": {"service": "api", "env": "production", "version": "2.4.1"}}, "aud": "https://deploy.example.com", "level": "high"}}),
            ),
            skill(
                "request_standing_approval",
                "Request standing approval",
                "Ask a person once for the same kind of action, within fixed limits, a few times, for a few days. Send {\"standing\": {\"for_type\", \"fixed\", \"vary\", \"max_uses\", \"days\", \"aud\"}}. Approved, the task completes with the standing approval as its artifact.",
                &["approval", "standing", "limits"],
                json!({"standing": {"for_type": "payment", "fixed": {"to": "ACME Ltd", "currency": "EUR"}, "vary": {"amount_cents": {"min": 1, "max": 2000}}, "max_uses": 5, "days": 7, "aud": "https://pay.example.com"}}),
            ),
            skill(
                "use_standing_approval",
                "Use standing approval",
                "Sign one use of an approved standing approval for one exact request inside its limits. Send {\"use\": {\"standingId\", \"request\"}}. The reply message holds approval and useToken; send both to the service.",
                &["approval", "standing"],
                json!({"use": {"standingId": "<task id>", "request": {"type": "payment", "params": {"to": "ACME Ltd", "currency": "EUR", "amount_cents": 450}}}}),
            ),
            skill(
                "record_receipt",
                "Record receipt",
                "After acting, pass back the service's signed receipt, so the person sees it was done. Send {\"receipt\": {\"id\", \"receipt\"}}.",
                &["receipt", "audit"],
                json!({"receipt": {"id": "<task id>", "receipt": "<receipt>"}}),
            ),
            skill(
                "revoke_standing_approval",
                "Revoke standing approval",
                "Stop a standing approval at once. Send {\"revoke\": {\"standingId\"}}. The reply holds signUrl: ask the person to sign the revocation there, so services refuse it too.",
                &["revoke", "standing"],
                json!({"revoke": {"standingId": "<task id>"}}),
            ),
        ],
    })
}

/// A strong validator for the card: FNV-1a 64 of its bytes.
fn etag(body: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in body.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("\"{h:016x}\"")
}

/// An HTTP date (IMF-fixdate) for Unix seconds.
fn http_date(secs: i64) -> String {
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let ymd = phygital_key_gate::util::date(secs);
    let mut it = ymd.split('-');
    let (y, m, d) = (
        it.next().unwrap_or("1970"),
        it.next().unwrap_or("01"),
        it.next().unwrap_or("01"),
    );
    let month = m
        .parse::<usize>()
        .ok()
        .and_then(|m| MONTHS.get(m.wrapping_sub(1)))
        .unwrap_or(&"Jan");
    let weekday = usize::try_from((days + 4).rem_euclid(7))
        .ok()
        .and_then(|w| DAYS.get(w))
        .unwrap_or(&"Thu");
    format!(
        "{weekday}, {d} {month} {y} {:02}:{:02}:{:02} GMT",
        sod / 3600,
        sod % 3600 / 60,
        sod % 60
    )
}

struct CardState {
    gate: Shared,
    /// When this process started serving the card (it cannot change before a restart).
    since: String,
}

async fn serve_card(State(st): State<Arc<CardState>>, headers: HeaderMap) -> Response {
    let body = card(&st.gate.settings.origin).to_string();
    let tag = etag(&body);
    let mut h = HeaderMap::new();
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300"),
    );
    if let Ok(v) = HeaderValue::from_str(&tag) {
        h.insert(header::ETAG, v);
    }
    if let Ok(v) = HeaderValue::from_str(&st.since) {
        h.insert(header::LAST_MODIFIED, v);
    }
    // Public deployments let any site read the card; on localhost, no CORS
    // header (like the key list), so web pages cannot probe a local gate.
    if st.gate.settings.rp_id != "localhost" {
        h.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        );
    }
    let fresh = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.split(',').any(|t| t.trim() == tag || t.trim() == "*"));
    if fresh {
        return (StatusCode::NOT_MODIFIED, h).into_response();
    }
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    (h, body).into_response()
}

/// The public routes.
pub fn router(gate: Shared) -> Router {
    let st = Arc::new(CardState {
        gate,
        since: http_date(phygital_key_gate::util::now()),
    });
    Router::new()
        .route(CARD_PATH, get(serve_card))
        .with_state(st)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)] // serde_json `v["key"]` in test asserts
mod tests {
    use super::*;

    #[test]
    fn validators() {
        assert_eq!(http_date(0), "Thu, 01 Jan 1970 00:00:00 GMT");
        assert_eq!(http_date(1_792_229_400), "Sat, 17 Oct 2026 09:30:00 GMT");
        assert_eq!(etag("a"), etag("a"));
        assert_ne!(etag("a"), etag("b"));
        assert!(etag("x").starts_with('"') && etag("x").len() == 18);
    }

    #[test]
    fn card_has_every_required_field() {
        let c = card("http://localhost:7478");
        for f in [
            "name",
            "description",
            "supportedInterfaces",
            "version",
            "capabilities",
            "defaultInputModes",
            "defaultOutputModes",
            "skills",
        ] {
            assert!(c.get(f).is_some(), "{f}");
        }
        assert_eq!(c["name"], "Phygital Key A2A");
        assert_eq!(
            c["supportedInterfaces"][0]["url"],
            "http://localhost:7478/a2a"
        );
        assert_eq!(c["supportedInterfaces"][0]["protocolBinding"], "JSONRPC");
        assert_eq!(c["supportedInterfaces"][0]["protocolVersion"], "1.0");
        assert_eq!(c["capabilities"]["streaming"], true);
        assert_eq!(c["capabilities"]["pushNotifications"], false);
        assert_eq!(c["capabilities"]["extendedAgentCard"], false);
        assert_eq!(c["capabilities"]["extensions"][0]["uri"], EXT);
        assert_eq!(c["capabilities"]["extensions"][0]["required"], false);
        assert_eq!(
            c["securitySchemes"]["bearer"]["httpAuthSecurityScheme"]["scheme"],
            "Bearer"
        );
        assert_eq!(
            c["securityRequirements"][0]["schemes"]["bearer"]["list"],
            json!([])
        );
        assert_eq!(c["provider"]["organization"], "Phygital-Key");
        let ids: Vec<&str> = c["skills"]
            .as_array()
            .map(|s| s.iter().filter_map(|k| k["id"].as_str()).collect())
            .unwrap_or_default();
        assert_eq!(
            ids,
            [
                "request_approval",
                "request_standing_approval",
                "use_standing_approval",
                "record_receipt",
                "revoke_standing_approval"
            ]
        );
        for s in c["skills"].as_array().into_iter().flatten() {
            for f in ["id", "name", "description", "tags", "examples"] {
                assert!(s.get(f).is_some(), "skill {f}");
            }
            // Examples are strings holding a data part object.
            let ex = s["examples"][0].as_str().unwrap_or_default();
            assert!(serde_json::from_str::<Value>(ex).is_ok_and(|v| v.is_object()));
        }
        // Nothing secret, no kind discriminators (A2A 1.0 has none).
        let text = c.to_string();
        assert!(!text.contains("\"kind\""));
        assert!(!text.to_lowercase().contains("token\":"));
    }
}

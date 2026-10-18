//! The JSON-RPC 2.0 envelope and the A2A error codes (spec sections 5.4 and 9.5).

use serde_json::{Value, json};

/// One JSON-RPC error. A2A errors carry a `google.rpc.ErrorInfo` in `data`.
#[derive(Debug, Clone, PartialEq)]
pub struct RpcError {
    /// JSON-RPC error code.
    pub code: i64,
    /// Human-readable message.
    pub message: String,
    /// `ErrorInfo.reason` for A2A errors (UPPER_SNAKE_CASE), none for plain JSON-RPC errors.
    pub reason: Option<&'static str>,
    /// `ErrorInfo.metadata`.
    pub metadata: Option<Value>,
}

impl RpcError {
    fn plain(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            reason: None,
            metadata: None,
        }
    }

    fn a2a(code: i64, reason: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            reason: Some(reason),
            metadata: None,
        }
    }

    fn with(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// -32700: the body is not JSON.
    pub fn parse_error() -> Self {
        Self::plain(-32700, "Parse error: the body is not valid JSON")
    }
    /// -32600: not a valid JSON-RPC 2.0 request.
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::plain(-32600, message)
    }
    /// -32601.
    pub fn method_not_found(method: &str) -> Self {
        Self::plain(-32601, format!("Method not found: {method}"))
    }
    /// -32602.
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::plain(-32602, message)
    }
    /// -32001 TaskNotFoundError.
    pub fn task_not_found(id: &str) -> Self {
        Self::a2a(-32001, "TASK_NOT_FOUND", "Task not found").with(json!({"taskId": id}))
    }
    /// -32002 TaskNotCancelableError.
    pub fn not_cancelable(id: &str) -> Self {
        Self::a2a(
            -32002,
            "TASK_NOT_CANCELABLE",
            "Task cannot be canceled: it has already ended",
        )
        .with(json!({"taskId": id}))
    }
    /// -32003 PushNotificationNotSupportedError.
    pub fn push_not_supported() -> Self {
        Self::a2a(
            -32003,
            "PUSH_NOTIFICATION_NOT_SUPPORTED",
            "Push notifications are not supported: use SubscribeToTask or GetTask",
        )
    }
    /// -32004 UnsupportedOperationError.
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::a2a(-32004, "UNSUPPORTED_OPERATION", message)
    }
    /// -32005 ContentTypeNotSupportedError.
    pub fn content_type_not_supported() -> Self {
        Self::a2a(
            -32005,
            "CONTENT_TYPE_NOT_SUPPORTED",
            "This agent answers in application/json and text/plain only",
        )
    }
    /// -32005 for a request body that is not `application/json`.
    pub fn request_content_type() -> Self {
        Self::a2a(
            -32005,
            "CONTENT_TYPE_NOT_SUPPORTED",
            "Content-Type must be application/json",
        )
    }
    /// -32009 VersionNotSupportedError.
    pub fn version_not_supported(asked: &str) -> Self {
        Self::a2a(
            -32009,
            "VERSION_NOT_SUPPORTED",
            format!(
                "A2A version {asked} is not supported: send the header A2A-Version: 1.0 (no header means 0.3)"
            ),
        )
        .with(json!({"supportedVersions": "1.0"}))
    }

    /// The `error` member of a JSON-RPC response.
    pub fn to_json(&self) -> Value {
        let mut e = json!({"code": self.code, "message": self.message});
        if let (Some(reason), Some(o)) = (self.reason, e.as_object_mut()) {
            let mut info = json!({
                "@type": "type.googleapis.com/google.rpc.ErrorInfo",
                "reason": reason,
                "domain": "a2a-protocol.org",
            });
            if let (Some(m), Some(i)) = (&self.metadata, info.as_object_mut()) {
                i.insert("metadata".into(), m.clone());
            }
            o.insert("data".into(), Value::Array(vec![info]));
        }
        e
    }
}

/// A successful JSON-RPC response.
pub fn ok(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

/// A JSON-RPC error response.
pub fn err(id: &Value, e: &RpcError) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": e.to_json()})
}

/// A parsed JSON-RPC request.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    /// The request id (string or number), echoed in the response.
    pub id: Value,
    /// Method name, e.g. `SendMessage`.
    pub method: String,
    /// Params object (`{}` when absent).
    pub params: Value,
}

/// Parse one JSON-RPC 2.0 request. On failure, the id to answer with (null
/// when it could not be read) and the error.
pub fn parse(body: &[u8]) -> Result<Call, (Value, RpcError)> {
    let v: Value =
        serde_json::from_slice(body).map_err(|_| (Value::Null, RpcError::parse_error()))?;
    let Some(o) = v.as_object() else {
        let why = if v.is_array() {
            "Invalid Request: batch requests are not supported, send one request per call"
        } else {
            "Invalid Request: the body must be a JSON-RPC 2.0 request object"
        };
        return Err((Value::Null, RpcError::invalid_request(why)));
    };
    let id = match o.get("id") {
        Some(Value::String(s)) => Value::String(s.clone()),
        Some(Value::Number(n)) => Value::Number(n.clone()),
        _ => {
            return Err((
                Value::Null,
                RpcError::invalid_request("Invalid Request: id must be a string or a number"),
            ));
        }
    };
    if o.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err((
            id,
            RpcError::invalid_request("Invalid Request: jsonrpc must be \"2.0\""),
        ));
    }
    let Some(method) = o.get("method").and_then(Value::as_str) else {
        return Err((
            id,
            RpcError::invalid_request("Invalid Request: method must be a string"),
        ));
    };
    let params = match o.get("params") {
        None | Some(Value::Null) => json!({}),
        Some(p @ Value::Object(_)) => p.clone(),
        Some(_) => {
            return Err((
                id,
                RpcError::invalid_params("Invalid params: params must be an object"),
            ));
        }
    };
    Ok(Call {
        id,
        method: method.to_string(),
        params,
    })
}

fn camel(k: &str) -> String {
    let mut out = String::with_capacity(k.len());
    let mut up = false;
    for c in k.chars() {
        if c == '_' {
            up = true;
        } else if up {
            out.extend(c.to_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Rename `snake_case` keys of one object to `lowerCamelCase` (the camelCase
/// key wins when both are present). Values are left alone.
fn rename_keys(v: &mut Value) {
    let Some(o) = v.as_object_mut() else {
        return;
    };
    let snake: Vec<String> = o.keys().filter(|k| k.contains('_')).cloned().collect();
    for k in snake {
        let c = camel(&k);
        if let Some(val) = o.remove(&k)
            && !o.contains_key(&c)
        {
            o.insert(c, val);
        }
    }
}

/// ProtoJSON readers accept the proto field names too (`history_length` for
/// `historyLength`). Normalize the protocol objects in params to camelCase,
/// never touching data the client owns (`data` parts, `metadata`).
pub fn normalize_params(params: &mut Value) {
    rename_keys(params);
    if let Some(c) = params.get_mut("configuration") {
        rename_keys(c);
    }
    if let Some(m) = params.get_mut("message") {
        rename_keys(m);
        if let Some(parts) = m.get_mut("parts").and_then(Value::as_array_mut) {
            parts.iter_mut().for_each(rename_keys);
        }
    }
}

/// Check the `A2A-Version` service parameter. Absent or empty means 0.3 (spec
/// 3.6.2); this agent speaks 1.0 only. A patch number is tolerated.
pub fn check_version(asked: Option<&str>) -> Result<(), RpcError> {
    let asked = asked.map(str::trim).unwrap_or_default();
    if asked.is_empty() {
        return Err(RpcError::version_not_supported(
            "0.3 (no A2A-Version header)",
        ));
    }
    let mut parts = asked.split('.');
    let major = parts.next().and_then(|p| p.parse::<u32>().ok());
    let minor = parts.next().and_then(|p| p.parse::<u32>().ok());
    let patch_ok = match parts.next() {
        None => true,
        Some(p) => p.parse::<u32>().is_ok() && parts.next().is_none(),
    };
    match (major, minor) {
        (Some(1), Some(0)) if patch_ok => Ok(()),
        _ => Err(RpcError::version_not_supported(asked)),
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)] // serde_json `v["key"]` in test asserts
mod tests {
    use super::*;

    fn code(body: &str) -> (Value, i64) {
        match parse(body.as_bytes()) {
            Ok(_) => (Value::Null, 0),
            Err((id, e)) => (id, e.code),
        }
    }

    #[test]
    fn envelope_errors() {
        assert_eq!(code("not json"), (Value::Null, -32700));
        assert_eq!(code("{\"jsonrpc\":\"2.0\""), (Value::Null, -32700));
        assert_eq!(code("[]"), (Value::Null, -32600));
        assert_eq!(
            code("[{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"GetTask\"}]").1,
            -32600
        );
        assert_eq!(code("42"), (Value::Null, -32600));
        assert_eq!(
            code("{\"jsonrpc\":\"2.0\",\"method\":\"GetTask\"}"),
            (Value::Null, -32600)
        );
        assert_eq!(
            code("{\"jsonrpc\":\"2.0\",\"id\":null,\"method\":\"GetTask\"}"),
            (Value::Null, -32600)
        );
        assert_eq!(
            code("{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"GetTask\"}"),
            (Value::Null, -32600)
        );
        assert_eq!(
            code("{\"jsonrpc\":\"1.0\",\"id\":7,\"method\":\"GetTask\"}"),
            (json!(7), -32600)
        );
        assert_eq!(
            code("{\"id\":\"a\",\"method\":\"GetTask\"}"),
            (json!("a"), -32600)
        );
        assert_eq!(
            code("{\"jsonrpc\":\"2.0\",\"id\":\"a\",\"method\":5}"),
            (json!("a"), -32600)
        );
        assert_eq!(
            code("{\"jsonrpc\":\"2.0\",\"id\":\"a\",\"method\":\"GetTask\",\"params\":[1]}"),
            (json!("a"), -32602)
        );
    }

    #[test]
    fn good_envelope() {
        let c = parse(br#"{"jsonrpc":"2.0","id":"x-1","method":"GetTask","params":{"id":"t"}}"#);
        assert_eq!(
            c,
            Ok(Call {
                id: json!("x-1"),
                method: "GetTask".into(),
                params: json!({"id": "t"})
            })
        );
        let c = parse(br#"{"jsonrpc":"2.0","id":3,"method":"GetExtendedAgentCard"}"#);
        assert_eq!(c.map(|c| c.params), Ok(json!({})));
    }

    #[test]
    fn a2a_errors_carry_error_info() {
        let e = RpcError::task_not_found("abc").to_json();
        assert_eq!(e["code"], -32001);
        assert_eq!(
            e["data"][0]["@type"],
            "type.googleapis.com/google.rpc.ErrorInfo"
        );
        assert_eq!(e["data"][0]["reason"], "TASK_NOT_FOUND");
        assert_eq!(e["data"][0]["domain"], "a2a-protocol.org");
        assert_eq!(e["data"][0]["metadata"]["taskId"], "abc");
        for (err, code, reason) in [
            (RpcError::not_cancelable("a"), -32002, "TASK_NOT_CANCELABLE"),
            (
                RpcError::push_not_supported(),
                -32003,
                "PUSH_NOTIFICATION_NOT_SUPPORTED",
            ),
            (RpcError::unsupported("x"), -32004, "UNSUPPORTED_OPERATION"),
            (
                RpcError::content_type_not_supported(),
                -32005,
                "CONTENT_TYPE_NOT_SUPPORTED",
            ),
            (
                RpcError::version_not_supported("0.3"),
                -32009,
                "VERSION_NOT_SUPPORTED",
            ),
        ] {
            let j = err.to_json();
            assert_eq!(j["code"], code);
            assert_eq!(j["data"][0]["reason"], reason);
        }
        // Plain JSON-RPC errors have no A2A ErrorInfo.
        for e in [
            RpcError::parse_error(),
            RpcError::invalid_request("x"),
            RpcError::method_not_found("X"),
            RpcError::invalid_params("x"),
        ] {
            assert!(e.to_json().get("data").is_none());
        }
    }

    #[test]
    fn responses() {
        assert_eq!(
            ok(&json!(1), json!({"task": {}})),
            json!({"jsonrpc": "2.0", "id": 1, "result": {"task": {}}})
        );
        let e = err(&json!("a"), &RpcError::method_not_found("Nope"));
        assert_eq!(e["jsonrpc"], "2.0");
        assert_eq!(e["id"], "a");
        assert_eq!(e["error"]["code"], -32601);
        assert!(e.get("result").is_none());
    }

    #[test]
    fn proto_field_names_are_accepted() {
        let mut p = json!({
            "history_length": 0,
            "include_artifacts": true,
            "pageSize": 5,
            "page_size": 9,
            "configuration": {"return_immediately": true},
            "message": {"message_id": "m", "task_id": "t", "parts": [{"media_type": "text/plain", "text": "x"}, {"data": {"snake_key": 1}}], "metadata": {"my_key": 1}},
        });
        normalize_params(&mut p);
        assert_eq!(p["historyLength"], 0);
        assert_eq!(p["includeArtifacts"], true);
        assert_eq!(p["pageSize"], 5);
        assert!(p.get("page_size").is_none());
        assert_eq!(p["configuration"]["returnImmediately"], true);
        assert_eq!(p["message"]["messageId"], "m");
        assert_eq!(p["message"]["taskId"], "t");
        assert_eq!(p["message"]["parts"][0]["mediaType"], "text/plain");
        // Client data stays as it is.
        assert_eq!(p["message"]["parts"][1]["data"]["snake_key"], 1);
        assert_eq!(p["message"]["metadata"]["my_key"], 1);
    }

    #[test]
    fn version_rules() {
        assert!(check_version(Some("1.0")).is_ok());
        assert!(check_version(Some(" 1.0 ")).is_ok());
        assert!(check_version(Some("1.0.1")).is_ok());
        for bad in [
            None,
            Some(""),
            Some("0.3"),
            Some("0.3.0"),
            Some("2.0"),
            Some("1.1"),
            Some("1"),
            Some("v1.0"),
            Some("1.0.x"),
            Some("1.0.0.0"),
        ] {
            assert_eq!(
                check_version(bad).map_err(|e| e.code),
                Err(-32009),
                "{bad:?}"
            );
        }
    }
}

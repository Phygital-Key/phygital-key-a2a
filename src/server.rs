//! The A2A JSON-RPC endpoint: one POST route at `/a2a`, the A2A 1.0 methods,
//! and server-sent events for the two streaming methods.
//!
//! The base guard has already checked the Host, the bearer access token and
//! that no browser `Origin` is present before a request gets here.

use std::collections::VecDeque;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use phygital_key_gate::state::PENDING_SECS;
use phygital_key_gate::{Shared, Status, View};
use serde_json::{Value, json};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};
use tokio::time::Instant;

use crate::intent::{self, HELP, Incoming, Intent, SendConfig};
use crate::proto::{EXT, TaskState, agent_message, data_part, text_part};
use crate::rpc::{self, Call, RpcError};
use crate::store::{HistoryFull, Kind, ListQuery, Store, TaskRec};
use crate::time::{now_ms, parse_iso};

/// Heartbeat comment interval on open streams.
const HEARTBEAT: Duration = Duration::from_secs(15);
/// Most streams (SendStreamingMessage and SubscribeToTask) open at once.
pub const MAX_STREAMS: usize = 32;

/// The adapter's state: the gate and the tasks made through A2A.
pub struct App {
    /// The gate.
    pub gate: Shared,
    /// The tasks.
    pub store: Store,
    /// One permit per open stream.
    pub streams: Arc<Semaphore>,
}

/// The `/a2a` route.
pub fn router(gate: Shared) -> Router {
    let app = Arc::new(App {
        gate,
        store: Store::default(),
        streams: Arc::new(Semaphore::new(MAX_STREAMS)),
    });
    Router::new()
        .route("/a2a", post(handle))
        // Some clients join the interface URL with "/" (e.g. httpx base_url).
        .route("/a2a/", post(handle))
        .layer(DefaultBodyLimit::max(64 * 1024))
        .with_state(app)
}

pub(crate) fn new_id() -> String {
    phygital_key_gate::util::new_id().unwrap_or_else(|_| format!("id-{}", now_ms()))
}

fn json_response(status: StatusCode, v: &Value) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        v.to_string(),
    )
        .into_response()
}

fn query_param(query: Option<&str>, name: &str) -> Option<String> {
    query?
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.to_string())
}

/// The body is JSON: `application/json`, or `application/a2a+json` (what
/// A2A 1.0.1 recommends), with any parameters such as charset.
fn json_content_type(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .and_then(|c| c.split(';').next())
        .map(str::trim)
        .is_some_and(|c| {
            c.eq_ignore_ascii_case("application/json")
                || c.eq_ignore_ascii_case("application/a2a+json")
        })
}

async fn handle(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    uri: Uri,
    body: Bytes,
) -> Response {
    if !json_content_type(&headers) {
        return json_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            &rpc::err(&Value::Null, &RpcError::request_content_type()),
        );
    }
    let call = match rpc::parse(&body) {
        Ok(c) => c,
        Err((id, e)) => return json_response(StatusCode::OK, &rpc::err(&id, &e)),
    };
    let version = headers
        .get("a2a-version")
        .and_then(|h| h.to_str().ok())
        .map(String::from)
        .or_else(|| query_param(uri.query(), "A2A-Version"));
    if let Err(e) = rpc::check_version(version.as_deref()) {
        return json_response(StatusCode::OK, &rpc::err(&call.id, &e));
    }
    let ext_on = headers
        .get_all("a2a-extensions")
        .iter()
        .filter_map(|h| h.to_str().ok())
        .flat_map(|h| h.split(','))
        .any(|u| u.trim() == EXT);
    let mut res = dispatch(&app, call).await;
    if ext_on {
        res.headers_mut()
            .insert("a2a-extensions", HeaderValue::from_static(EXT));
    }
    res
}

/// What a sent message led to.
pub(crate) enum Outcome {
    /// A task (new, or the current state of the one the message named).
    Task(TaskRec),
    /// A direct reply, no task.
    Message(Value),
}

enum Reply {
    Json(Value),
    Stream(Response),
}

async fn dispatch(app: &Arc<App>, mut call: Call) -> Response {
    rpc::normalize_params(&mut call.params);
    let id = call.id.clone();
    let r: Result<Reply, RpcError> = match call.method.as_str() {
        "SendMessage" => send_message(app, &call.params).await.map(Reply::Json),
        "SendStreamingMessage" => send_streaming(app, &call).await.map(Reply::Stream),
        "GetTask" => get_task(app, &call.params).map(Reply::Json),
        "ListTasks" => list_tasks(app, &call.params).map(Reply::Json),
        "CancelTask" => cancel_task(app, &call.params).map(Reply::Json),
        "SubscribeToTask" => subscribe(app, &call).map(Reply::Stream),
        "GetExtendedAgentCard" => Err(RpcError::unsupported(
            "This agent has no extended agent card: everything is in /.well-known/agent-card.json",
        )),
        "CreateTaskPushNotificationConfig"
        | "GetTaskPushNotificationConfig"
        | "ListTaskPushNotificationConfigs"
        | "DeleteTaskPushNotificationConfig" => Err(RpcError::push_not_supported()),
        other => Err(RpcError::method_not_found(
            &other.chars().take(64).collect::<String>(),
        )),
    };
    match r {
        Ok(Reply::Json(v)) => json_response(StatusCode::OK, &rpc::ok(&id, v)),
        Ok(Reply::Stream(s)) => s,
        Err(e) => json_response(StatusCode::OK, &rpc::err(&id, &e)),
    }
}

/// A gate refusal as the agent should read it here (the gate's wording names
/// the MCP tools in places; here the same thing is a message shape).
fn gate_error(e: &str) -> String {
    if e.starts_with("\"standing\" is reserved") {
        return "\"standing\" is reserved: for a standing approval, send a {\"standing\": {...}} message instead".into();
    }
    e.to_string()
}

fn error_message(context_id: &str, e: &str) -> Value {
    let e = gate_error(e);
    agent_message(
        &new_id(),
        context_id,
        None,
        vec![text_part(&e), data_part(json!({"error": e}))],
    )
}

fn history_full_message(t: &TaskRec) -> Value {
    let why = "This message was not kept: the agent holds as many follow-up messages as it can right now. Nothing said here changes the task: the person decides on the approval page. Poll with GetTask.";
    agent_message(
        &new_id(),
        &t.context_id,
        Some(&t.id),
        vec![text_part(why), data_part(json!({"error": why, "id": t.id}))],
    )
}

fn help_message(context_id: &str, why: &str) -> Value {
    agent_message(
        &new_id(),
        context_id,
        None,
        vec![
            text_part(&format!("{why} {HELP}")),
            data_part(json!({"error": why})),
        ],
    )
}

impl App {
    fn new_task(
        self: &Arc<Self>,
        view: &View,
        kind: Kind,
        context_id: String,
        first: Value,
    ) -> TaskRec {
        let t = self
            .store
            .insert(TaskRec::new(view, kind, context_id, first));
        self.watch(t.id.clone());
        t
    }

    /// Follow one request at the gate until the person decides, it is cancelled,
    /// or it expires, and record the change.
    fn watch(self: &Arc<Self>, id: String) {
        let app = self.clone();
        tokio::spawn(async move {
            let end = Instant::now() + Duration::from_secs(PENDING_SECS.unsigned_abs() + 30);
            loop {
                match app.gate.wait(&id, 120).await {
                    Ok((v, a)) if v.status != Status::Pending => {
                        app.store.observe(&id, Some(&v), a.as_deref());
                        break;
                    }
                    Ok(_) => {}
                    Err(_) => {
                        app.store.observe(&id, None, None);
                        break;
                    }
                }
                let ended = app.store.get(&id).is_none_or(|t| t.state.is_terminal());
                if ended || Instant::now() >= end {
                    break;
                }
            }
        });
    }

    /// Handle one sent message (SendMessage and SendStreamingMessage).
    fn accept(self: &Arc<Self>, m: Incoming) -> Result<Outcome, RpcError> {
        if let Some(task_id) = &m.task_id {
            let t = self
                .store
                .refresh(&self.gate, task_id)
                .ok_or_else(|| RpcError::task_not_found(task_id))?;
            if m.context_id.as_ref().is_some_and(|c| c != &t.context_id) {
                return Err(RpcError::invalid_params(
                    "message.contextId does not match the task's contextId",
                ));
            }
            if t.state.is_terminal() {
                return Err(RpcError::unsupported(format!(
                    "Task {task_id} has ended ({}): it accepts no more messages. Send a new message without taskId.",
                    t.state.as_str()
                )));
            }
            // Waiting for the person: the decision happens out of band, on the
            // approval page. Nothing said here changes it; only its text is
            // kept in the task's history (capped).
            return Ok(
                match self.store.add_history(task_id, intent::follow_up(&m)) {
                    Some(Ok(t)) => Outcome::Task(t),
                    Some(Err(HistoryFull)) => Outcome::Message(history_full_message(&t)),
                    None => Outcome::Task(t),
                },
            );
        }
        let ctx = m.context_id.clone().unwrap_or_else(new_id);
        let intent = match intent::intent(&m.parts) {
            Ok(i) => i,
            Err(why) => return Ok(Outcome::Message(help_message(&ctx, &why))),
        };
        let out = match intent {
            Intent::Request {
                request,
                aud,
                level,
            } => match self.gate.create(request, &aud, level) {
                Ok(v) => Outcome::Task(self.new_task(&v, Kind::Approval, ctx, m.raw)),
                Err(e) => Outcome::Message(error_message(&ctx, &e)),
            },
            Intent::Standing {
                for_type,
                fixed,
                vary,
                max_uses,
                days,
                aud,
            } => match self
                .gate
                .create_standing(&for_type, fixed, vary, max_uses, days, &aud)
            {
                Ok(v) => Outcome::Task(self.new_task(&v, Kind::Standing, ctx, m.raw)),
                Err(e) => Outcome::Message(error_message(&ctx, &e)),
            },
            Intent::Use {
                standing_id,
                request,
            } => Outcome::Message(match self.gate.use_standing(&standing_id, &request) {
                Ok(u) => agent_message(
                    &new_id(),
                    &ctx,
                    None,
                    vec![
                        text_part(
                            "Signed one use. Send both `approval` (header Phygital-Approval) and `useToken` (header Phygital-Use) to the service with this exact request. The use token works once, within 60 seconds.",
                        ),
                        data_part(json!({
                            "standingId": standing_id,
                            "approval": u.approval,
                            "useToken": u.use_token,
                            "usesLeft": u.uses_left,
                        })),
                    ],
                ),
                Err(e) => error_message(&ctx, &e),
            }),
            Intent::Receipt { id, receipt } => {
                Outcome::Message(match self.gate.record_receipt(&id, &receipt) {
                    Ok(v) => agent_message(
                        &new_id(),
                        &ctx,
                        None,
                        vec![
                            text_part(&format!(
                                "Receipt recorded. The service reported: {}. The person can see it on the approval page.",
                                v.service_reported.as_deref().unwrap_or("nothing")
                            )),
                            data_part(json!({"id": v.id, "serviceReported": v.service_reported})),
                        ],
                    ),
                    Err(e) => error_message(&ctx, &e),
                })
            }
            Intent::Revoke { standing_id } => {
                Outcome::Message(match self.gate.revoke(&standing_id) {
                    Ok((_, sign_url)) => agent_message(
                        &new_id(),
                        &ctx,
                        None,
                        vec![
                            text_part(&format!(
                                "Revoked: no new uses can be signed. Ask the person to open {sign_url} and sign the revocation, so the service refuses it too."
                            )),
                            data_part(
                                json!({"standingId": standing_id, "revoked": true, "signUrl": sign_url}),
                            ),
                        ],
                    ),
                    Err(e) => error_message(&ctx, &e),
                })
            }
        };
        Ok(out)
    }
}

fn opt_u64(params: &Value, name: &str) -> Result<Option<u64>, RpcError> {
    match params.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v.as_u64().map(Some).ok_or_else(|| {
            RpcError::invalid_params(format!("{name} must be an integer, 0 or more"))
        }),
    }
}

fn required_id(params: &Value) -> Result<&str, RpcError> {
    params
        .get("id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| RpcError::invalid_params("id is required (the task id)"))
}

/// A sent message: a conformance-kit message in a `tck` build, else the gate.
async fn accept_any(
    app: &Arc<App>,
    m: Incoming,
    cfg: &SendConfig,
    streaming: bool,
) -> Result<Outcome, RpcError> {
    #[cfg(feature = "tck")]
    if let Some(r) = crate::tck::send(app, &m, cfg, streaming).await {
        return r;
    }
    #[cfg(not(feature = "tck"))]
    let _ = (cfg, streaming);
    app.accept(m)
}

async fn send_message(app: &Arc<App>, params: &Value) -> Result<Value, RpcError> {
    let (m, cfg) = intent::parse_send(params)?;
    // Blocking or not, the answer is the same: a new task is at once in
    // TASK_STATE_AUTH_REQUIRED, an interrupted state, so there is nothing to wait for.
    let history_length = cfg.history_length;
    Ok(match accept_any(app, m, &cfg, false).await? {
        Outcome::Task(t) => json!({"task": t.to_json(history_length, true)}),
        Outcome::Message(msg) => json!({"message": msg}),
    })
}

fn get_task(app: &Arc<App>, params: &Value) -> Result<Value, RpcError> {
    let id = required_id(params)?;
    let history = opt_u64(params, "historyLength")?;
    let t = app
        .store
        .refresh(&app.gate, id)
        .ok_or_else(|| RpcError::task_not_found(id))?;
    Ok(t.to_json(history, true))
}

fn cancel_task(app: &Arc<App>, params: &Value) -> Result<Value, RpcError> {
    let id = required_id(params)?;
    let t = app
        .store
        .refresh(&app.gate, id)
        .ok_or_else(|| RpcError::task_not_found(id))?;
    if t.state.is_terminal() {
        return Err(RpcError::not_cancelable(id));
    }
    #[cfg(feature = "tck")]
    if t.kind == Kind::Test {
        return crate::tck::cancel(app, id);
    }
    match app.gate.cancel(id) {
        Ok(v) => {
            let t = app
                .store
                .observe(id, Some(&v), None)
                .ok_or_else(|| RpcError::task_not_found(id))?;
            if t.state == TaskState::Canceled {
                Ok(t.to_json(None, true))
            } else {
                Err(RpcError::not_cancelable(id))
            }
        }
        // Decided a moment ago: report it as not cancelable.
        Err(_) => {
            app.store.refresh(&app.gate, id);
            Err(RpcError::not_cancelable(id))
        }
    }
}

fn list_tasks(app: &Arc<App>, params: &Value) -> Result<Value, RpcError> {
    let str_param = |name: &str| -> Result<Option<String>, RpcError> {
        match params.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(s)) if s.is_empty() => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(RpcError::invalid_params(format!("{name} must be a string"))),
        }
    };
    let page_size = match params.get("pageSize") {
        None | Some(Value::Null) => 50,
        Some(v) => match v.as_u64() {
            Some(n @ 1..=100) => usize::try_from(n).unwrap_or(50),
            _ => {
                return Err(RpcError::invalid_params(
                    "pageSize must be an integer from 1 to 100",
                ));
            }
        },
    };
    let status = match str_param("status")? {
        None => None,
        Some(s) if s == "TASK_STATE_UNSPECIFIED" => None,
        Some(s) => Some(TaskState::parse(&s).ok_or_else(|| {
            RpcError::invalid_params("status must be a TaskState such as TASK_STATE_COMPLETED")
        })?),
    };
    let after_ms = match str_param("statusTimestampAfter")? {
        None => None,
        Some(s) => Some(parse_iso(&s).ok_or_else(|| {
            RpcError::invalid_params("statusTimestampAfter must be an ISO 8601 timestamp")
        })?),
    };
    let include_artifacts = match params.get("includeArtifacts") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => {
            return Err(RpcError::invalid_params(
                "includeArtifacts must be true or false",
            ));
        }
    };
    // No history unless the client asks for it.
    let history = opt_u64(params, "historyLength")?.or(Some(0));
    let q = ListQuery {
        context_id: str_param("contextId")?,
        status,
        after_ms,
        page_size,
        page_token: str_param("pageToken")?,
    };
    app.store.refresh_all(&app.gate);
    let (page, next, total) = app.store.list(&q).map_err(RpcError::invalid_params)?;
    Ok(json!({
        "tasks": page.iter().map(|t| t.to_json(history, include_artifacts)).collect::<Vec<_>>(),
        "nextPageToken": next,
        "pageSize": page_size,
        "totalSize": total,
    }))
}

/// One open stream: the task first, then its changes, closed at a terminal state.
struct Live {
    app: Arc<App>,
    task_id: String,
    rpc_id: Value,
    queue: VecDeque<Value>,
    sent: usize,
    rx: watch::Receiver<u64>,
    end: Instant,
    done: bool,
    /// Held while the stream is open; dropped (freed) when it ends or the
    /// client goes away.
    _permit: OwnedSemaphorePermit,
}

/// A stream closes when the task reaches a terminal state, or moves to needing more input.
/// It stays open in TASK_STATE_AUTH_REQUIRED: the person decides out of band
/// and the stream then carries the result (spec 7.6.1).
fn ends_stream(s: TaskState) -> bool {
    s.is_terminal() || s == TaskState::InputRequired
}

fn sse_event(rpc_id: &Value, result: Value) -> Result<Event, Infallible> {
    Ok(Event::default().data(rpc::ok(rpc_id, result).to_string()))
}

async fn next_event(mut st: Live) -> Option<(Result<Event, Infallible>, Live)> {
    loop {
        if let Some(ev) = st.queue.pop_front() {
            let e = sse_event(&st.rpc_id, ev);
            return Some((e, st));
        }
        if st.done {
            return None;
        }
        match st.app.store.refresh(&st.app.gate, &st.task_id) {
            None => {
                st.done = true;
                continue;
            }
            Some(t) => {
                let new = t.events.get(st.sent..).unwrap_or_default();
                if !new.is_empty() {
                    st.sent = t.events.len();
                    st.queue.extend(new.iter().cloned());
                    st.done = ends_stream(t.state);
                    continue;
                }
                if t.state.is_terminal() {
                    st.done = true;
                    continue;
                }
            }
        }
        match tokio::time::timeout_at(st.end, st.rx.changed()).await {
            Ok(Ok(())) => {}
            // The decision window is over (or the store is gone): close.
            _ => st.done = true,
        }
    }
}

/// A permit for one more open stream, or UnsupportedOperation when all are taken.
fn stream_permit(streams: &Arc<Semaphore>) -> Result<OwnedSemaphorePermit, RpcError> {
    streams.clone().try_acquire_owned().map_err(|_| {
        RpcError::unsupported(format!(
            "too many open streams; poll with GetTask (at most {MAX_STREAMS} are open at once)"
        ))
    })
}

fn stream_task(
    app: &Arc<App>,
    rpc_id: &Value,
    t: &TaskRec,
    history: Option<u64>,
    permit: OwnedSemaphorePermit,
) -> Response {
    // Subscribe before the first look, so no change can slip between them.
    let rx = app.store.subscribe();
    let live = Live {
        app: app.clone(),
        task_id: t.id.clone(),
        rpc_id: rpc_id.clone(),
        queue: VecDeque::from([json!({"task": t.to_json(history, true)})]),
        sent: t.events.len(),
        rx,
        end: Instant::now() + Duration::from_secs(PENDING_SECS.unsigned_abs()),
        // A task already waiting for input stays subscribed until it moves on.
        done: t.state.is_terminal(),
        _permit: permit,
    };
    let stream = futures_util::stream::unfold(live, next_event);
    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(HEARTBEAT))
        .into_response()
}

async fn send_streaming(app: &Arc<App>, call: &Call) -> Result<Response, RpcError> {
    let (m, cfg) = intent::parse_send(&call.params)?;
    // Before the message is acted on, so a refused stream makes no task.
    let permit = stream_permit(&app.streams)?;
    Ok(match accept_any(app, m, &cfg, true).await? {
        Outcome::Task(t) => stream_task(app, &call.id, &t, cfg.history_length, permit),
        Outcome::Message(msg) => {
            let one = futures_util::stream::iter([sse_event(&call.id, json!({"message": msg}))]);
            Sse::new(one).into_response()
        }
    })
}

fn subscribe(app: &Arc<App>, call: &Call) -> Result<Response, RpcError> {
    let id = required_id(&call.params)?;
    let t = app
        .store
        .refresh(&app.gate, id)
        .ok_or_else(|| RpcError::task_not_found(id))?;
    if t.state.is_terminal() {
        return Err(RpcError::unsupported(format!(
            "Task {id} has ended ({}): there is nothing to subscribe to. Use GetTask.",
            t.state.as_str()
        )));
    }
    let permit = stream_permit(&app.streams)?;
    Ok(stream_task(app, &call.id, &t, None, permit))
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)] // serde_json `v["key"]` in test asserts
mod tests {
    use super::*;

    #[test]
    fn query_params() {
        assert_eq!(
            query_param(Some("a=1&A2A-Version=1.0"), "A2A-Version"),
            Some("1.0".into())
        );
        assert_eq!(
            query_param(Some("a2a-version=1.0"), "A2A-Version"),
            Some("1.0".into())
        );
        assert_eq!(query_param(Some("x=1"), "A2A-Version"), None);
        assert_eq!(query_param(None, "A2A-Version"), None);
    }

    #[test]
    fn content_types() {
        let ct = |v: Option<&str>| {
            let mut h = HeaderMap::new();
            if let Some(v) = v.and_then(|v| HeaderValue::from_str(v).ok()) {
                h.insert(header::CONTENT_TYPE, v);
            }
            json_content_type(&h)
        };
        for ok in [
            "application/json",
            "application/a2a+json",
            "Application/A2A+JSON",
            "application/json; charset=utf-8",
            "application/a2a+json;charset=UTF-8",
        ] {
            assert!(ct(Some(ok)), "{ok}");
        }
        for bad in [
            "text/plain",
            "application/jsonx",
            "application/a2a",
            "application/x-a2a+json",
            "multipart/form-data",
            "",
        ] {
            assert!(!ct(Some(bad)), "{bad}");
        }
        assert!(!ct(None));
    }

    #[test]
    fn gate_errors_speak_a2a() {
        for e in [
            "\"standing\" is reserved: use request_standing_approval",
            "\"standing\" is reserved: ask for a standing approval instead",
        ] {
            assert_eq!(
                gate_error(e),
                "\"standing\" is reserved: for a standing approval, send a {\"standing\": {...}} message instead"
            );
        }
        assert_eq!(
            gate_error("request is nested too deeply"),
            "request is nested too deeply"
        );
        let m = error_message("c", "no key enrolled");
        assert_eq!(m["role"], "ROLE_AGENT");
        assert_eq!(m["contextId"], "c");
        assert_eq!(m["parts"][0]["text"], "no key enrolled");
        assert_eq!(m["parts"][1]["data"]["error"], "no key enrolled");
        assert!(m.get("taskId").is_none());
    }

    #[test]
    fn streams_are_capped() {
        let streams = Arc::new(Semaphore::new(MAX_STREAMS));
        let mut open: Vec<_> = (0..MAX_STREAMS)
            .filter_map(|_| stream_permit(&streams).ok())
            .collect();
        assert_eq!(open.len(), MAX_STREAMS);
        let refused = stream_permit(&streams).err();
        assert_eq!(refused.as_ref().map(|e| e.code), Some(-32004));
        assert!(
            refused
                .map(|e| e.message)
                .unwrap_or_default()
                .starts_with("too many open streams; poll with GetTask")
        );
        // One stream closes: one more can open.
        open.pop();
        let again = stream_permit(&streams);
        assert!(again.is_ok());
        assert!(stream_permit(&streams).is_err());
        drop(again);
        drop(open);
        assert_eq!(streams.available_permits(), MAX_STREAMS);
    }

    #[test]
    fn history_full_is_an_agent_message() {
        let t = TaskRec {
            id: "t1".into(),
            context_id: "c".into(),
            kind: Kind::Approval,
            state: TaskState::AuthRequired,
            ts_ms: 0,
            approve_url: String::new(),
            status_message: Value::Null,
            artifact: None,
            history: Vec::new(),
            history_bytes: 0,
            seq: 0,
            events: Vec::new(),
        };
        let m = history_full_message(&t);
        assert_eq!(m["role"], "ROLE_AGENT");
        assert_eq!(m["taskId"], "t1");
        assert!(
            m["parts"][0]["text"]
                .as_str()
                .unwrap_or_default()
                .contains("GetTask")
        );
    }

    #[test]
    fn help_lists_the_shapes() {
        let m = help_message("c", "This message has no data part.");
        let text = m["parts"][0]["text"].as_str().unwrap_or_default();
        for k in [
            "\"request\"",
            "\"standing\"",
            "\"use\"",
            "\"receipt\"",
            "\"revoke\"",
            "never counts as approval",
        ] {
            assert!(text.contains(k), "{k}");
        }
        assert_eq!(
            m["parts"][1]["data"]["error"],
            "This message has no data part."
        );
    }
}

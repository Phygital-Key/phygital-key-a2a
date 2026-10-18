//! Test behaviours for the official A2A conformance kit (a2a-tck).
//!
//! Compiled only with `cargo build --features tck`; the release workflow never
//! enables it. The kit drives a "system under test" through messages whose
//! `messageId` starts with `tck-` (scenarios/*.feature in the kit): complete a
//! task, add artifacts, ask for input, stream updates. Those tasks never touch
//! the gate, so nothing here can make an approval; a build with this feature
//! must never be used for real approvals.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};

use crate::intent::{Incoming, SendConfig};
use crate::proto::{TaskState, agent_message, text_part};
use crate::rpc::RpcError;
use crate::server::{App, Outcome, new_id};
use crate::store::{Kind, TaskRec};
use crate::time::now_ms;

/// The kit's streaming timeout is 2 s; the resubscribe scenario waits twice that.
const RESUBSCRIBE_WAIT_MS: u64 = 4000;

#[derive(Debug, Clone)]
enum Step {
    Working,
    InputRequired,
    Complete(Option<&'static str>),
    Reject,
    Artifact {
        part: Value,
        append: bool,
        last: bool,
    },
    Wait(u64),
}

enum Plan {
    Message(&'static str),
    Steps(Vec<Step>),
}

fn text_artifact(text: &str, append: bool, last: bool) -> Step {
    Step::Artifact {
        part: json!({"text": text}),
        append,
        last,
    }
}

fn file_part() -> Value {
    // b"tck", base64.
    json!({"raw": "dGNr", "filename": "output.txt", "mediaType": "text/plain"})
}

/// The kit's scenarios, by messageId prefix (longest prefix wins).
fn plan(message_id: &str) -> Option<Plan> {
    if !message_id.starts_with("tck-") && !message_id.starts_with("test-resubscribe") {
        return None;
    }
    use Step::*;
    let streamed =
        |text: &'static str| vec![Working, text_artifact(text, false, true), Complete(None)];
    let table: Vec<(&str, Plan)> = vec![
        (
            "test-resubscribe-message-id",
            Plan::Steps(vec![Working, Wait(RESUBSCRIBE_WAIT_MS), Complete(None)]),
        ),
        (
            "tck-stream-artifact-chunked",
            Plan::Steps(vec![
                Working,
                text_artifact("chunk-1 ", false, false),
                text_artifact("chunk-2", true, true),
                Complete(None),
            ]),
        ),
        (
            "tck-stream-artifact-file",
            Plan::Steps(vec![
                Working,
                Artifact {
                    part: file_part(),
                    append: false,
                    last: true,
                },
                Complete(None),
            ]),
        ),
        (
            "tck-stream-artifact-text",
            Plan::Steps(streamed("Streamed text content")),
        ),
        (
            "tck-stream-ordering-001",
            Plan::Steps(streamed("Ordered output")),
        ),
        (
            "tck-artifact-file-url",
            Plan::Steps(vec![
                Artifact {
                    part: json!({"url": "https://example.com/output.txt", "filename": "output.txt", "mediaType": "text/plain"}),
                    append: false,
                    last: true,
                },
                Complete(None),
            ]),
        ),
        (
            "tck-message-response",
            Plan::Message("Direct message response"),
        ),
        ("tck-input-required", Plan::Steps(vec![InputRequired])),
        (
            "tck-artifact-text",
            Plan::Steps(vec![
                text_artifact("Generated text content", false, true),
                Complete(None),
            ]),
        ),
        (
            "tck-artifact-file",
            Plan::Steps(vec![
                Artifact {
                    part: file_part(),
                    append: false,
                    last: true,
                },
                Complete(None),
            ]),
        ),
        (
            "tck-artifact-data",
            Plan::Steps(vec![
                Artifact {
                    part: json!({"data": {"key": "value", "count": 42}, "mediaType": "application/json"}),
                    append: false,
                    last: true,
                },
                Complete(None),
            ]),
        ),
        (
            "tck-complete-task",
            Plan::Steps(vec![Complete(Some("Hello from TCK"))]),
        ),
        ("tck-reject-task", Plan::Steps(vec![Reject])),
        (
            "tck-stream-001",
            Plan::Steps(streamed("Stream hello from TCK")),
        ),
        ("tck-stream-002", Plan::Steps(vec![Complete(None)])),
        (
            "tck-stream-003",
            Plan::Steps(streamed("Stream task lifecycle")),
        ),
    ];
    let mut best: Option<(usize, Plan)> = None;
    for (prefix, p) in table {
        if message_id.starts_with(prefix) && best.as_ref().is_none_or(|(n, _)| prefix.len() > *n) {
            best = Some((prefix.len(), p));
        }
    }
    Some(match best {
        Some((_, p)) => p,
        // The kit's default: complete the task.
        None => Plan::Steps(vec![Step::Complete(None)]),
    })
}

fn status_event(t: &TaskRec) -> Value {
    json!({"statusUpdate": {"taskId": t.id, "contextId": t.context_id, "status": t.status_json()}})
}

fn set_state(app: &App, id: &str, state: TaskState, text: Option<&str>) {
    app.store.update(id, |t| {
        t.state = state;
        t.ts_ms = now_ms();
        t.status_message = match text {
            Some(x) => agent_message(&new_id(), &t.context_id, Some(&t.id), vec![text_part(x)]),
            None => Value::Null,
        };
        let ev = status_event(t);
        t.events.push(ev);
    });
}

async fn run(app: Arc<App>, id: String, steps: Vec<Step>) {
    for step in steps {
        match step {
            Step::Working => set_state(&app, &id, TaskState::Working, None),
            Step::InputRequired => set_state(
                &app,
                &id,
                TaskState::InputRequired,
                Some("More input is needed."),
            ),
            Step::Complete(text) => set_state(&app, &id, TaskState::Completed, text),
            Step::Reject => set_state(&app, &id, TaskState::Rejected, Some("rejected")),
            Step::Wait(ms) => tokio::time::sleep(Duration::from_millis(ms)).await,
            Step::Artifact { part, append, last } => {
                app.store.update(&id, |t| {
                    let chunk = json!({"artifactId": "tck-artifact", "parts": [part.clone()]});
                    match (&mut t.artifact, append) {
                        (Some(a), true) => {
                            if let Some(parts) = a.get_mut("parts").and_then(Value::as_array_mut) {
                                parts.push(part);
                            }
                        }
                        (slot, _) => *slot = Some(chunk.clone()),
                    }
                    t.events.push(json!({"artifactUpdate": {
                        "taskId": t.id, "contextId": t.context_id, "artifact": chunk, "append": append, "lastChunk": last,
                    }}));
                });
            }
        }
        // Keep each change its own event for streams.
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// Handle a kit message, or `None` when the message is not one.
pub async fn send(
    app: &Arc<App>,
    m: &Incoming,
    cfg: &SendConfig,
    streaming: bool,
) -> Option<Result<Outcome, RpcError>> {
    let existing = match &m.task_id {
        Some(id) => match app.store.get(id) {
            Some(t) if t.kind == Kind::Test => Some(t),
            _ => return None,
        },
        None => None,
    };
    let plan = match (plan(&m.message_id), &existing) {
        (Some(p), _) => p,
        (None, Some(_)) => Plan::Steps(vec![Step::Complete(None)]),
        (None, None) => return None,
    };
    let ctx = m.context_id.clone().unwrap_or_else(new_id);
    let task = match existing {
        Some(t) => {
            if m.context_id.as_ref().is_some_and(|c| c != &t.context_id) {
                return Some(Err(RpcError::invalid_params(
                    "message.contextId does not match the task's contextId",
                )));
            }
            if t.state.is_terminal() {
                return Some(Err(RpcError::unsupported(
                    "The task has ended: it accepts no more messages.",
                )));
            }
            app.store
                .add_history(&t.id, crate::intent::follow_up(m))
                .and_then(Result::ok)
                .unwrap_or(t)
        }
        None => {
            if let Plan::Message(text) = &plan {
                let reply = agent_message(&new_id(), &ctx, None, vec![text_part(text)]);
                return Some(Ok(Outcome::Message(reply)));
            }
            app.store.insert(TaskRec {
                id: new_id(),
                context_id: ctx,
                kind: Kind::Test,
                state: TaskState::Submitted,
                ts_ms: now_ms(),
                approve_url: String::new(),
                status_message: Value::Null,
                artifact: None,
                history: vec![m.raw.clone()],
                // Counted by the store when it keeps the task.
                history_bytes: 0,
                seq: 0,
                events: Vec::new(),
            })
        }
    };
    Some(Ok(start(app, task, plan, cfg, streaming).await))
}

async fn start(
    app: &Arc<App>,
    t: TaskRec,
    plan: Plan,
    cfg: &SendConfig,
    streaming: bool,
) -> Outcome {
    let steps = match plan {
        Plan::Steps(s) => s,
        Plan::Message(text) => vec![Step::Complete(Some(text))],
    };
    if streaming || cfg.return_immediately {
        let (a, id) = (app.clone(), t.id.clone());
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            run(a, id, steps).await;
        });
        Outcome::Task(t)
    } else {
        // Blocking: wait for a terminal or interrupted state.
        run(app.clone(), t.id.clone(), steps).await;
        Outcome::Task(app.store.get(&t.id).unwrap_or(t))
    }
}

/// CancelTask on a kit task.
pub fn cancel(app: &App, id: &str) -> Result<Value, RpcError> {
    match app.store.get(id) {
        Some(t) if t.state.is_terminal() => Err(RpcError::not_cancelable(id)),
        Some(_) => {
            set_state(app, id, TaskState::Canceled, None);
            app.store
                .get(id)
                .map(|t| t.to_json(None, true))
                .ok_or_else(|| RpcError::task_not_found(id))
        }
        None => Err(RpcError::task_not_found(id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_kit_ids_and_longest_prefix() {
        assert!(plan("m1").is_none());
        assert!(plan("deploy-123").is_none());
        assert!(matches!(
            plan("tck-message-response-abc"),
            Some(Plan::Message("Direct message response"))
        ));
        assert!(matches!(
            plan("tck-artifact-file-url-x"),
            Some(Plan::Steps(s)) if matches!(s.first(), Some(Step::Artifact { part, .. }) if part.get("url").is_some())
        ));
        assert!(matches!(
            plan("tck-something-else"),
            Some(Plan::Steps(s)) if matches!(s.as_slice(), [Step::Complete(None)])
        ));
    }
}

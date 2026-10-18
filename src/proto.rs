//! A2A 1.0 objects in ProtoJSON (camelCase names, `TASK_STATE_*` and `ROLE_*`
//! enum strings, parts without a `kind`), and how gate statuses map to task states.

use phygital_key_gate::Status;
use serde_json::{Value, json};

/// The Phygital Key A2A extension: one stable URI, documented in docs/extension.md.
pub const EXT: &str =
    "https://github.com/Phygital-Key/phygital-key-a2a/blob/main/docs/extension.md#v1";

/// A2A task states (proto `TaskState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Submitted and acknowledged.
    Submitted,
    /// Being worked on.
    Working,
    /// Needs more input (interrupted).
    InputRequired,
    /// Needs an authorization: here, the person's decision on the approval page (interrupted).
    AuthRequired,
    /// Approved: the artifact holds the gate's approval string (terminal).
    Completed,
    /// Expired, or the gate no longer knows the request (terminal).
    Failed,
    /// Withdrawn by the agent (terminal).
    Canceled,
    /// The person said no (terminal).
    Rejected,
}

impl TaskState {
    /// The ProtoJSON enum string.
    pub fn as_str(self) -> &'static str {
        match self {
            TaskState::Submitted => "TASK_STATE_SUBMITTED",
            TaskState::Working => "TASK_STATE_WORKING",
            TaskState::InputRequired => "TASK_STATE_INPUT_REQUIRED",
            TaskState::AuthRequired => "TASK_STATE_AUTH_REQUIRED",
            TaskState::Completed => "TASK_STATE_COMPLETED",
            TaskState::Failed => "TASK_STATE_FAILED",
            TaskState::Canceled => "TASK_STATE_CANCELED",
            TaskState::Rejected => "TASK_STATE_REJECTED",
        }
    }

    /// Parse a ProtoJSON enum string (for the ListTasks `status` filter).
    pub fn parse(s: &str) -> Option<TaskState> {
        [
            TaskState::Submitted,
            TaskState::Working,
            TaskState::InputRequired,
            TaskState::AuthRequired,
            TaskState::Completed,
            TaskState::Failed,
            TaskState::Canceled,
            TaskState::Rejected,
        ]
        .into_iter()
        .find(|t| t.as_str() == s)
    }

    /// Terminal states accept no more messages and cannot be canceled or subscribed to.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            TaskState::Completed | TaskState::Failed | TaskState::Canceled | TaskState::Rejected
        )
    }
}

/// The task state for a gate status. `approval` is the gate's approval string:
/// a task is COMPLETED only when the gate handed one out, never on a status alone.
/// `None` means "no change yet".
pub fn map_status(status: Status, approval: Option<&str>) -> Option<TaskState> {
    match status {
        Status::Pending => Some(TaskState::AuthRequired),
        Status::Approved => approval
            .filter(|a| !a.is_empty())
            .map(|_| TaskState::Completed),
        Status::Denied => Some(TaskState::Rejected),
        Status::Cancelled => Some(TaskState::Canceled),
        Status::Expired => Some(TaskState::Failed),
    }
}

/// A text part.
pub fn text_part(text: &str) -> Value {
    json!({"text": text, "mediaType": "text/plain"})
}

/// A data part (a JSON object).
pub fn data_part(data: Value) -> Value {
    json!({"data": data, "mediaType": "application/json"})
}

/// A message from this agent.
pub fn agent_message(
    message_id: &str,
    context_id: &str,
    task_id: Option<&str>,
    parts: Vec<Value>,
) -> Value {
    let mut m = json!({
        "messageId": message_id,
        "contextId": context_id,
        "role": "ROLE_AGENT",
        "parts": parts,
    });
    if let (Some(t), Some(o)) = (task_id, m.as_object_mut()) {
        o.insert("taskId".into(), Value::from(t));
    }
    m
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)] // serde_json `v["key"]` in test asserts
mod tests {
    use super::*;

    #[test]
    fn enum_strings_round_trip() {
        for s in [
            "TASK_STATE_SUBMITTED",
            "TASK_STATE_WORKING",
            "TASK_STATE_INPUT_REQUIRED",
            "TASK_STATE_AUTH_REQUIRED",
            "TASK_STATE_COMPLETED",
            "TASK_STATE_FAILED",
            "TASK_STATE_CANCELED",
            "TASK_STATE_REJECTED",
        ] {
            assert_eq!(TaskState::parse(s).map(TaskState::as_str), Some(s));
        }
        for bad in [
            "completed",
            "TASK_STATE_UNSPECIFIED",
            "TASK_STATE_CANCELLED",
            "",
        ] {
            assert_eq!(TaskState::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn terminal_states() {
        let terminal: Vec<_> = [
            "TASK_STATE_COMPLETED",
            "TASK_STATE_FAILED",
            "TASK_STATE_CANCELED",
            "TASK_STATE_REJECTED",
        ]
        .iter()
        .filter_map(|s| TaskState::parse(s))
        .collect();
        assert!(terminal.iter().all(|t| t.is_terminal()));
        assert!(!TaskState::AuthRequired.is_terminal());
        assert!(!TaskState::InputRequired.is_terminal());
        assert!(!TaskState::Working.is_terminal());
        assert!(!TaskState::Submitted.is_terminal());
    }

    #[test]
    fn gate_status_to_task_state() {
        assert_eq!(
            map_status(Status::Pending, None),
            Some(TaskState::AuthRequired)
        );
        assert_eq!(
            map_status(Status::Approved, Some("eyJ.x.y")),
            Some(TaskState::Completed)
        );
        assert_eq!(map_status(Status::Denied, None), Some(TaskState::Rejected));
        assert_eq!(
            map_status(Status::Cancelled, None),
            Some(TaskState::Canceled)
        );
        assert_eq!(map_status(Status::Expired, None), Some(TaskState::Failed));
    }

    #[test]
    fn never_completed_without_the_approval_string() {
        assert_eq!(map_status(Status::Approved, None), None);
        assert_eq!(map_status(Status::Approved, Some("")), None);
        // An approval string next to any other status never completes a task.
        for s in [
            Status::Pending,
            Status::Denied,
            Status::Cancelled,
            Status::Expired,
        ] {
            assert_ne!(map_status(s, Some("eyJ.x.y")), Some(TaskState::Completed));
        }
    }

    #[test]
    fn protojson_shapes() {
        assert_eq!(
            text_part("hi"),
            json!({"text": "hi", "mediaType": "text/plain"})
        );
        let d = data_part(json!({"a": 1}));
        assert_eq!(
            d,
            json!({"data": {"a": 1}, "mediaType": "application/json"})
        );
        assert!(d.get("kind").is_none());
        let m = agent_message("m1", "c1", Some("t1"), vec![text_part("x")]);
        assert_eq!(m["role"], "ROLE_AGENT");
        assert_eq!(m["messageId"], "m1");
        assert_eq!(m["contextId"], "c1");
        assert_eq!(m["taskId"], "t1");
        let m = agent_message("m1", "c1", None, vec![]);
        assert!(m.get("taskId").is_none());
    }
}

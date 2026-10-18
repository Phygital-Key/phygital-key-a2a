//! The A2A tasks this adapter made. Each task id is the gate's request id; the
//! gate stays the only source of truth for the decision, and this store keeps
//! what A2A needs on top: the context, the last state and when it changed.
//!
//! A task leaves TASK_STATE_AUTH_REQUIRED exactly once. A terminal task never
//! changes again (a completed standing approval stays COMPLETED after a revoke;
//! an approval that ran out after it was handed out stays COMPLETED).

use std::collections::HashMap;
use std::sync::Mutex;

use phygital_key_gate::{Gate, Status, View};
use phygital_key_verify::b64;
use serde_json::{Value, json};
use tokio::sync::watch;

use crate::intent::json_len;
use crate::proto::{EXT, TaskState, agent_message, data_part, map_status, text_part};
use crate::time::{iso, now_ms};

/// Most tasks kept. The oldest ended ones go first.
const MAX_TASKS: usize = 5000;
/// Most messages kept in one task's history (the first one always stays).
pub const MAX_HISTORY: usize = 20;
/// Most bytes (as JSON) kept in one task's history. The oldest follow-ups go first.
pub const MAX_TASK_HISTORY_BYTES: usize = 64 * 1024;
/// Most bytes (as JSON) of history kept across all tasks. When reached, the
/// oldest follow-ups of ended tasks go first, then new follow-ups are refused.
pub const MAX_STORE_HISTORY_BYTES: usize = 16 * 1024 * 1024;
/// Ended tasks are forgotten a day after they ended.
const KEEP_ENDED_MS: i64 = 24 * 3600 * 1000;

/// What the task asks the person for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// One exact action, once.
    Approval,
    /// A standing approval within limits.
    Standing,
    /// A conformance-kit task (`--features tck` only, never in a release).
    #[cfg(feature = "tck")]
    Test,
}

/// One task.
#[derive(Debug, Clone)]
pub struct TaskRec {
    /// Task id = gate request id.
    pub id: String,
    /// The A2A context.
    pub context_id: String,
    /// One-time or standing.
    pub kind: Kind,
    /// Current state.
    pub state: TaskState,
    /// When the state was last set (Unix ms).
    pub ts_ms: i64,
    /// The approval page link.
    pub approve_url: String,
    /// The status message for the current state.
    pub status_message: Value,
    /// The approval artifact, once completed.
    pub artifact: Option<Value>,
    /// The client's messages on this task, oldest first (the first one made it).
    pub history: Vec<Value>,
    /// The size of `history` as JSON (kept up to date by the store).
    pub history_bytes: usize,
    /// Creation order (tie-break for equal timestamps).
    pub seq: u64,
    /// Every change after creation, as stream events (`StreamResponse`), in
    /// order. Each open stream sends the ones it has not sent yet, so all
    /// streams of a task see the same events.
    pub events: Vec<Value>,
}

fn status_text(kind: Kind, state: TaskState, approve_url: &str, view: Option<&View>) -> String {
    match (state, kind) {
        (TaskState::AuthRequired, Kind::Approval) => format!(
            "Ask the person to open {approve_url} and approve or deny this exact request."
        ),
        (TaskState::AuthRequired, Kind::Standing) => format!(
            "Ask the person to open {approve_url} and approve or deny this standing approval (the page shows every limit)."
        ),
        (TaskState::Completed, Kind::Approval) => "Approved. Send `approval` with the action to the service, in the Phygital-Approval header (or the field the service names). It works once, for this exact request, before `exp`.".into(),
        (TaskState::Completed, Kind::Standing) => "Approved standing approval. For each action, send a message with one data part {\"use\": {\"standingId\": \"<this task id>\", \"request\": {...}}}, then send both `approval` (header Phygital-Approval) and `useToken` (header Phygital-Use) to the service. It lasts until `exp` or until its uses run out.".into(),
        (TaskState::Rejected, _) => "The person said no. Do not perform the action.".into(),
        (TaskState::Canceled, _) => "Cancelled. Do not perform the action.".into(),
        (TaskState::Failed, _) if view.is_none() => "Failed: the gate no longer has this request (it expired or was forgotten). Do not perform the action.".into(),
        (TaskState::Failed, _) if view.is_some_and(|v| v.approved_by.is_some()) => "Expired: the approval ran out before it was collected. Do not perform the action. Ask again only if the person wants to.".into(),
        (TaskState::Failed, _) => "Expired: nobody decided in time. Do not perform the action. Ask again only if the person wants to.".into(),
        _ => String::new(),
    }
}

impl TaskRec {
    /// A new task waiting for the person.
    pub fn new(view: &View, kind: Kind, context_id: String, first: Value) -> TaskRec {
        let mut t = TaskRec {
            id: view.id.clone(),
            context_id,
            kind,
            state: TaskState::AuthRequired,
            ts_ms: now_ms(),
            approve_url: view.approve_url.clone(),
            status_message: Value::Null,
            artifact: None,
            history_bytes: json_len(&first),
            history: vec![first],
            seq: 0,
            events: Vec::new(),
        };
        let text = status_text(kind, TaskState::AuthRequired, &view.approve_url, Some(view));
        t.status_message = t.message(vec![
            text_part(&text),
            data_part(json!({"approveUrl": view.approve_url, "id": view.id})),
        ]);
        t
    }

    fn message(&self, parts: Vec<Value>) -> Value {
        let mid = format!(
            "{}-{}",
            self.id,
            self.state
                .as_str()
                .trim_start_matches("TASK_STATE_")
                .to_ascii_lowercase()
        );
        agent_message(&mid, &self.context_id, Some(&self.id), parts)
    }

    /// Apply what the gate says. Only a task waiting for the person can change,
    /// and only once. Returns true when the state changed.
    pub fn apply(&mut self, view: Option<&View>, approval: Option<&str>, now: i64) -> bool {
        if self.state != TaskState::AuthRequired {
            return false;
        }
        let next = match view {
            Some(v) => match map_status(v.status, approval) {
                Some(s) => s,
                None => return false,
            },
            None => TaskState::Failed,
        };
        if next == TaskState::AuthRequired {
            return false;
        }
        self.state = next;
        self.ts_ms = now;
        let text = status_text(self.kind, next, &self.approve_url, view);
        let mut parts = vec![text_part(&text)];
        if next == TaskState::Completed
            && let (Some(v), Some(a)) = (view, approval)
        {
            let mut d = json!({"approval": a, "exp": v.exp, "approvedBy": v.approved_by});
            if self.kind == Kind::Standing
                && let Some(o) = d.as_object_mut()
            {
                o.insert("standingId".into(), Value::from(self.id.clone()));
            }
            let (name, description) = match self.kind {
                Kind::Approval => (
                    "phygital-key-approval",
                    "The person's signed approval of this exact request, for one service. It works once, before exp.",
                ),
                Kind::Standing => (
                    "phygital-key-approval",
                    "The person's signed standing approval. Each use also needs a use token from a use message.",
                ),
                #[cfg(feature = "tck")]
                Kind::Test => return false,
            };
            self.artifact = Some(json!({
                "artifactId": "approval",
                "name": name,
                "description": description,
                "parts": [data_part(d)],
                "extensions": [EXT],
            }));
        } else {
            parts.push(data_part(
                json!({"id": self.id, "approveUrl": self.approve_url}),
            ));
        }
        self.status_message = self.message(parts);
        let ev = self.update_events();
        self.events.extend(ev);
        true
    }

    fn kind_name(&self) -> &'static str {
        match self.kind {
            Kind::Approval => "approval",
            Kind::Standing => "standing",
            #[cfg(feature = "tck")]
            Kind::Test => "test",
        }
    }

    /// `TaskStatus`.
    pub fn status_json(&self) -> Value {
        let mut s = json!({"state": self.state.as_str(), "timestamp": iso(self.ts_ms)});
        if let (false, Some(o)) = (self.status_message.is_null(), s.as_object_mut()) {
            o.insert("message".into(), self.status_message.clone());
        }
        s
    }

    /// The `Task` object. `history`: at most this many of the latest history
    /// messages (None: all of them, 0: none).
    pub fn to_json(&self, history: Option<u64>, artifacts: bool) -> Value {
        let mut t = json!({
            "id": self.id,
            "contextId": self.context_id,
            "status": self.status_json(),
            "metadata": {EXT: {"approveUrl": self.approve_url, "type": self.kind_name()}},
        });
        let Some(o) = t.as_object_mut() else {
            return t;
        };
        if artifacts && let Some(a) = &self.artifact {
            o.insert("artifacts".into(), json!([a]));
        }
        let keep = history.map_or(self.history.len(), |n| {
            usize::try_from(n)
                .unwrap_or(usize::MAX)
                .min(self.history.len())
        });
        if keep > 0 {
            let msgs: Vec<Value> = self
                .history
                .iter()
                .skip(self.history.len() - keep)
                .map(|m| {
                    let mut m = m.clone();
                    if let Some(mo) = m.as_object_mut() {
                        mo.insert("taskId".into(), Value::from(self.id.clone()));
                        mo.insert("contextId".into(), Value::from(self.context_id.clone()));
                    }
                    m
                })
                .collect();
            o.insert("history".into(), Value::Array(msgs));
        }
        t
    }

    /// The streaming events for the change to the current state.
    pub fn update_events(&self) -> Vec<Value> {
        let mut out = Vec::new();
        if let Some(a) = &self.artifact {
            out.push(json!({"artifactUpdate": {
                "taskId": self.id,
                "contextId": self.context_id,
                "artifact": a,
                "append": false,
                "lastChunk": true,
            }}));
        }
        out.push(json!({"statusUpdate": {
            "taskId": self.id,
            "contextId": self.context_id,
            "status": self.status_json(),
        }}));
        out
    }
}

/// `ListTasks` filters and paging.
#[derive(Debug, Clone, Default)]
pub struct ListQuery {
    /// Only this context.
    pub context_id: Option<String>,
    /// Only this state.
    pub status: Option<TaskState>,
    /// Only tasks whose status changed at or after this time (Unix ms).
    pub after_ms: Option<i64>,
    /// 1 to 100.
    pub page_size: usize,
    /// Cursor from the previous page.
    pub page_token: Option<String>,
}

fn cursor(t: &TaskRec) -> String {
    b64::encode(format!("{}:{}", t.ts_ms, t.seq).as_bytes())
}

/// Read a page token. None when it is not one this agent made.
pub fn parse_cursor(token: &str) -> Option<(i64, u64)> {
    let raw = b64::decode(token)?;
    let s = String::from_utf8(raw).ok()?;
    let (ts, seq) = s.split_once(':')?;
    Some((ts.parse().ok()?, seq.parse().ok()?))
}

struct Inner {
    tasks: HashMap<String, TaskRec>,
    seq: u64,
    /// The sum of every task's `history_bytes`.
    history_bytes: usize,
}

/// A follow-up was not kept: the store-wide history limit is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryFull;

fn history_len(h: &[Value]) -> usize {
    h.iter().map(json_len).sum()
}

impl Inner {
    fn remove(&mut self, id: &str) {
        if let Some(t) = self.tasks.remove(id) {
            self.history_bytes = self.history_bytes.saturating_sub(t.history_bytes);
        }
    }

    /// Drop the oldest follow-ups of ended tasks until `need` more bytes fit
    /// under [`MAX_STORE_HISTORY_BYTES`]. True when they fit.
    fn make_room(&mut self, need: usize) -> bool {
        if self.history_bytes + need <= MAX_STORE_HISTORY_BYTES {
            return true;
        }
        let mut ended: Vec<(i64, u64, String)> = self
            .tasks
            .values()
            .filter(|t| t.state.is_terminal() && t.history.len() > 1)
            .map(|t| (t.ts_ms, t.seq, t.id.clone()))
            .collect();
        ended.sort();
        for (_, _, id) in ended {
            let Some(t) = self.tasks.get_mut(&id) else {
                continue;
            };
            while t.history.len() > 1 && self.history_bytes + need > MAX_STORE_HISTORY_BYTES {
                let gone = json_len(&t.history.remove(1));
                t.history_bytes = t.history_bytes.saturating_sub(gone);
                self.history_bytes = self.history_bytes.saturating_sub(gone);
            }
            if self.history_bytes + need <= MAX_STORE_HISTORY_BYTES {
                return true;
            }
        }
        false
    }
}

/// All tasks, plus a change counter streams wait on.
pub struct Store {
    inner: Mutex<Inner>,
    tick: watch::Sender<u64>,
}

impl Default for Store {
    fn default() -> Self {
        Store {
            inner: Mutex::new(Inner {
                tasks: HashMap::new(),
                seq: 0,
                history_bytes: 0,
            }),
            tick: watch::Sender::new(0),
        }
    }
}

impl Store {
    fn bump(&self) {
        self.tick.send_modify(|n| *n = n.wrapping_add(1));
    }

    /// Wait for any change.
    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.tick.subscribe()
    }

    /// Keep a new task.
    pub fn insert(&self, mut t: TaskRec) -> TaskRec {
        if let Ok(mut inner) = self.inner.lock() {
            inner.seq += 1;
            t.seq = inner.seq;
            t.history_bytes = history_len(&t.history);
            let now = now_ms();
            let old: Vec<String> = inner
                .tasks
                .values()
                .filter(|x| x.state.is_terminal() && now - x.ts_ms >= KEEP_ENDED_MS)
                .map(|x| x.id.clone())
                .collect();
            for id in old {
                inner.remove(&id);
            }
            while inner.tasks.len() >= MAX_TASKS {
                let oldest = inner
                    .tasks
                    .values()
                    .filter(|x| x.state.is_terminal())
                    .min_by_key(|x| (x.ts_ms, x.seq))
                    .or_else(|| inner.tasks.values().min_by_key(|x| (x.ts_ms, x.seq)))
                    .map(|x| x.id.clone());
                match oldest {
                    Some(id) => inner.remove(&id),
                    None => break,
                }
            }
            // The first message always stays (it is capped when parsed); make
            // room for it from ended tasks' follow-ups when that is possible.
            inner.make_room(t.history_bytes);
            inner.remove(&t.id);
            inner.history_bytes += t.history_bytes;
            inner.tasks.insert(t.id.clone(), t.clone());
        }
        self.bump();
        t
    }

    /// Change a task in place and wake its streams (conformance-kit tasks only).
    #[cfg(feature = "tck")]
    pub fn update(&self, id: &str, f: impl FnOnce(&mut TaskRec)) -> Option<TaskRec> {
        let t = {
            let mut inner = self.inner.lock().ok()?;
            let t = inner.tasks.get_mut(id)?;
            let before = t.history_bytes;
            f(t);
            t.history_bytes = history_len(&t.history);
            let t = t.clone();
            inner.history_bytes = inner.history_bytes.saturating_sub(before) + t.history_bytes;
            t
        };
        self.bump();
        Some(t)
    }

    /// Keep a follow-up message in a task's history: at most [`MAX_HISTORY`]
    /// messages and [`MAX_TASK_HISTORY_BYTES`] per task (the first message
    /// always stays, the oldest follow-ups go first), and at most
    /// [`MAX_STORE_HISTORY_BYTES`] across all tasks. None: no such task.
    pub fn add_history(&self, id: &str, message: Value) -> Option<Result<TaskRec, HistoryFull>> {
        let size = json_len(&message);
        let mut inner = self.inner.lock().ok()?;
        if !inner.tasks.contains_key(id) {
            return None;
        }
        if !inner.make_room(size) {
            return Some(Err(HistoryFull));
        }
        let t = inner.tasks.get_mut(id)?;
        let before = t.history_bytes;
        t.history.push(message);
        t.history_bytes += size;
        while t.history.len() > 1
            && (t.history.len() > MAX_HISTORY || t.history_bytes > MAX_TASK_HISTORY_BYTES)
        {
            let gone = json_len(&t.history.remove(1));
            t.history_bytes = t.history_bytes.saturating_sub(gone);
        }
        let t = t.clone();
        inner.history_bytes = inner.history_bytes.saturating_sub(before) + t.history_bytes;
        Some(Ok(t))
    }

    /// A task as it is now (without asking the gate).
    pub fn get(&self, id: &str) -> Option<TaskRec> {
        self.inner.lock().ok()?.tasks.get(id).cloned()
    }

    /// Apply what the gate says about one task.
    pub fn observe(
        &self,
        id: &str,
        view: Option<&View>,
        approval: Option<&str>,
    ) -> Option<TaskRec> {
        let (changed, t) = {
            let mut inner = self.inner.lock().ok()?;
            let t = inner.tasks.get_mut(id)?;
            (t.apply(view, approval, now_ms()), t.clone())
        };
        if changed {
            self.bump();
        }
        Some(t)
    }

    /// Ask the gate about a task still waiting for the person, then return it.
    pub fn refresh(&self, gate: &Gate, id: &str) -> Option<TaskRec> {
        self.sync(gate, id);
        self.get(id)
    }

    /// Ask the gate about a task still waiting for the person (no copy made).
    fn sync(&self, gate: &Gate, id: &str) {
        let waiting = match self.inner.lock() {
            Ok(inner) => inner
                .tasks
                .get(id)
                .is_some_and(|t| t.state == TaskState::AuthRequired),
            Err(_) => false,
        };
        if !waiting {
            return;
        }
        let r = gate.approval(id);
        let changed = {
            let Ok(mut inner) = self.inner.lock() else {
                return;
            };
            let Some(t) = inner.tasks.get_mut(id) else {
                return;
            };
            match r {
                // Pending: nothing to record.
                Ok((v, _)) if v.status == Status::Pending => return,
                // Decided: the approval string is collected right here.
                Ok((v, a)) => t.apply(Some(&v), a.as_deref(), now_ms()),
                Err(_) => t.apply(None, None, now_ms()),
            }
        };
        if changed {
            self.bump();
        }
    }

    /// Refresh every waiting task (before a list).
    pub fn refresh_all(&self, gate: &Gate) {
        let waiting: Vec<String> = match self.inner.lock() {
            Ok(inner) => inner
                .tasks
                .values()
                .filter(|t| t.state == TaskState::AuthRequired)
                .map(|t| t.id.clone())
                .collect(),
            Err(_) => return,
        };
        for id in waiting {
            self.sync(gate, &id);
        }
    }

    /// One page of tasks, newest status first. Returns (page, next token, total).
    pub fn list(&self, q: &ListQuery) -> Result<(Vec<TaskRec>, String, usize), &'static str> {
        let after = match &q.page_token {
            Some(tok) if !tok.is_empty() => {
                Some(parse_cursor(tok).ok_or("pageToken is not valid")?)
            }
            _ => None,
        };
        let Ok(inner) = self.inner.lock() else {
            return Ok((Vec::new(), String::new(), 0));
        };
        // Sort keys only; the page alone is copied.
        let mut all: Vec<(i64, u64, &str)> = inner
            .tasks
            .values()
            .filter(|t| q.context_id.as_ref().is_none_or(|c| c == &t.context_id))
            .filter(|t| q.status.is_none_or(|s| s == t.state))
            .filter(|t| q.after_ms.is_none_or(|a| t.ts_ms >= a))
            .map(|t| (t.ts_ms, t.seq, t.id.as_str()))
            .collect();
        all.sort_unstable_by(|a, b| b.cmp(a));
        let total = all.len();
        let mut start = all
            .into_iter()
            .filter(|(ts, seq, _)| after.is_none_or(|c| (*ts, *seq) < c));
        let page: Vec<TaskRec> = start
            .by_ref()
            .take(q.page_size)
            .filter_map(|(_, _, id)| inner.tasks.get(id).cloned())
            .collect();
        let more = start.next().is_some();
        drop(inner);
        let next = match (more, page.last()) {
            (true, Some(last)) => cursor(last),
            _ => String::new(),
        };
        Ok((page, next, total))
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)] // serde_json `v["key"]` in test asserts
mod tests {
    use super::*;
    use phygital_key_verify::Level;

    fn view(status: Status) -> View {
        View {
            id: "req1".into(),
            status,
            aud: "https://svc.example".into(),
            level: Level::High,
            request_jcs: "{}".into(),
            request_hash: "h".into(),
            approve_url: "http://localhost:7478/r/req1".into(),
            created: 0,
            deadline: 600,
            exp: Some(1_792_229_460),
            approved_by: if status == Status::Approved {
                Some("Laptop".into())
            } else {
                None
            },
            adds_key: false,
            first_key: false,
            removes_key: false,
            revokes: false,
            standing: None,
            revocation_signed: None,
            service_reported: None,
        }
    }

    fn task(kind: Kind) -> TaskRec {
        TaskRec::new(
            &view(Status::Pending),
            kind,
            "ctx".into(),
            json!({"messageId": "m1", "role": "ROLE_USER", "parts": [{"text": "x"}]}),
        )
    }

    #[test]
    fn new_task_asks_the_person() {
        let t = task(Kind::Approval);
        let j = t.to_json(None, true);
        assert_eq!(j["id"], "req1");
        assert_eq!(j["contextId"], "ctx");
        assert_eq!(j["status"]["state"], "TASK_STATE_AUTH_REQUIRED");
        let m = &j["status"]["message"];
        assert_eq!(m["role"], "ROLE_AGENT");
        assert_eq!(m["taskId"], "req1");
        assert_eq!(m["contextId"], "ctx");
        assert_eq!(
            m["parts"][0]["text"],
            "Ask the person to open http://localhost:7478/r/req1 and approve or deny this exact request."
        );
        assert_eq!(
            m["parts"][1]["data"]["approveUrl"],
            "http://localhost:7478/r/req1"
        );
        assert_eq!(m["parts"][1]["data"]["id"], "req1");
        assert_eq!(
            j["metadata"][EXT]["approveUrl"],
            "http://localhost:7478/r/req1"
        );
        assert!(j.get("artifacts").is_none());
        let ts = j["status"]["timestamp"].as_str().unwrap_or_default();
        assert!(ts.ends_with('Z') && ts.len() == 24, "{ts}");
        assert_eq!(j["history"][0]["messageId"], "m1");
        assert_eq!(j["history"][0]["taskId"], "req1");
        assert!(t.to_json(Some(0), true).get("history").is_none());
        assert_eq!(
            t.to_json(Some(5), true)["history"].as_array().map(Vec::len),
            Some(1)
        );
    }

    #[test]
    fn approved_completes_with_the_approval_artifact() {
        let mut t = task(Kind::Approval);
        assert!(t.apply(Some(&view(Status::Approved)), Some("APPROVAL"), 5));
        let j = t.to_json(None, true);
        assert_eq!(j["status"]["state"], "TASK_STATE_COMPLETED");
        let a = &j["artifacts"][0];
        assert_eq!(a["artifactId"], "approval");
        assert_eq!(a["name"], "phygital-key-approval");
        assert_eq!(a["extensions"][0], EXT);
        assert_eq!(a["parts"][0]["data"]["approval"], "APPROVAL");
        assert_eq!(a["parts"][0]["data"]["exp"], 1_792_229_460);
        assert_eq!(a["parts"][0]["data"]["approvedBy"], "Laptop");
        assert!(a["parts"][0]["data"].get("standingId").is_none());
        assert!(t.to_json(None, false).get("artifacts").is_none());
        let ev = t.update_events();
        assert_eq!(t.events, ev);
        assert!(ev[0].get("artifactUpdate").is_some());
        assert_eq!(
            ev[1]["statusUpdate"]["status"]["state"],
            "TASK_STATE_COMPLETED"
        );
        assert_eq!(ev[1]["statusUpdate"]["taskId"], "req1");
    }

    #[test]
    fn standing_artifact_names_the_standing_id() {
        let mut t = task(Kind::Standing);
        assert!(t.apply(Some(&view(Status::Approved)), Some("S"), 5));
        assert_eq!(
            t.to_json(None, true)["artifacts"][0]["parts"][0]["data"]["standingId"],
            "req1"
        );
    }

    #[test]
    fn approved_without_the_string_does_not_complete() {
        let mut t = task(Kind::Approval);
        assert!(!t.apply(Some(&view(Status::Approved)), None, 5));
        assert_eq!(t.state, TaskState::AuthRequired);
        assert!(t.artifact.is_none());
    }

    #[test]
    fn other_endings() {
        for (s, want, text) in [
            (
                Status::Denied,
                TaskState::Rejected,
                "The person said no. Do not perform the action.",
            ),
            (
                Status::Cancelled,
                TaskState::Canceled,
                "Cancelled. Do not perform the action.",
            ),
            (
                Status::Expired,
                TaskState::Failed,
                "Expired: nobody decided in time. Do not perform the action. Ask again only if the person wants to.",
            ),
        ] {
            let mut t = task(Kind::Approval);
            assert!(t.apply(Some(&view(s)), None, 9));
            assert_eq!(t.state, want);
            assert_eq!(t.ts_ms, 9);
            assert!(t.artifact.is_none());
            assert_eq!(t.status_message["parts"][0]["text"], text);
            assert_eq!(t.update_events().len(), 1);
            assert_eq!(t.events.len(), 1);
        }
        let mut t = task(Kind::Approval);
        assert!(t.apply(None, None, 9));
        assert_eq!(t.state, TaskState::Failed);
    }

    #[test]
    fn terminal_is_final() {
        let mut t = task(Kind::Standing);
        assert!(t.apply(Some(&view(Status::Approved)), Some("S"), 5));
        // Revoked later (the gate then says cancelled), or expired: still completed.
        assert!(!t.apply(Some(&view(Status::Cancelled)), None, 6));
        assert!(!t.apply(Some(&view(Status::Expired)), None, 7));
        assert!(!t.apply(None, None, 8));
        assert_eq!(t.state, TaskState::Completed);
        assert_eq!(t.ts_ms, 5);
        assert_eq!(t.events.len(), 2);
        let mut d = task(Kind::Approval);
        assert!(d.apply(Some(&view(Status::Denied)), None, 5));
        assert!(!d.apply(Some(&view(Status::Approved)), Some("LATE"), 6));
        assert_eq!(d.state, TaskState::Rejected);
        assert!(d.artifact.is_none());
    }

    #[test]
    fn pending_changes_nothing() {
        let mut t = task(Kind::Approval);
        let before = t.ts_ms;
        assert!(!t.apply(Some(&view(Status::Pending)), None, before + 100));
        assert_eq!(t.ts_ms, before);
    }

    fn rec(id: &str, ctx: &str, ts: i64, state: TaskState) -> TaskRec {
        let mut t = task(Kind::Approval);
        t.id = id.into();
        t.context_id = ctx.into();
        t.ts_ms = ts;
        t.state = state;
        t
    }

    #[test]
    fn history_keeps_follow_ups_bounded() {
        let s = Store::default();
        s.insert(task(Kind::Approval));
        for i in 0..30 {
            s.add_history(
                "req1",
                json!({"messageId": format!("f{i}"), "role": "ROLE_USER", "parts": []}),
            );
        }
        let t = s
            .get("req1")
            .map(|t| t.to_json(None, false))
            .unwrap_or_default();
        let h = t["history"].as_array().cloned().unwrap_or_default();
        assert_eq!(h.len(), MAX_HISTORY);
        assert_eq!(h[0]["messageId"], "m1");
        assert_eq!(h[MAX_HISTORY - 1]["messageId"], "f29");
        let last2 = s
            .get("req1")
            .map(|t| t.to_json(Some(2), false))
            .unwrap_or_default();
        assert_eq!(last2["history"][0]["messageId"], "f28");
        assert_eq!(last2["history"][1]["messageId"], "f29");
    }

    fn text_msg(id: &str, bytes: usize) -> Value {
        let base = json!({"messageId": id, "role": "ROLE_USER", "parts": [{"text": ""}]});
        let pad = bytes.saturating_sub(json_len(&base));
        json!({"messageId": id, "role": "ROLE_USER", "parts": [{"text": "x".repeat(pad)}]})
    }

    fn total(s: &Store) -> usize {
        s.inner
            .lock()
            .map(|i| i.history_bytes)
            .unwrap_or(usize::MAX)
    }

    fn real_total(s: &Store) -> usize {
        s.inner
            .lock()
            .map(|i| i.tasks.values().map(|t| history_len(&t.history)).sum())
            .unwrap_or(0)
    }

    #[test]
    fn history_bytes_per_task_are_capped() {
        let s = Store::default();
        s.insert(task(Kind::Approval));
        for i in 0..19 {
            s.add_history("req1", text_msg(&format!("f{i}"), 10_000));
        }
        let t = s.get("req1");
        let h = t.as_ref().map(|t| t.history.clone()).unwrap_or_default();
        assert!(h.len() < MAX_HISTORY);
        assert_eq!(h[0]["messageId"], "m1");
        assert_eq!(h[h.len() - 1]["messageId"], "f18");
        let bytes = history_len(&h);
        assert!(bytes <= MAX_TASK_HISTORY_BYTES, "{bytes}");
        assert_eq!(t.map(|t| t.history_bytes), Some(bytes));
        assert_eq!(total(&s), real_total(&s));
    }

    #[test]
    fn history_bytes_store_wide_are_capped() {
        let s = Store::default();
        let now = now_ms();
        // An ended task with follow-ups.
        s.insert(rec("ended", "c", now, TaskState::Completed));
        for i in 0..5 {
            assert!(matches!(
                s.add_history("ended", text_msg(&format!("e{i}"), 2000)),
                Some(Ok(_))
            ));
        }
        // Waiting tasks whose first messages fill the store up to 1 KB short.
        let mut n = 0;
        while total(&s) + 1000 < MAX_STORE_HISTORY_BYTES {
            let left = MAX_STORE_HISTORY_BYTES - total(&s) - 1000;
            let mut t = rec(&format!("w{n}"), "c", now + 1 + n, TaskState::AuthRequired);
            t.history = vec![text_msg("first", left.min(60_000))];
            s.insert(t);
            n += 1;
        }
        assert_eq!(s.get("ended").map(|t| t.history.len()), Some(6));
        // A follow-up on a waiting task: the ended task's oldest follow-ups make room.
        assert!(matches!(
            s.add_history("w0", text_msg("x1", 2000)),
            Some(Ok(_))
        ));
        let e = s.get("ended").map(|t| t.history).unwrap_or_default();
        assert_eq!(e.len(), 5);
        assert_eq!(e[0]["messageId"], "m1");
        assert_eq!(e[1]["messageId"], "e1");
        assert!(total(&s) <= MAX_STORE_HISTORY_BYTES);
        // Until none are left: then follow-ups are refused, and nothing changes.
        for i in 0..4 {
            assert!(matches!(
                s.add_history(&format!("w{}", i + 1), text_msg(&format!("y{i}"), 2000)),
                Some(Ok(_))
            ));
        }
        assert_eq!(s.get("ended").map(|t| t.history.len()), Some(1));
        let before = s.get("w5").map(|t| t.history.len());
        assert!(matches!(
            s.add_history("w5", text_msg("z", 2000)),
            Some(Err(HistoryFull))
        ));
        assert_eq!(s.get("w5").map(|t| t.history.len()), before);
        assert!(s.add_history("nope", text_msg("z", 10)).is_none());
        assert!(total(&s) <= MAX_STORE_HISTORY_BYTES);
        assert_eq!(total(&s), real_total(&s));
    }

    #[test]
    fn paging_with_equal_timestamps() {
        let s = Store::default();
        let now = now_ms();
        for i in 0..5 {
            s.insert(rec(&format!("t{i}"), "c", now, TaskState::AuthRequired));
        }
        let mut q = ListQuery {
            page_size: 2,
            ..Default::default()
        };
        let mut seen = Vec::new();
        while let Ok((page, next, _)) = s.list(&q) {
            seen.extend(page.iter().map(|t| t.id.clone()));
            if next.is_empty() {
                break;
            }
            q.page_token = Some(next);
        }
        // Same time: the later one (higher seq) first.
        assert_eq!(seen, ["t4", "t3", "t2", "t1", "t0"]);
    }

    #[test]
    fn paging_newest_first() {
        let s = Store::default();
        let now = now_ms();
        for i in 0..7 {
            s.insert(rec(
                &format!("t{i}"),
                if i % 2 == 0 { "even" } else { "odd" },
                now + i,
                TaskState::AuthRequired,
            ));
        }
        let mut q = ListQuery {
            page_size: 3,
            ..Default::default()
        };
        let mut seen = Vec::new();
        while let Ok((page, next, total)) = s.list(&q) {
            assert_eq!(total, 7);
            seen.extend(page.iter().map(|t| t.id.clone()));
            if next.is_empty() {
                break;
            }
            q.page_token = Some(next);
        }
        assert_eq!(seen, ["t6", "t5", "t4", "t3", "t2", "t1", "t0"]);
        let even = s.list(&ListQuery {
            page_size: 50,
            context_id: Some("even".into()),
            ..Default::default()
        });
        assert_eq!(
            even.map(|(p, n, t)| (p.len(), n, t)),
            Ok((4, String::new(), 4))
        );
        let after = s.list(&ListQuery {
            page_size: 50,
            after_ms: Some(now + 4),
            ..Default::default()
        });
        assert_eq!(after.map(|(p, _, _)| p.len()), Ok(3));
        let st = s.list(&ListQuery {
            page_size: 50,
            status: Some(TaskState::Completed),
            ..Default::default()
        });
        assert_eq!(st.map(|(p, _, t)| (p.len(), t)), Ok((0, 0)));
        assert!(
            s.list(&ListQuery {
                page_size: 5,
                page_token: Some("not a token".into()),
                ..Default::default()
            })
            .is_err()
        );
        // Exactly one full page: the last page has an empty token.
        let one = s.list(&ListQuery {
            page_size: 7,
            ..Default::default()
        });
        assert_eq!(one.map(|(p, n, _)| (p.len(), n)), Ok((7, String::new())));
    }

    #[test]
    fn store_is_bounded() {
        let s = Store::default();
        let now = now_ms();
        for i in 0..(MAX_TASKS + 10) {
            s.insert(rec(
                &format!("t{i}"),
                "c",
                now + i as i64,
                TaskState::Completed,
            ));
        }
        let all = s.list(&ListQuery {
            page_size: 1,
            ..Default::default()
        });
        assert_eq!(all.map(|(_, _, t)| t), Ok(MAX_TASKS));
        assert!(s.get("t0").is_none());
        assert!(s.get(&format!("t{}", MAX_TASKS + 9)).is_some());
    }
}

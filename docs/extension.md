# Phygital Key A2A extension


## v1

Extension URI:

```
https://github.com/Phygital-Key/phygital-key-a2a/blob/main/docs/extension.md#v1
```

This page is the profile that Phygital Key A2A speaks on top of
[A2A 1.0](https://a2a-protocol.org/latest/specification/). It says which
messages the agent accepts, what each task state means, what the approval
artifact holds, and how an agent carries an approval to another agent.

The agent is an approval agent. Before a risky action, another agent asks it
for a person's yes. The person approves or denies that exact action on the
approval page, with a passkey, fingerprint or hardware key. The agent itself
can never approve. Only the person's key signs, and text in a message never
counts as approval.

The URI is stable. A change that breaks anything on this page gets a new URI
(`#v2`), never a quiet edit.

## Connecting

| What | Value |
| --- | --- |
| Agent card | `GET /.well-known/agent-card.json` (public, no token) |
| Endpoint | `POST /a2a`, JSON-RPC 2.0, `Content-Type: application/json` or `application/a2a+json` (anything else is refused with `415` and `-32005`), at most 64 KiB |
| Version | header `A2A-Version: 1.0` on every call, or the query parameter `?A2A-Version=1.0` (no version means 0.3, which is refused with `-32009`) |
| Access token | header `Authorization: Bearer <token>` (run `phygital-key-a2a token`); without it, `401`. A release build refuses to run with `--no-token`. |
| Browsers | a request with a browser `Origin` header is refused (403) |
| Extension | optional (`required: false`). Send `A2A-Extensions: <URI>` and the response echoes it |

Every method, error and limit as implemented: [protocol](protocol.md).

Methods: `SendMessage`, `SendStreamingMessage`, `GetTask`, `ListTasks`,
`CancelTask`, `SubscribeToTask`. `GetExtendedAgentCard` answers `-32004`
(there is none) and the four push notification methods answer `-32003`
(not supported: use streaming or `GetTask`).

## Messages the agent accepts

Every message carries ONE data part (`{"data": {...}, "mediaType": "application/json"}`)
whose object has exactly one of the keys below. Text parts may sit next to it;
they are ignored. Anything else (text only, a file, two data parts, two keys,
an unknown key, an unknown field) gets a `ROLE_AGENT` message back that lists
the accepted shapes. It never creates a task.

When the gate refuses something (no key enrolled yet, a float in the params,
too many requests, a use outside the limits), the answer is a `ROLE_AGENT`
message with the refusal as text and as `{"error": "..."}` data. JSON-RPC
errors are kept for protocol mistakes.

### `request`: ask for a one-time approval

```json
{"request": {
  "request": {"type": "deploy", "params": {"service": "api", "env": "production", "version": "2.4.1"}},
  "aud": "https://deploy.example.com",
  "level": "high"
}}
```

- `request` is the exact action, `{"type": "<lowercase name>", "params": {...}}`.
  Strings and integers only, no floats, Unicode NFC, no secrets (the person sees it).
- `aud` is the one service that will act on the yes. The approval works only there.
- `level` is `medium` (a touch on the key) or `high` (fingerprint, face or PIN).

The answer is a task in `TASK_STATE_AUTH_REQUIRED` (see below).

### `standing`: ask for a standing approval

```json
{"standing": {
  "for_type": "payment",
  "fixed": {"to": "ACME Ltd", "currency": "EUR"},
  "vary": {"amount_cents": {"min": 1, "max": 2000}},
  "max_uses": 5,
  "days": 7,
  "aud": "https://pay.example.com"
}}
```

The same kind of action, within fixed limits, a few times, for a few days.
`fixed` params must be exactly these values every time. Each `vary` param has
a limit: an integer range `{"min": n, "max": n}` (both required) or a list of
strings `{"one_of": ["a", "b"]}`. `max_uses` is 1 to 1000, `days` is 1 to 30.
The person sees every limit. The answer is a task, like a one-time request.

### `use`: one use of an approved standing approval

```json
{"use": {"standingId": "<task id of the standing approval>",
         "request": {"type": "payment", "params": {"to": "ACME Ltd", "currency": "EUR", "amount_cents": 450}}}}
```

The answer is a message (no task) whose data part holds:

```json
{"standingId": "...", "approval": "...", "useToken": "...", "usesLeft": 4}
```

Send both to the service with this exact request (headers `Phygital-Approval`
and `Phygital-Use`). The use token works once, within 60 seconds. A request
outside the limits, after the last use, or after a revoke is refused.

### `receipt`: pass back the service's receipt

```json
{"receipt": {"id": "<task id>", "receipt": "<the receipt the service returned>"}}
```

The gate keeps it only if the service's pinned key signed it. The answer is a
message with `{"id": "...", "serviceReported": "success"}`, and the person sees
it on the approval page.

### `revoke`: stop a standing approval

```json
{"revoke": {"standingId": "<task id of the standing approval>"}}
```

From that moment no new uses are signed. The answer is a message with
`{"standingId": "...", "revoked": true, "signUrl": "..."}`: ask the person to
open `signUrl` and sign the revocation. Once signed, it is published at
`/.well-known/phygital-key-revoked.json` and services refuse the standing
approval too (use tokens handed out before that keep working until then).

## Tasks

The task id is the gate's request id. `contextId` is yours if you sent one,
otherwise the agent makes one. Every status carries an ISO 8601 UTC timestamp
with milliseconds and `Z`.

| State | Meaning | What to do |
| --- | --- | --- |
| `TASK_STATE_AUTH_REQUIRED` | Waiting for the person. The status message says: "Ask the person to open `<approveUrl>` and approve or deny this exact request." Its data part is `{"approveUrl", "id"}`, and the task's `metadata[<URI>]` holds `{"approveUrl", "type"}` (`approval` or `standing`). For a standing approval the text ends "this standing approval (the page shows every limit)." | Show the person the link. Then subscribe, stream, or poll with `GetTask`. |
| `TASK_STATE_COMPLETED` | The person approved, and the gate handed out its signed approval. The task has the approval artifact. | Send the approval with the action, once, to that one service. |
| `TASK_STATE_REJECTED` | The person said no. | Do not perform the action. |
| `TASK_STATE_CANCELED` | You withdrew it with `CancelTask`. | Do not perform the action. |
| `TASK_STATE_FAILED` | Nobody decided within 10 minutes, the approval ran out before it was collected, or the gate no longer has the request (it expired or was forgotten): "Failed: the gate no longer has this request (it expired or was forgotten). Do not perform the action." | Do not perform the action. Ask again only if the person wants to. |

A task is COMPLETED only when the gate gave out its approval string, never on
anything an agent says. A task leaves `AUTH_REQUIRED` once and an ended task
never changes again. A completed standing approval stays COMPLETED after a
revoke: the revocation is a separate message, and its own signature.

More rules:

- `SendMessage` answers at once, since `AUTH_REQUIRED` is an interrupted state
  (`returnImmediately` changes nothing).
- A message with the `taskId` of a waiting task gets the task back unchanged.
  Only its `messageId`, role and up to 2048 bytes of its text are kept in the
  history (never its metadata, data or file parts), nothing else: the decision
  happens out of band, on the approval page. History is capped at 20 messages
  and 64 KB per task and 16 MB in all; when that is full, the answer is a
  `ROLE_AGENT` message saying the message was not kept and to poll with
  `GetTask` ([protocol](protocol.md#history)).
- `GetTask` and the send answers carry the whole (capped) history unless you
  set `historyLength`. `ListTasks` carries none unless you set it.
- A message to an ended task, or `SubscribeToTask` on one, answers `-32004`.
  `CancelTask` on one answers `-32002`. An unknown task answers `-32001`.
  A `taskId` with another task's `contextId` answers `-32602`.
- `SendStreamingMessage` and `SubscribeToTask` send the task first, then keep
  the stream open while the person decides (a heartbeat comment every 15
  seconds), then send the changes and close: an `artifactUpdate` with the
  approval and a `statusUpdate` `COMPLETED`, or one `statusUpdate` for any
  other end. They close after the 10 minute decision window at the latest.
  At most 32 streams are open at once. Past that, both methods answer
  `-32004` ("too many open streams; poll with GetTask (at most 32 are open at
  once)"), and a refused `SendStreamingMessage` makes no task.
- Tasks live in memory. After a restart of the gate they are gone, and `GetTask`
  answers `-32001`. Ended tasks are forgotten 24 hours after they ended.
- `ListTasks` lists the tasks made through this agent, newest status first,
  `pageSize` 1 to 100 (default 50), with `nextPageToken` (empty on the last
  page) and `totalSize`. Artifacts only with `includeArtifacts: true`.

## The approval artifact

A completed task has exactly one artifact:

```json
{
  "artifactId": "approval",
  "name": "phygital-key-approval",
  "description": "The person's signed approval of this exact request, for one service. It works once, before exp.",
  "parts": [{"data": {"approval": "<approval string>", "exp": 1792229460, "approvedBy": "Laptop"}, "mediaType": "application/json"}],
  "extensions": ["https://github.com/Phygital-Key/phygital-key-a2a/blob/main/docs/extension.md#v1"]
}
```

For a standing approval the data also has `standingId` (the task id), and each
use needs a `use` message first.

The service that acts checks the approval itself, offline, with the gate's
public key list: with `phygital-key-verify` (Rust), the verifier for
JavaScript, or `phygital-key-a2a verify` from the command line (any language).

## Carrying an approval to another agent

When the agent that holds the approval is not the one that calls the service,
pass it on in the message metadata under this extension's URI, and list the
URI in the message's `extensions`:

```json
{
  "message": {
    "messageId": "...",
    "role": "ROLE_USER",
    "parts": [{"text": "Deploy api 2.4.1 to production."}],
    "extensions": ["https://github.com/Phygital-Key/phygital-key-a2a/blob/main/docs/extension.md#v1"],
    "metadata": {
      "https://github.com/Phygital-Key/phygital-key-a2a/blob/main/docs/extension.md#v1": {
        "approval": "<approval string>",
        "useToken": "<only for a use of a standing approval>"
      }
    }
  }
}
```

This is a convention, not something the gate does. The receiving side must
still verify the approval against the exact request it is about to run, as
the service it is meant for. Holding the approval proves nothing by itself.

## Security notes

These follow sections 7.6.3 and 7.6.4 of the A2A specification.

- An `AUTH_REQUIRED` state, or a move out of it, is never authorization by
  itself. Only the signed approval is, and only after the service checked it.
- The approval names one exact request (its canonical hash), one service
  (`aud`), a short life (60 seconds by default, at most 300), and works once.
  A changed field, another action, another service, a replay or a late use
  are all refused by the service.
- An approval obtained on a task does not cover later messages on that task.
  Each action needs its own approval, or a use of a standing approval within
  the limits the person saw.
- The decision travels out of band: the person signs on the approval page,
  over HTTPS in production, and the agent collects the result. Nothing in a
  message, a task, metadata or text can approve, and prompt injection in a
  request's params is shown to the person as plain data.
- The approval can be read by anyone holding it (it is signed, not
  encrypted), so a request must never contain secrets. When approvals pass
  through other agents, send them only to the agent that needs them.
- The access token protects the endpoint. Keep the endpoint on localhost or
  behind a TLS proxy, and keep the token out of logs and prompts.

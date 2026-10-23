# Security

Phygital Key A2A is pre-audit: no outside security firm has reviewed it yet.

## Report a problem

Please report a security problem privately, not in a public issue:

**[Report a vulnerability](https://github.com/Phygital-Key/phygital-key-a2a/security/advisories/new)** (GitHub private vulnerability reporting).

Say what you found, how to reproduce it, and which version you ran (`phygital-key-a2a --version`). If the problem is in the shared engine (the approval page, keys, standing approvals, receipts, the secure install, signed updates or the verifier) and not in the A2A side, you can report it here or in the [base repository](https://github.com/Phygital-Key/phygital-key/security/advisories/new).

We aim to reply within 7 days, and to publish a fix for every confirmed issue. Please keep the problem private until a fixed release is out. If you would like credit, say which name or GitHub account to use.

## Policy and threat model

The security policy, what is in scope, the threat model and the internal reviews are shared by every Phygital Key product and live in the base:

- [Security policy](https://github.com/Phygital-Key/phygital-key/blob/main/SECURITY.md)
- [Threat model](https://github.com/Phygital-Key/phygital-key/blob/main/docs/threat-model.md)
- [Security reviews](https://github.com/Phygital-Key/phygital-key/blob/main/docs/security-reviews.md)

## Hardening on the A2A side

- A release build refuses `--no-token` (exit code 2): the endpoint always needs the access token. Only the test-only `tck` build accepts the option.
- History is capped: follow-ups keep only their `messageId`, role and 2048 bytes of text, the first message at most 32 KB, each task at most 20 messages and 64 KB, all tasks 16 MB.
- At most 32 streams are open at once; past that the streaming methods answer `-32004` and make no task.
- Request bodies are `application/json` or `application/a2a+json`, at most 64 KiB; anything else is refused before it is read as JSON-RPC.

Every limit: [protocol](docs/protocol.md#limits).

What this repository adds to check: the JSON-RPC endpoint at `/a2a` and its streams (`src/server.rs`, `src/rpc.rs`), the message shapes (`src/intent.rs`), the task states and the approval artifact (`src/proto.rs`, `src/store.rs`), the public agent card (`src/card.rs`), and the test-only `tck` feature (`src/tck.rs`), which must never reach a release. The security notes of the A2A profile: [extension](docs/extension.md#security-notes). The tests for them: [testing](docs/testing.md).

# Changelog

Changes to Phygital Key A2A. Changes to the shared engine (approval page, keys, secure install, updates, verifier) are listed in the [base changelog](https://github.com/Phygital-Key/phygital-key/blob/main/CHANGELOG.md).

## 0.1.0

First release. Phygital Key A2A is a new product on the shared base [Phygital-Key/phygital-key](https://github.com/Phygital-Key/phygital-key), next to [Phygital Key MCP](https://github.com/Phygital-Key/phygital-key-mcp). It uses the same approval page and the same approval format (`pka.1`), so a service that already checks Phygital Key approvals needs no change.

- An A2A 1.0 agent over JSON-RPC 2.0 at `/a2a`, with the header `A2A-Version: 1.0` and the bearer access token from `phygital-key-a2a token`. Requests may be `application/json` or `application/a2a+json`. Methods: `SendMessage`, `SendStreamingMessage`, `GetTask`, `ListTasks`, `CancelTask`, `SubscribeToTask`. Push notifications answer `-32003` and `GetExtendedAgentCard` answers `-32004`.
- A public agent card at `/.well-known/agent-card.json` with five skills, streaming on, push notifications off, a bearer security scheme, `ETag`, `Last-Modified` and `Cache-Control: public, max-age=300`.
- Five message shapes in one data part: `request`, `standing`, `use`, `receipt`, `revoke`. A request becomes a task in `TASK_STATE_AUTH_REQUIRED`; approved, it is `TASK_STATE_COMPLETED` with the `phygital-key-approval` artifact; denied, `TASK_STATE_REJECTED`; withdrawn, `TASK_STATE_CANCELED`; not decided in 10 minutes, `TASK_STATE_FAILED`.
- The extension profile at [docs/extension.md](docs/extension.md#v1), URI `https://github.com/Phygital-Key/phygital-key-a2a/blob/main/docs/extension.md#v1`, with a convention for passing an approval to another agent in message metadata.
- Streams send the task first, stay open with a heartbeat comment every 15 seconds while the person decides, then send the result and close.
- `ListTasks` with filters (`contextId`, `status`, `statusTimestampAfter`), pages of 1 to 100 tasks and `includeArtifacts`.
- Hardening:
  - A release build refuses `--no-token`: it prints `phygital-key-a2a: --no-token is not allowed: anyone on this machine could read approvals. Use the access token (phygital-key-a2a token).` and exits with code 2. Only the test-only `tck` build accepts it.
  - History is capped. A follow-up keeps only its `messageId`, role and 2048 bytes of text, never metadata, data or file parts. The first message keeps its A2A fields, with each text part cut to 2 KB, metadata over 2 KB left out, 32 KB in all. Per task at most 20 messages and 64 KB; across all tasks 16 MB, after which a follow-up gets a `ROLE_AGENT` message saying to poll with `GetTask`. `ListTasks` returns no history unless `historyLength` is set.
  - At most 32 streams are open at once; past that, `SendStreamingMessage` and `SubscribeToTask` answer `-32004`, and a refused `SendStreamingMessage` makes no task.
- 165 end-to-end and red-team checks with the official A2A JavaScript client (`@a2a-js/sdk` 1.3.0), a real Chromium with a virtual passkey and the demo service.
- The official conformance kit (`a2a-tck`, MUST level, JSON-RPC) in CI: 81 passed, 0 failed with the test-only `tck` build. The release build is built without that feature, and the kit cannot run against it (it needs `--no-token`).
- Program `phygital-key-a2a`, port 7478 (demo service 7479), data folder `~/.phygital-key-a2a`, environment `PHYGITAL_KEY_A2A_HOME`, `_PORT`, `_BIND`, `_ORIGIN`.
- Secure install: user `_phygitalkeya2a` and folder `/Library/PhygitalKey/a2a` on macOS (launchd labels `io.phygitalkey.a2a.gate`, `io.phygitalkey.a2a.update`), user `phygitalkey-a2a` and folders `/usr/lib/phygital-key-a2a`, `/var/lib/phygital-key-a2a` on Linux (units `phygital-key-a2a.service`, `phygital-key-a2a-update.{service,timer}`), admin helper `phygital-key-a2a-admin`.
- Container `ghcr.io/phygital-key/phygital-key-a2a` (`linux/amd64`, `linux/arm64`) and Homebrew `phygital-key/tap/phygital-key-a2a`.
- A daily workflow (`sync-base.yml`) moves this product to the base's latest commit after the tests and the 165 end-to-end checks pass.

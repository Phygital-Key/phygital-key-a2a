# AGENTS.md

Notes for AI coding agents (Claude Code, Codex, Cursor, Copilot and others) and human contributors who change this repository, Phygital Key A2A. Read this before you change code, tests or docs.

> [!NOTE]
> If you are an agent **using** this A2A agent (asking a person to approve an action, not editing this code), read [docs/connect-your-agent.md](docs/connect-your-agent.md) and the [extension profile](docs/extension.md) instead. The short version: you can ask, you can never approve; wait while the task is in `TASK_STATE_AUTH_REQUIRED`; send the exact approved request with the artifact's `approval`; on `TASK_STATE_REJECTED`, `TASK_STATE_CANCELED` or `TASK_STATE_FAILED`, do not act.

## What this repository is

Phygital Key A2A is one product built on the shared base [Phygital-Key/phygital-key](https://github.com/Phygital-Key/phygital-key). The base holds the engine: the verifier (`phygital-key-verify`), the gate library (`phygital-key-gate`: approval page, keys, standing approvals, receipts, audit log, secure install, signed updates and the shared commands), the spec, the test vectors and the shared docs. The base publishes no programs and no releases.

This repository adds only what is A2A:

- the product's identity (name `Phygital Key A2A`, program `phygital-key-a2a`, id `a2a`, port 7478, repository `Phygital-Key/phygital-key-a2a`, its pinned release keys), as a `Product` in `src/main.rs`;
- the A2A adapter (the base's `Adapter` trait): the JSON-RPC 2.0 endpoint at `/a2a` (A2A 1.0, with server-sent events for streaming), the public agent card at `/.well-known/agent-card.json`, and the connect hints;
- the message shapes, the task states and the approval artifact, documented as an A2A extension ([docs/extension.md](docs/extension.md));
- the test behaviours for the conformance kit, behind the `tck` cargo feature;
- the end-to-end tests, the CI, the release workflow, the container, and the A2A docs.

Everything else (every command, the approval page, the data folder, the secure install) comes from `phygital-key-gate`. `phygital_key_gate::run(&PRODUCT, A2a)` in `src/main.rs` starts it. There is no stdio mode and no push notifications. Status: pre-audit, not released yet.

## Layout

```
Cargo.toml                       the product crate: phygital-key-gate and phygital-key-verify from the base, axum, tokio, serde; feature `tck`; unsafe_code = "forbid"
Cargo.lock                       pins every dependency (CI builds with --locked)
release-keys.pem                 the product's pinned release public keys (P-256 PEM); src/main.rs embeds it, release.yml checks the signature against it; empty until scripts/release-key.sh runs
LICENSE, NOTICE                  Apache-2.0, and the credit line "Created by Phygital-Key" (must stay)
README.md                        the product page
CHANGELOG.md, SECURITY.md, CONTRIBUTING.md
AGENTS.md, CLAUDE.md             this file, and the pointer to it
src/main.rs                      PRODUCT and the A2a adapter (path /a2a, public router, connect and demo hints)
src/card.rs                      the agent card, its ETag, Last-Modified and Cache-Control
src/server.rs                    the /a2a route, method dispatch, SendMessage, GetTask, ListTasks, CancelTask, the two streams
src/rpc.rs                       the JSON-RPC envelope, the A2A error codes with google.rpc.ErrorInfo, the A2A-Version check
src/intent.rs                    the SendMessage checks and the five message shapes (request, standing, use, receipt, revoke)
src/proto.rs                     A2A 1.0 objects in ProtoJSON, TaskState, the gate status to task state mapping, the extension URI
src/store.rs                     the task store in memory: capped history, events for streams, ListTasks paging
src/time.rs                      ISO 8601 timestamps
src/tck.rs                       conformance-kit behaviours, compiled only with --features tck
tests/e2e/run.mjs                172 end-to-end and red-team checks
tests/e2e/package.json, package-lock.json   pinned @a2a-js/sdk 1.3.0 and Playwright
packaging/container/Dockerfile   ghcr.io/phygital-key/phygital-key-a2a: distroless cc-debian12:nonroot (pinned by digest), uid 65532, data in /data
packaging/npm/                   build.sh makes the npm packages from a release's archives (the launcher phygital-key-a2a and one package per target), publish.sh publishes them
.github/workflows/ci.yml         CI on every push to main, every pull request, and when the sync starts it
.github/workflows/release.yml    on a v* tag: test, build 5 targets, attest, sign release.txt and release2.txt (P-256, and ML-DSA-65 once pinned), publish, then npm and the container
.github/workflows/sync-base.yml  daily: move to the base's latest main commit, test, push, start CI
docs/                            the A2A guides only (getting-started, connect-your-agent, protocol, extension, testing) and images/

Generated from the base's templates (never edit by hand):
install-service.sh               the secure installer for this product
install.sh                       the user install script
tests/secure-install.sh          the CI test of the secure install
scripts/release-key.sh           owner only, once: makes the release key, pins it
scripts/package-manifests.sh     writes the Homebrew, Scoop and winget manifests for a published release
```

While the base and this product are built side by side, `Cargo.toml` takes the two base crates as path dependencies (`../repo/crates/...`). For publishing they switch to the git base (`Phygital-Key/phygital-key`, branch `main`); `Cargo.lock` then pins the exact base commit, and the sync moves it.

## Files made from the base

The five generated files are the base's templates in `crates/phygital-key-gate/assets/` (`install-service.sh`, `install.sh`, `secure-install-test.sh`, `release-key.sh`, `package-manifests.sh`) with two blocks filled in for this product: `# BEGIN PRODUCT` (`P_NAME`, `P_BIN`, `P_ID`, `P_REPO`, `P_PORT`, `P_VERSION`, and `P_PATH=/a2a` in the test) and, in the two installers, `# BEGIN RELEASE KEYS` from `release-keys.pem`. The program writes them:

```bash
cargo build --release --locked
target/release/phygital-key-a2a product-files --out .
```

The CI job `files made from the base are up to date` runs exactly that and fails on `git diff --exit-code`. So:

- Never edit a generated file. Change the template in the base, or the product values in `src/main.rs`, or `release-keys.pem`, then run `product-files` and commit the result.
- After you change `version` in `Cargo.toml`, run `product-files` again: `P_VERSION` comes from it.
- `scripts/release-key.sh` writes `release-keys.pem` and runs `product-files` itself.

## Changing shared behaviour

Anything that is not A2A lives in the base: the approval page, enrollment, keys and key history, standing approvals, revocations, receipts, the audit log, the HTTP guard (host, `Origin`, token), the request rules (no floats, NFC, the flood limits), the commands `serve`, `demo`, `enroll`, `token`, `keys`, `verify`, `install-service`, `update` and the rest, the secure install, the signed updates, the verifier and the spec. Change those in [Phygital-Key/phygital-key](https://github.com/Phygital-Key/phygital-key), with its tests and vectors, following its [AGENTS.md](https://github.com/Phygital-Key/phygital-key/blob/main/AGENTS.md). Do not copy base code into this repository to change it here.

A change in the base reaches this product through the daily sync, or at once if you run `cargo update -p phygital-key-gate -p phygital-key-verify` here.

## How the sync works

`.github/workflows/sync-base.yml` runs every day at 05:23 UTC (and by hand):

1. `cargo update -p phygital-key-gate -p phygital-key-verify` moves `Cargo.lock` to the base's latest `main` commit.
2. It builds the release binary and runs `product-files --out .`. If nothing changed, it stops.
3. It runs `cargo test --locked` and the 172 end-to-end checks.
4. Only if both pass, it commits (`Update the base to <commit>`), pushes to `main`, and starts the full CI (`gh workflow run ci.yml`), which adds every OS, the `tck` build, the conformance kit, the secure install and the container.

A failing base change is never pushed here: the job stops before the commit.

## Rules that must never break

1. **Nothing can approve.** No method, message shape, metadata key, header, flag, route, feature or test hook may approve, sign for the person, complete a gate task or skip the check. A task reaches `TASK_STATE_COMPLETED` only when the gate hands out its approval string (`map_status` in `src/proto.rs`, and its unit test `never_completed_without_the_approval_string`).
2. **The `tck` feature stays out of releases.** Its tasks never touch the gate, and it is the only build that accepts `--no-token` (a release prints the refusal and exits with code 2, `src/main.rs`). Never enable it in `release.yml`, the container or an install script, and never make a `tck-` message affect a real request.
3. **No LLM in the gate or the approval page, ever.** The page shows the canonical request built by the gate, never text from the agent. Text parts are ignored.
4. **A follow-up message never changes a waiting task.** It is only kept in the history. The decision happens on the approval page.
5. **Fail closed.** Unclear, malformed, unknown or out of range is refused. Unknown fields in a message shape are refused (`deny_unknown_fields`). Never normalize text, never accept floats.
6. **Never log or print secrets.** The access token lives only in `<home>/access-token` (0600). The agent card holds no token (an end-to-end check tests it).
7. **Keep the generated files generated.** See above.
8. **No new dependencies without asking.** Open an issue first. Licences: Apache-2.0, MIT, BSD or similar.
9. **The release signing key lives only in the GitHub environment secret** `RELEASE_SIGNING_KEY` of the environment `release`, which only a run from a `v*` tag gets. Never put a private key in the repository, a workflow, an issue, a test or a log. Never run `scripts/release-key.sh` (owner only, in the owner's own terminal), never approve a release run, and never push a `v*` tag unless the owner asked: a pushed tag signs and publishes a release.
10. **Keep the credit.** `NOTICE` keeps the line "Created by Phygital-Key". Never write any real person's name anywhere, and add no AI attribution lines to docs, commits or release notes.
11. **Docs in plain words, no em dashes.** Shared topics are documented only in the base; link to them with `https://github.com/Phygital-Key/phygital-key/blob/main/docs/<page>.md`. Relative links here must resolve.

Changing a message shape, a task state, a status text, the artifact or the agent card also means: update [docs/extension.md](docs/extension.md) (a breaking change gets a new extension URI, `#v2`, and `EXT` in `src/proto.rs`), [docs/protocol.md](docs/protocol.md), the `HELP` text in `src/intent.rs`, the card's skills in `src/card.rs`, and the end-to-end checks.

## Build and test

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features tck -- -D warnings
cargo test --locked
cargo test --locked --features tck
```

`cargo test` runs 48 unit tests, and 49 with `--features tck` (the extra one is in `src/tck.rs`).

End-to-end and red-team checks (Node 20+, a release build, Chromium from Playwright):

```bash
cargo build --release --locked
cd tests/e2e && npm ci && npx playwright install chromium
PKA_BIN=../../target/release/phygital-key-a2a node run.mjs
```

It prints one `PASS` or `FAIL` line per check and `172 passed, 0 failed` at the end, and exits non-zero on any failure. It first checks that `serve --no-token` is refused (port 17485), then starts its own gate in demo mode on port 17478 and the demo service on 17479 (`PKA_PORT` moves all three), in a temporary data folder.

The conformance kit needs the test build, Python 3.11+ and a clone of [a2a-tck](https://github.com/a2aproject/a2a-tck) at the commit CI pins: see [testing](docs/testing.md#a2a-conformance-kit). Never use a `--features tck` build for real approvals.

`tests/secure-install.sh` runs in CI only (the `secure install` job on Linux and macOS). Do not run it on your own machine: it needs password-less sudo and installs a system service.

Every CI job, the container check and the sync: [docs/testing.md](docs/testing.md).

## Next

- [CONTRIBUTING.md](CONTRIBUTING.md): ground rules and checks.
- [docs/testing.md](docs/testing.md): every test and CI job.
- [docs/protocol.md](docs/protocol.md): the methods and errors as implemented.
- [The base's AGENTS.md](https://github.com/Phygital-Key/phygital-key/blob/main/AGENTS.md): the engine, the spec and the vectors.

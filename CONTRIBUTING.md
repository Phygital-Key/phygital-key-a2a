# Contributing

Thank you for helping. Phygital Key A2A is free and open source under [Apache-2.0](LICENSE), and contributions are accepted under the same license. Keep the [`NOTICE`](NOTICE) file and its line "Created by Phygital-Key".

## Where a change belongs

| Change | Repository |
|---|---|
| The A2A endpoint, the agent card, the message shapes, the task states, the extension profile, the `tck` behaviours, these docs | Here |
| The approval page, keys, standing approvals, receipts, the audit log, the commands, the secure install, updates, the verifier, the spec | The base, [Phygital-Key/phygital-key](https://github.com/Phygital-Key/phygital-key) ([its contributing guide](https://github.com/Phygital-Key/phygital-key/blob/main/CONTRIBUTING.md)) |
| `install.sh`, `install-service.sh`, `tests/secure-install.sh`, `scripts/*.sh` | The base's templates: these files are generated, never edited here |

Read [AGENTS.md](AGENTS.md) first: the layout, the rules that must never break, and how the generated files and the daily sync work.

## Ground rules

- Open an issue before a large change or a new dependency.
- Nothing may approve, no LLM may run in the gate or the approval page, and the `tck` feature never reaches a release.
- A breaking change to the message shapes, task states or artifact gets a new extension URI ([docs/extension.md](docs/extension.md)).
- Docs in plain words, no em dashes, every command one you ran. Shared topics are documented only in the base.
- Never write any real person's name, and add no AI attribution lines.

## Checks to run

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features tck -- -D warnings
cargo test --locked
cargo test --locked --features tck
cargo build --release --locked
target/release/phygital-key-a2a product-files --out . && git diff --exit-code
cd tests/e2e && npm ci && npx playwright install chromium && PKA_BIN=../../target/release/phygital-key-a2a node run.mjs
```

The last line runs the 165 end-to-end checks. The conformance kit, what each test covers and every CI job: [testing](docs/testing.md).

Security problems go to [SECURITY.md](SECURITY.md), not to a public issue.

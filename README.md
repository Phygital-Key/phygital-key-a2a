<p align="center"><img src="docs/images/hero.png" alt="Phygital Key A2A. Agents can ask. Only you can approve. Beside the headline, an agent sends a SendMessage data part with a nested deploy request (request with type and params, aud and level high); the task waits in TASK_STATE_AUTH_REQUIRED, you sign with a fingerprint on your laptop, the task completes with the phygital-key-approval artifact for one exact request, one service, 60 seconds, used once, and the agent resending a changed version is refused with REQUEST_MISMATCH." width="100%"></p>

<h1 align="center">Phygital Key A2A</h1>

<p align="center"><b>Other AI agents ask before they act. You say yes with your fingerprint, face or security key. The service that does the work can check that yes on its own, offline.</b></p>

<p align="center">
<a href="#open-source-and-credit"><img src="https://img.shields.io/badge/-open%20source-26D0F2" alt="Open source"></a>
<a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue" alt="License: Apache-2.0"></a>
<a href="https://github.com/Phygital-Key/phygital-key-a2a/actions/workflows/ci.yml"><img src="https://github.com/Phygital-Key/phygital-key-a2a/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
<a href="https://github.com/Phygital-Key/phygital-key-a2a/releases/latest"><img src="https://img.shields.io/github/v/release/Phygital-Key/phygital-key-a2a" alt="Latest release"></a>
<a href="Cargo.toml"><img src="https://img.shields.io/badge/rust-1.89%2B-orange" alt="Rust 1.89+"></a>
<a href="https://a2a-protocol.org/latest/specification/"><img src="https://img.shields.io/badge/A2A-1.0%20JSON--RPC-black" alt="A2A 1.0, JSON-RPC"></a>
<a href="https://github.com/Phygital-Key/phygital-key-a2a/stargazers"><img src="https://img.shields.io/github/stars/Phygital-Key/phygital-key-a2a?style=flat" alt="GitHub stars"></a>
<a href="https://github.com/Phygital-Key/phygital-key-a2a/commits/main"><img src="https://img.shields.io/github/last-commit/Phygital-Key/phygital-key-a2a" alt="Last commit"></a>
<a href="https://scorecard.dev/viewer/?uri=github.com/Phygital-Key/phygital-key-a2a"><img src="https://api.scorecard.dev/projects/github.com/Phygital-Key/phygital-key-a2a/badge" alt="OpenSSF Scorecard"></a>
</p>

<p align="center">
<a href="https://github.com/Phygital-Key/phygital-key/blob/main/docs/webauthn.md"><img src="https://img.shields.io/badge/WebAuthn-ES256%20%C2%B7%20P--256-FF6B5A" alt="WebAuthn, ES256 on P-256"></a>
<a href="docs/testing.md"><img src="https://img.shields.io/badge/end--to--end-172%20checks-26D0F2" alt="172 end-to-end checks"></a>
<a href="docs/testing.md#a2a-conformance-kit"><img src="https://img.shields.io/badge/a2a--tck%20MUST-81%20passed%20(test%20build)-26D0F2" alt="a2a-tck MUST level on JSON-RPC: 81 passed with the test build"></a>
<a href="https://github.com/Phygital-Key/phygital-key/blob/main/docs/secure-install.md"><img src="https://img.shields.io/badge/secure%20install-macOS%20%C2%B7%20Linux-555" alt="Secure install on macOS and Linux"></a>
<a href="https://github.com/Phygital-Key/phygital-key/blob/main/docs/threat-model.md"><img src="https://img.shields.io/badge/status-pre--audit-FF6B5A" alt="Status: pre-audit"></a>
<a href="https://github.com/Phygital-Key/phygital-key/blob/main/docs/updates.md"><img src="https://img.shields.io/badge/releases-signed-26D0F2" alt="Signed releases"></a>
<a href="Cargo.toml"><img src="https://img.shields.io/badge/%23!%5Bforbid(unsafe__code)%5D-yes-555" alt="#![forbid(unsafe_code)]"></a>
<a href="https://github.com/Phygital-Key/phygital-key/blob/main/docs/privacy.md"><img src="https://img.shields.io/badge/telemetry-none-555" alt="No telemetry"></a>
</p>

<p align="center"><b>Free and open source under the Apache-2.0 license.</b> Keep the <code>NOTICE</code> file. See <a href="#open-source-and-credit">Open source and credit</a>.</p>

<p align="center"><b>Pick your product:</b> <b>Phygital Key A2A</b> (this one, for agents that speak A2A) · <a href="https://github.com/Phygital-Key/phygital-key-mcp">Phygital Key MCP</a> (for AI apps that speak MCP) · <a href="https://github.com/Phygital-Key/phygital-key">the shared base</a> (verifiers, spec, shared guides)</p>

<p align="center">
<a href="#install">Install</a> ·
<a href="#try-it-in-2-minutes">Quick start</a> ·
<a href="#connect-an-agent">Connect an agent</a> ·
<a href="#the-message-shapes">Messages</a> ·
<a href="#conformance">Conformance</a> ·
<a href="#check-approvals-in-your-service">Check approvals</a> ·
<a href="#safety-verification-and-authority">Safety</a> ·
<a href="#all-the-guides">Guides</a>
</p>

## What it is

Phygital Key A2A is a [Phygital Key](https://github.com/Phygital-Key/phygital-key) product for AI agents that use [A2A](https://a2a-protocol.org/latest/specification/) (Agent2Agent, version 1.0, a common way for agents to talk to each other). It is one program, `phygital-key-a2a`. It runs an A2A agent and the [approval page](https://github.com/Phygital-Key/phygital-key/blob/main/docs/glossary.md#approval-page), the page where you say yes or no.

It is an agent whose only job is to get your approval. Before another agent deploys code, pays, deletes or sends something, it sends this agent a message with the exact thing it wants to do. You read it on the approval page and approve with a passkey: your fingerprint, face, PIN or a hardware security key. Your key signs that exact request (WebAuthn, ES256 on P-256). The asking agent gets the signed approval back as a task result (an artifact) and passes it to the service that does the work. The service checks it itself, offline, before it acts.

The agent can never approve. Only your key's signature on the approval page counts as a yes. A message, a task state, extra data or any text never does. If the request changes by even one character, the service refuses it with `REQUEST_MISMATCH`.

The approval page, the keys, standing approvals, receipts, the audit log, the secure install and signed updates come from the shared base, [Phygital-Key/phygital-key](https://github.com/Phygital-Key/phygital-key). This repository adds the A2A part. It uses the same approval format (`pka.1`) as [Phygital Key MCP](https://github.com/Phygital-Key/phygital-key-mcp), so a service that already checks Phygital Key approvals needs no change.

## Why it exists

Agents call other agents to deploy, order, pay and send. When an agent asks "may I?" in its own conversation, the yes it gets back is a message. Nobody signed it, so the service that finally acts can't tell whether a person said yes, or what they said yes to. Any agent along the way could have written it.

Phygital Key A2A gives the service proof it can check:

| Promise | How |
|---|---|
| **Can't be faked** | Only your key can sign (ES256 on P-256). The secret part of the key never leaves your phone, your laptop's security chip or your security key. No agent holds a key, none of the five skills can approve, and the gate (this program) has no key that can sign approvals. A task only reaches `TASK_STATE_COMPLETED` with your signed approval. |
| **Can't be changed** | Your signature covers the exact request, written in one fixed form (RFC 8785). The service rebuilds the request it is about to run and compares the two, character by character. One changed character gives `REQUEST_MISMATCH`. |
| **Can't be reused** | Each approval names one service (`aud`), lasts 60 seconds by default and works once. A second use gives `REPLAYED`. |
| **Can't be phished** | Passkeys only sign for the real address of your approval page. A fake copy on another address gets nothing it can use. |
| **Can't be copied and used by someone else** | An agent can ask for an approval tied to its own key (`agent_key`). Then only that agent can use the approval: a copy taken from a log or a chat is refused. |
| **Out of the AI's reach** | With the [secure install](#out-of-the-ais-reach), the gate runs as its own locked-down user on your computer (`_phygitalkeya2a` on macOS, `phygitalkey-a2a` on Linux). An agent on the same computer, even acting as you, can't change its program, read its data or add keys. Only your administrator password can. |
| **Checked offline** | The service only needs your public keys and a few lines of code. Nothing calls home. No account, no cloud. |

## How it works

<p align="center"><img src="https://raw.githubusercontent.com/Phygital-Key/phygital-key/main/docs/images/promo/how.png" alt="How it works, four roles and one signed yes: 01 the agent asks for one exact request for one service, over MCP or A2A; 02 the gate shows every field on the approval page and has no key, so it can never approve; 03 you approve with your fingerprint, face, PIN or key, which makes an ES256 signature; 04 the service checks the signed yes itself, offline, with verify() returning ok, then acts." width="100%"></p>

```mermaid
sequenceDiagram
    participant A as Asking agent (A2A client)
    participant G as Gate (phygital-key-a2a)
    participant P as You + passkey
    participant S as Service (verifier)
    A->>G: SendMessage with a data part (request, aud, level)
    G-->>A: task in TASK_STATE_AUTH_REQUIRED, approveUrl
    A->>P: shows approveUrl
    P->>G: Approve with fingerprint, face, PIN or touch
    G-->>A: task COMPLETED, artifact phygital-key-approval (stream, subscribe or GetTask)
    A->>S: action + Phygital-Approval header
    S->>S: check audience, time, exact request, level, key, signature, first use
    S-->>A: done (and a signed receipt), or refused with a code
```

Four roles, and what each one can and can't do:

| Who | Decides | Can never |
|---|---|---|
| **You** (the person who approves) | Yes or no, on the exact request shown | Be skipped: no key, no approval |
| **The asking agent** | What to ask for, in one of five message shapes | Approve, sign, or change a request after you signed it |
| **The gate** (`phygital-key-a2a`) | Shows the request and collects your signature | Sign an approval by itself, or finish a task without your signature |
| **The service** (the one that checks, also called the verifier) | Its own name (audience), the request it runs, how strong a yes each action needs, which keys it trusts | Accept a request you did not sign |

More: [how it works](https://github.com/Phygital-Key/phygital-key/blob/main/docs/how-it-works.md), [protocol](docs/protocol.md), [the open spec](https://github.com/Phygital-Key/phygital-key/blob/main/docs/spec.md).

## What you see

<p align="center">
<img src="https://raw.githubusercontent.com/Phygital-Key/phygital-key/main/docs/images/approve-dark-phone.png" alt="The approval page on a phone in the dark theme: a deploy request carried out by the demo service, proof needed fingerprint, face or PIN, decide within 10 minutes, every detail listed (env production, service api, version 2.4.1), the exact text being signed, and Deny and Approve with my key buttons" width="230">
<img src="https://raw.githubusercontent.com/Phygital-Key/phygital-key/main/docs/images/check-this-light.png" alt="A payment request in the light theme to ACME Ltd for amount_cents 12000 in EUR, where the note and reference fields are flagged with a Check this warning about look-alike, invisible or extra-space characters" width="230">
<img src="https://raw.githubusercontent.com/Phygital-Key/phygital-key/main/docs/images/standing-dark.png" alt="A standing approval request in the dark theme: payments to ACME Ltd in EUR, amount_cents from 1 to 2000, anything else refused, at most 5 times until an end date, with every detail and the Deny and Approve with my key buttons" width="230">
</p>

These screenshots come from the shared approval page, so Phygital Key A2A shows the same one (with its own demo service on port 7479). The gate builds the page from the request itself, never from text an agent wrote. The page:

- shows every field;
- warns you about look-alike letters, invisible characters and extra spaces;
- shows a short fingerprint (SHA-256) of the exact text you sign;
- checks that what it shows is what will be signed, before it asks your key.

Text inside a request or a follow-up message, like "already approved", is shown as plain text and changes nothing. How to read the page: [approving](https://github.com/Phygital-Key/phygital-key/blob/main/docs/approving.md). How the page protects you: [approval page](https://github.com/Phygital-Key/phygital-key/blob/main/docs/approval-page.md).

## Install

```bash
npm i @phygital-key/a2a
```

| Where | Command |
|---|---|
| macOS, Linux (script) | `curl -fsSL https://github.com/Phygital-Key/phygital-key-a2a/releases/latest/download/install.sh \| sh` |
| macOS, Linux (Homebrew) | `brew install phygital-key/tap/phygital-key-a2a` |
| Windows (Scoop) | `scoop bucket add phygital-key-a2a https://github.com/Phygital-Key/phygital-key-a2a` then `scoop install phygital-key-a2a` |
| Anywhere with Rust | `cargo install --locked --git https://github.com/Phygital-Key/phygital-key-a2a phygital-key-a2a` |
| Anywhere with Node.js (npm) | `npx -y @phygital-key/a2a serve`, or `npm install -g @phygital-key/a2a` |
| Server (Docker) | `docker run -d --name phygital-key-a2a -p 127.0.0.1:7478:7478 -v phygital-key-a2a:/data ghcr.io/phygital-key/phygital-key-a2a:latest` |
| Secure install (macOS, Linux) | See [out of the AI's reach](#out-of-the-ais-reach) |
| By hand, also Windows | Download from [Releases](https://github.com/Phygital-Key/phygital-key-a2a/releases). Each file has a `.sha256` next to it, and the signed lists `release.txt` and `release2.txt` give every file's SHA-256 |

The first release was v0.1.0. Each release has files for Linux (x86_64, arm64), macOS (Apple silicon, Intel) and Windows (x86_64), the public container image `ghcr.io/phygital-key/phygital-key-a2a` (`linux/amd64`, `linux/arm64`), the Homebrew formula and a Scoop manifest.

The npm package `@phygital-key/a2a` runs the same released program: it holds the build for each computer (macOS, Linux, Windows) and starts the one for yours. The release workflow publishes it with npm provenance. Update it with npm (`npm install -g @phygital-key/a2a@latest`).

The script downloads the latest release for your computer and checks two things before it installs anything. First, that the release's file list (`release2.txt`) is signed by a release key built into the script, and is still in date. Second, that the download matches its SHA-256. Then it writes one file, `~/.local/bin/phygital-key-a2a`. It needs `curl`, `tar` and `openssl`. To update later: `phygital-key-a2a update`.

The container runs `phygital-key-a2a --bind 0.0.0.0 serve` as user `65532`, with its data in `/data` (`PHYGITAL_KEY_A2A_HOME=/data`). Add your key with `docker exec -it phygital-key-a2a phygital-key-a2a enroll`, and get the access token with `docker exec -it phygital-key-a2a phygital-key-a2a token`. Only open the port on `127.0.0.1`, or put an HTTPS proxy in front and pass `--origin https://your.host` ([your own server](https://github.com/Phygital-Key/phygital-key/blob/main/docs/own-server.md)).

Every way to install and update: [updates](https://github.com/Phygital-Key/phygital-key/blob/main/docs/updates.md). The names this product uses: [reference](https://github.com/Phygital-Key/phygital-key/blob/main/docs/reference.md#names-per-product).

## Try it in 2 minutes

```bash
phygital-key-a2a demo
```

The demo starts the gate on `http://localhost:7478`, and a pretend service on `http://localhost:7479` that only acts when it gets a valid approval (`/deploy`, `/payment`, `/send_message`; nothing real happens). It prints:

- a one-time link (good for 10 minutes) to add your passkey;
- the agent card (the agent's public description);
- the address with your token;
- a `curl` line to try.

Then ask for an approval, with the token from `phygital-key-a2a token` in `$TOKEN`:

```bash
curl -s http://localhost:7478/a2a \
  -H "Authorization: Bearer $TOKEN" -H "A2A-Version: 1.0" -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"SendMessage","params":{"message":{
        "messageId":"msg-1","role":"ROLE_USER",
        "parts":[{"mediaType":"application/json","data":{"request":{
          "request":{"type":"deploy","params":{"service":"api","env":"production","version":"2.4.1"}},
          "aud":"http://localhost:7479","level":"high"}}}]}}}'
```

You get back a task in `TASK_STATE_AUTH_REQUIRED` with an `approveUrl`. Open it and approve. Now `GetTask` returns the task as `COMPLETED`, with the signed approval. POST the params to `http://localhost:7479/deploy` with the `Phygital-Approval` header, and the demo service "deploys". Send it again, or with version 2.4.2, and it refuses. The full walk-through, with real outputs and the same steps in the official JavaScript client: [getting started](docs/getting-started.md).

## Connect an agent

<p align="center"><img src="docs/images/card.png" alt="Discovery, found like any agent: an agent card at the standard address, a JSON-RPC endpoint and a bearer token, streaming included, tested with the official A2A client and conformance kit. A card titled agent card, no token needed, shows GET /.well-known/agent-card.json with name Phygital Key A2A, one supported interface with protocolBinding JSONRPC and protocolVersion 1.0, and capabilities streaming true. A second card lists the 5 skills: request_approval, request_standing_approval, use_standing_approval, record_receipt and revoke_standing_approval." width="100%"></p>

| What | Value |
|---|---|
| Agent card | `GET http://localhost:7478/.well-known/agent-card.json` (public, no token) |
| Address | `POST http://localhost:7478/a2a`, JSON-RPC 2.0, `Content-Type: application/json` or `application/a2a+json` |
| Headers on every call | `Authorization: Bearer <token>` and `A2A-Version: 1.0` |
| The token | `phygital-key-a2a token` prints it (it is saved in `<home>/access-token`, readable only by you, mode 0600) |
| Methods | `SendMessage`, `SendStreamingMessage`, `GetTask`, `ListTasks`, `CancelTask`, `SubscribeToTask` |
| Not supported | push notifications (`-32003`), the extended agent card (`-32004`), the gRPC and HTTP+JSON bindings, stdio |

The card says: streaming yes, push notifications no, one bearer-token security scheme, the Phygital Key extension (optional), and five skills: `request_approval`, `request_standing_approval`, `use_standing_approval`, `record_receipt` and `revoke_standing_approval`. None of them can approve.

The address refuses calls without the token (`401`), calls from a web browser (`Origin` header, `403`), and calls with a wrong `Host` header (a trick called DNS rebinding, `421`). Streams send the task first, then stay open while you decide, with a heartbeat every 15 seconds. At most 32 can be open at once.

Any A2A 1.0 client that reads the card and sends the token header works. The [official JavaScript client](https://www.npmjs.com/package/@a2a-js/sdk) (`@a2a-js/sdk` 1.3.0) runs in every end-to-end test. Examples in JavaScript, Python and `curl`, waiting by polling or by streaming: [connect your agent](docs/connect-your-agent.md).

## How it maps onto A2A

<p align="center"><img src="docs/images/states.png" alt="Phygital Key A2A on A2A 1.0, a task that waits for you: another agent sends a task, it waits while you decide on the approval page, then ends one of four ways. SendMessage carries a data part with the exact request, the service and the level; the task waits in TASK_STATE_AUTH_REQUIRED with the approval link in its status while you approve or deny on the page; it ends in TASK_STATE_COMPLETED (signed), TASK_STATE_REJECTED (refused), TASK_STATE_CANCELED (cancelled) or TASK_STATE_FAILED (expired)." width="100%"></p>

Each request for approval becomes an A2A task. The task id is the gate's request id.

```mermaid
stateDiagram-v2
    [*] --> AUTH_REQUIRED: SendMessage with a request
    AUTH_REQUIRED --> COMPLETED: the person approves
    AUTH_REQUIRED --> REJECTED: the person denies
    AUTH_REQUIRED --> CANCELED: the agent calls CancelTask
    AUTH_REQUIRED --> FAILED: nobody decided in 10 minutes
    COMPLETED --> [*]
    REJECTED --> [*]
    CANCELED --> [*]
    FAILED --> [*]
```

| Task state | What it means | What the asking agent does |
|---|---|---|
| `TASK_STATE_AUTH_REQUIRED` | Waiting for the person. The status message holds the `approveUrl`. | Show the person the link, then stream, subscribe or check back. |
| `TASK_STATE_COMPLETED` | The person approved. The task has one result, `phygital-key-approval`, with the signed `approval`, `exp` and `approvedBy`. | Send the approval with the exact action, once, to the one service it names. |
| `TASK_STATE_REJECTED` | The person said no. | Don't act. |
| `TASK_STATE_CANCELED` | The agent withdrew it. | Don't act. |
| `TASK_STATE_FAILED` | Nobody decided within 10 minutes, or the approval ran out before the agent collected it. | Don't act. |

A task leaves `TASK_STATE_AUTH_REQUIRED` only once, and a finished task never changes again. Every state, message and error: [protocol](docs/protocol.md) and the [extension profile](docs/extension.md).

## The message shapes

Every message to this agent has one data part (`mediaType: application/json`), and its object has exactly one key:

| Key | What it asks | The answer |
|---|---|---|
| `request` | A one-time approval: `{"request": {"type", "params"}, "aud", "level"}`, and optionally `agent_key` (the asking agent's own key, so only it can use the approval) | A task in `TASK_STATE_AUTH_REQUIRED` |
| `standing` | A [standing approval](https://github.com/Phygital-Key/phygital-key/blob/main/docs/standing-approvals.md): `{"for_type", "fixed", "vary", "max_uses", "days", "aud"}` | A task in `TASK_STATE_AUTH_REQUIRED` |
| `use` | One use of an approved standing approval: `{"standingId", "request"}` | A message with `approval`, `useToken`, `usesLeft` |
| `receipt` | Passes back the service's signed receipt: `{"id", "receipt"}` | A message with `serviceReported` |
| `revoke` | Stops a standing approval: `{"standingId"}` | A message with `revoked` and the `signUrl` for the person |

The smallest request:

```json
{"data": {"request": {
  "request": {"type": "deploy", "params": {"service": "api", "env": "production", "version": "2.4.1"}},
  "aud": "https://deploy.example.com",
  "level": "high"
}}, "mediaType": "application/json"}
```

`level` is `medium` (a touch on your key) or `high` (fingerprint, face or PIN). `aud` is the one service that will act on the yes. Text parts next to the data part are ignored. A message without a valid data part gets a `ROLE_AGENT` message back that lists these shapes, never a task. The full profile, with every field, the approval result and how to pass an approval to another agent: [extension](docs/extension.md).

## Conformance

The official [A2A conformance kit](https://github.com/a2aproject/a2a-tck) (`a2a-tck`, pinned commit `263b9cf`) is a test suite that checks an agent follows the A2A rules. It runs in CI at the MUST level (the required rules) on the JSON-RPC binding:

| Build | Result | Why |
|---|---|---|
| Test build, `cargo build --features tck` | 81 of 81 required checks passed | The kit runs its scenarios through messages whose `messageId` starts with `tck-` (finish a task, add results, ask for input, stream updates). Only this build answers them, and those tasks never reach the gate. |
| Release build (no features) | Can't run | The kit connects without the access token, and a release build refuses `--no-token` (exit code 2). Its scenarios also need an agent that finishes tasks on its own, while this agent only finishes a task after a person's signed approval. |

The release workflow never turns on the `tck` feature. Only a `tck` build accepts `--no-token`, and the connection details it prints start with a warning never to use it for real approvals. How to run the kit yourself: [testing](docs/testing.md#a2a-conformance-kit).

## Check approvals in your service

<p align="center"><img src="https://raw.githubusercontent.com/Phygital-Key/phygital-key/main/docs/images/promo/check.png" alt="For services, check it anywhere: Rust, JavaScript or any language through the command line, with the same 141 shared test vectors. Three cards: Rust with phygital-key-verify, building the request you are about to run and calling verifier.verify with Level::High, which refuses on any doubt; JavaScript with no dependencies, where r.ok is true to act and r.error is REPLAYED; and the command line, phygital-key-mcp verify with the approval, the request and the audience, printing ok true with exit 0 or error REPLAYED with exit 1, the same in phygital-key-a2a." width="100%"></p>

This part is for whoever runs the service. The service does the checking. It compares the approval with the request **it** is about to run, built from its own data. The check is the same no matter which product made the approval, so an approval collected over A2A is checked exactly like one collected over MCP.

**Rust**, with the [`phygital-key-verify`](https://github.com/Phygital-Key/phygital-key/blob/main/docs/verifier-rust.md) library from the base:

```rust
use phygital_key_verify::{Config, Keyring, Level, Verifier};

let verifier = Verifier::new(
    "https://deploy.example.com",                                     // your audience
    Keyring::from_json(&std::fs::read_to_string("keyring.json")?)?,   // trusted keys: `phygital-key-a2a keys`
    Config { origins: vec!["https://approve.example.com".into()], rp_ids: vec!["approve.example.com".into()] },
);
let request = serde_json::json!({"type": "deploy", "params": {"service": "api", "env": "production", "version": "2.4.1"}});
verifier.verify(&approval, &request, Level::High)?; // refuses on any doubt
// ...now deploy.
```

**Any language:** run the command line and look at the exit code.

```bash
phygital-key-a2a verify --approval "$APPROVAL" \
  --request '{"type":"deploy","params":{"service":"api","env":"production","version":"2.4.1"}}' \
  --aud https://deploy.example.com --level high
```

Exit 0 with `{"ok": true, ...}` means go ahead. Exit 1 with `{"ok": false, "error": "REQUEST_MISMATCH"}` means no. Used approvals are saved in a local file, so each one works only once. There is also a JavaScript checker with no dependencies, and it passes the same 141 shared test cases ([JavaScript](https://github.com/Phygital-Key/phygital-key/blob/main/docs/verifier-js.md)).

Full guide: [integrate](https://github.com/Phygital-Key/phygital-key/blob/main/docs/integrate.md). By language: [Rust](https://github.com/Phygital-Key/phygital-key/blob/main/docs/verifier-rust.md), [JavaScript](https://github.com/Phygital-Key/phygital-key/blob/main/docs/verifier-js.md), [command line](https://github.com/Phygital-Key/phygital-key/blob/main/docs/cli.md).

## Safety, verification and authority

### What the service checks, in order

When in doubt, the checker says no. The first check that fails refuses the approval, with a code that says why.

| Step | What is checked | Refused with |
|:-:|---|---|
| 1 | The approval is there, is clean JSON, has only known fields, and isn't too big or too deep | `APPROVAL_REQUIRED`, `MALFORMED` |
| 2 | It was approved for this service (`aud`) | `WRONG_AUDIENCE` |
| 3 | It is in time: not before `iat` (30 seconds of clock difference allowed), not after `exp`, and it lives at most 300 seconds | `EXPIRED` |
| 4 | It is the exact request, byte for byte, in the fixed form | `REQUEST_MISMATCH` |
| 5 | It is a strong enough yes for this action (`medium` is a touch, `high` is fingerprint, face or PIN) | `ASSURANCE_TOO_LOW` |
| 6 | It was signed by a key in your keyring (your list of trusted keys), for the right site | `UNTRUSTED_KEY` |
| 7 | The passkey's signed answer (WebAuthn assertion): type, challenge, origin, site hash and ES256 signature | `BAD_SIGNATURE` |
| 8 | The "person was there" and "person was verified" flags match the level | `ASSURANCE_TOO_LOW` |
| 9 | It is the first use (used approvals are remembered) | `REPLAYED` |

Every code, why it happens and what to do: [error codes](https://github.com/Phygital-Key/phygital-key/blob/main/docs/error-codes.md).

### The cryptography

Your key signs a challenge that locks in every detail of the approval:

$$\text{challenge} = \mathrm{SHA256}\big(\texttt{"phygital-key/approval/v1"} \,\|\, \texttt{0x00} \,\|\, \mathrm{JCS}(\text{claims})\big)$$

- The signature is ES256 (ECDSA on P-256 with SHA-256), made by your passkey (a WebAuthn assertion).
- Hashes are SHA-256.
- The fixed form is RFC 8785 (JCS): strict JSON, only safe whole numbers, no decimals, and text in Unicode NFC. Other text is refused, never rewritten.
- Receipts and use tokens are ES256 JWS. The audit log is a SHA-256 hash chain, so a changed entry shows. Release lists are signed with ECDSA P-256, and also with ML-DSA-65, a signature built to resist future quantum computers.

Deep dives: [canonical form](https://github.com/Phygital-Key/phygital-key/blob/main/docs/canonical-form.md), [WebAuthn checks](https://github.com/Phygital-Key/phygital-key/blob/main/docs/webauthn.md), [key history](https://github.com/Phygital-Key/phygital-key/blob/main/docs/key-history.md).

> [!NOTE]
> Approvals are **signed, not encrypted**. The gate, the service, every agent that passes the task result along and anyone holding an approval can read the request. Keep passwords and card numbers out of requests. Use HTTPS for any gate that isn't on `localhost`. Why: [decision 33](https://github.com/Phygital-Key/phygital-key/blob/main/docs/decisions.md).

Every key you add or remove is itself an approval, signed by a key you already trust. So a service can check your whole key history starting from your first key's fingerprint (`phygital-key-a2a keys --check <fingerprint>`).

### Out of the AI's reach

<p align="center"><img src="https://raw.githubusercontent.com/Phygital-Key/phygital-key/main/docs/images/promo/reach.png" alt="Secure install on macOS and Linux, out of the AI's reach: the gate runs as its own locked-down system user, and an agent acting as you cannot touch it. You, and any AI acting as you, are refused when trying to change its program, read its data, write its keys or serve a fake page on its port. Only the administrator password crosses into the gate's own system user." width="100%"></p>

An agent on your computer runs as you, so it could change files you own, including a gate you installed yourself. The secure install puts the gate where the AI can't reach it (macOS and Linux). Open a new terminal and run:

```bash
curl -q -fsSL https://raw.githubusercontent.com/Phygital-Key/phygital-key-a2a/v0.3.1/install-service.sh | sudo bash
```

It downloads the official release itself and checks it against the signed release list. It installs it into folders only the administrator can change, and runs it as its own user. If any other program is already using port 7478, it refuses to run. Then:

```bash
phygital-key-a2a-admin enroll      # add your key
phygital-key-a2a-admin token       # the connection details for your agents
phygital-key-a2a-admin status      # check that the real gate is the one answering
```

| | macOS | Linux |
|---|---|---|
| Gate user | `_phygitalkeya2a` | `phygitalkey-a2a` |
| Program | `/Library/PhygitalKey/a2a/bin/phygital-key-a2a` | `/usr/lib/phygital-key-a2a/bin/phygital-key-a2a` |
| Data | `/Library/Application Support/PhygitalKey/a2a` | `/var/lib/phygital-key-a2a` |
| Background services | launchd `io.phygitalkey.a2a.gate`, `io.phygitalkey.a2a.update` | systemd `phygital-key-a2a.service`, `phygital-key-a2a-update.timer` |

On Linux, systemd holds the gate's port while the gate restarts for an update, so no other program can grab it in between.

> [!IMPORTANT]
> The secure install only protects the gate if agents can't get your administrator password. Run it from a terminal you trust, don't let your user run `sudo` without a password, and never type that password where an agent can see it. Each admin command asks for it.

Details, every folder and every refusal: [secure install](https://github.com/Phygital-Key/phygital-key/blob/main/docs/secure-install.md).

### Signed automatic updates

The secure install checks for updates once a day. It installs a new release only if all of this is true:

- its file list (`release2.txt`) is signed by the release keys built into this program and its installers (ECDSA P-256, plus ML-DSA-65 once that key exists);
- the list is still in date, and not older than one this computer already saw;
- it is newer than the version you have.

The signing key is a secret kept in this repository's GitHub environment `release`, and only a release run started from a `v*` tag can use it.

The update waits while a standing approval is active (7 days at most), because a restart would end it. If the new gate doesn't start and pass its checks, the previous version comes back. The update runs as the administrator with a clean setup, so your own settings like `~/.curlrc` and `PATH` are never used.

`phygital-key-a2a-admin status` shows the last result, `phygital-key-a2a-admin update` updates now, and `--no-auto-update` turns automatic updates off. How updates are protected: [the update channel](https://github.com/Phygital-Key/phygital-key/blob/main/docs/threat-model.md#the-update-channel). Every way to update: [updates](https://github.com/Phygital-Key/phygital-key/blob/main/docs/updates.md).

### How it is tested

| What | Where | How |
|---|---|---|
| Unit tests | This repository | 48 tests in `src/` (49 with `--features tck`). They check that a gate status alone never finishes a task, a finished task never changes, text never asks or approves, and the limits on history and streams |
| End to end, and attacks | This repository | 172 checks with the real release program, a real Chromium browser with a pretend passkey, the official A2A JavaScript client and the demo service. They include attacks: changed requests, prompt injection in params and follow-up messages, forged signatures, reused approvals, wrong versions and content types, DNS rebinding |
| Conformance kit | This repository | `a2a-tck` at the MUST level on JSON-RPC: 81 of 81 required checks passed with the test-only `tck` build |
| Secure install | This repository | 36 checks on Linux, 32 on macOS. It is installed for real in CI, then attacked as the user. Also checked: the update job (started by systemd and launchd), refusing a badly signed release, a planted `~/.curlrc` and `PATH`, switching back after a failed update, and the sandbox score |
| Container | This repository | Builds the image, starts the gate, checks the agent card, user `65532` and adding a key |
| Generated files | This repository | The files made from the base's templates must match the ones saved here |
| Shared test cases | Base | 141 cases that both the Rust and the JavaScript checkers must pass, plus Google's Wycheproof cases, a cross-check with SimpleWebAuthn, property tests, fuzzing and mutation testing |
| Security reviews | Base | 9 separate AI security reviews. Every problem found was fixed or written down as a known limit. Review 8 covers the A2A part |

CI runs on every change on Linux, macOS and Windows, and again after each daily base update. Job by job: [testing](docs/testing.md). The engine's tests: [testing in the base](https://github.com/Phygital-Key/phygital-key/blob/main/docs/testing.md). Found a problem? [Report it privately](SECURITY.md).

## Standing approvals

<p align="center"><img src="https://raw.githubusercontent.com/Phygital-Key/phygital-key/main/docs/images/promo/standing.png" alt="Standing approvals, say yes once, with limits: for small repeated actions, every use is checked against what you signed, and you can revoke it at any time. A signed, active standing approval for payment to ACME Ltd in EUR, amount_cents 1 to 2000, at most 5 uses in 7 days, used 2 of 5; a use with amount_cents 5000 is refused with NOT_ALLOWED." width="100%"></p>

For small actions that repeat, you can say yes once, with limits:

> Allow **payment** to **ACME Ltd** in **EUR**, **up to 20.00** each, **at most 5 times**, **for 7 days**.

The asking agent sends a `standing` message (up to 30 days). Once you approve, the task finishes with a `standingId`. For each action, the agent sends a `use` message and gets back the approval, a use token that lasts one minute, and how many uses are left. It sends the approval and the token to the service (the `Phygital-Approval` and `Phygital-Use` headers). The service checks both and counts the uses itself. A different payee, a bigger amount or an extra field is refused with `NOT_ALLOWED`. Standing approvals always need your fingerprint, face or PIN.

You can cancel it on the approval page, or have the agent send a `revoke` message. New uses stop at once. One more touch on the `signUrl` signs a cancellation notice that services read from `/.well-known/phygital-key-revoked.json`. Guides: [for people](https://github.com/Phygital-Key/phygital-key/blob/main/docs/standing-approvals.md), [for services](https://github.com/Phygital-Key/phygital-key/blob/main/docs/integrate.md#standing-approvals), [cancellations](https://github.com/Phygital-Key/phygital-key/blob/main/docs/revocations.md).

## Receipts

After it acts, a service can send back a signed receipt ("done" or "failed"). The agent passes it on in a `receipt` message. Your approval page shows it only if it was signed by the key you saved for that service:

```bash
phygital-key-a2a trust-service --aud https://deploy.example.com --key <its receipt_public_key>
```

Format and checks: [receipts](https://github.com/Phygital-Key/phygital-key/blob/main/docs/receipts.md).

## All the guides

The shared guides are in the base repository: **[all the guides](https://github.com/Phygital-Key/phygital-key#all-the-guides)** (approving, keys, standing approvals, secure install, checking approvals in your service, security). Guides for one product (connecting agents, its messages) are in that product's repository.

This product's guides: [Getting started](docs/getting-started.md) · [Connect your agent](docs/connect-your-agent.md) · [Protocol](docs/protocol.md) · [Extension](docs/extension.md) · [Testing](docs/testing.md)

## This repository

| Path | What |
|---|---|
| [`src/main.rs`](src/main.rs) | The product (name, program, port, release keys) and the A2A part |
| [`src/card.rs`](src/card.rs) | The public agent card at `/.well-known/agent-card.json` |
| [`src/server.rs`](src/server.rs), [`src/rpc.rs`](src/rpc.rs) | The JSON-RPC address at `/a2a`, the methods, the errors and the streams |
| [`src/intent.rs`](src/intent.rs), [`src/proto.rs`](src/proto.rs), [`src/store.rs`](src/store.rs) | The accepted message shapes, the A2A objects and task states, and where tasks are kept |
| [`src/tck.rs`](src/tck.rs) | Test behaviours for the conformance kit, only built with `--features tck` |
| [`tests/e2e/`](tests/e2e) | 172 end-to-end and attack checks: release program, Chromium with a pretend passkey, the official A2A client, the demo service |
| [`docs/`](docs) | [Getting started](docs/getting-started.md), [connect your agent](docs/connect-your-agent.md), [protocol](docs/protocol.md), [extension](docs/extension.md), [testing](docs/testing.md) |
| [`packaging/container/`](packaging/container) | The container image |
| [`release-keys.pem`](release-keys.pem) | The public release keys that every update and installer checks against |
| `install.sh`, `install-service.sh`, `tests/secure-install.sh`, `scripts/` | Made from the base's templates for this product with `phygital-key-a2a product-files`. Never edited by hand: CI fails if they don't match. See [AGENTS.md](AGENTS.md) |

Everything else comes from the base, [Phygital-Key/phygital-key](https://github.com/Phygital-Key/phygital-key): the approval page, keys, standing approvals, receipts, audit log, secure install, signed updates and the checker. Every day the `sync-base` workflow moves this product to the base's latest version. It keeps the change only if the tests and the 172 end-to-end checks pass, then saves it and runs the full CI. How the base and the products fit together: [architecture](https://github.com/Phygital-Key/phygital-key/blob/main/docs/architecture.md#the-base-and-the-products).

Changes: [CHANGELOG.md](CHANGELOG.md). Security problems: [SECURITY.md](SECURITY.md). Contributing: [CONTRIBUTING.md](CONTRIBUTING.md).

## Privacy

The gate keeps its data in one folder: `~/.phygital-key-a2a`, or after the secure install, a folder only the gate's own user can read. In it:

- the public keyring (your public keys);
- an audit log, chained so changes show (ids, hashes and results, never the request itself);
- saved service keys;
- signed cancellations;
- the access token.

Waiting requests and tasks live only in memory, with a limited history. The agent card holds no token. No analytics, no accounts, no tracking. The fonts are built into the program, so the approval page loads nothing from the internet. More: [privacy](https://github.com/Phygital-Key/phygital-key/blob/main/docs/privacy.md).

## Open source and credit

Phygital Key A2A is free and open source under the [Apache-2.0](LICENSE) license. You can use it at home or at work, change it, put it in a product and sell that product. You can read every line of code that handles your approvals, and check the release you run against its signed SHA-256 list.

Keep the [`NOTICE`](NOTICE) file, with the line "Created by Phygital-Key", in every copy or changed version (Apache-2.0, section 4(d)).

Contributions are welcome under the same license: see [CONTRIBUTING.md](CONTRIBUTING.md). Shared behaviour is changed in the base, never here.

## No warranty

Provided "as is", without warranty of any kind, as the [license](LICENSE) says. It hasn't been audited yet: try it with the demo first, read the [threat model](https://github.com/Phygital-Key/phygital-key/blob/main/docs/threat-model.md), and keep a second key added as a backup.

## License

[Apache-2.0](LICENSE). Created by Phygital-Key.

# Getting started

This page takes you from nothing to a first approval over A2A: start the demo, add your key, ask for an approval with `curl`, approve it on the approval page, and let the demo service check it. Then the same flow with the official A2A JavaScript client. It is for developers who build or run agents. Words like [approval](https://github.com/Phygital-Key/phygital-key/blob/main/docs/glossary.md#approval), [audience](https://github.com/Phygital-Key/phygital-key/blob/main/docs/glossary.md#audience) and [level](https://github.com/Phygital-Key/phygital-key/blob/main/docs/glossary.md#level) are in the glossary.

> [!NOTE]
> Nothing is released yet. The install commands in the [README](../README.md#install) work from the first release, v0.1.0. Every output on this page was captured from the v0.1.0 build; ids, tokens and times differ on your machine, and long signed strings are cut to `...`.

**On this page:** [before you start](#before-you-start) · [start the demo](#start-the-demo) · [add your key](#add-your-key) · [ask with curl](#ask-for-an-approval-with-curl) · [approve](#approve-on-the-approval-page) · [collect the approval](#collect-the-approval) · [let the service check it](#let-the-service-check-it) · [a deny](#what-a-deny-looks-like) · [with the JavaScript client](#the-same-flow-with-the-official-javascript-client)

## Before you start

You need:

- **A key.** Any passkey works: Touch ID or Face ID on a Mac or iPhone, Windows Hello, an Android phone, a password manager that stores passkeys, or a FIDO2 security key. Your first key must be able to check a fingerprint, face or PIN ([keys and devices](https://github.com/Phygital-Key/phygital-key/blob/main/docs/keys-and-devices.md)).
- **A recent browser.** Passkeys work only on `https` pages or on `localhost`.
- **The program**, `phygital-key-a2a` ([install](../README.md#install)). Check it:

```bash
phygital-key-a2a --version
```

```text
phygital-key-a2a 0.1.0
```

## Start the demo

The demo runs the gate and a fake service on your computer. The fake service deploys, pays and sends messages only in words, and refuses every action that does not carry a valid approval.

```bash
phygital-key-a2a demo
```

```text
phygital-key-a2a: no key enrolled yet. Open this link to add your passkey (valid 10 minutes, works once):
  http://localhost:7478/enroll#token=wWyNy4hW...
phygital-key-a2a demo
  approval page   http://localhost:7478
  agent card      http://localhost:7478/.well-known/agent-card.json  (public, no token)
  A2A endpoint    http://localhost:7478/a2a  (JSON-RPC, A2A 1.0; needs headers  Authorization: Bearer FhD6Svob...  and  A2A-Version: 1.0)
  Try it:         curl -s http://localhost:7478/a2a -H "Authorization: Bearer FhD6Svob..." -H "A2A-Version: 1.0" -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"ListTasks","params":{}}'
  demo service    http://localhost:7479  (fake deploy, payment, send_message)
Point any A2A client at the agent card above. Send a message with one data part {"request": {"request": {"type": "deploy", "params": {"service": "api", "env": "production", "version": "2.4.1"}}, "aud": "<demo service address>", "level": "high"}}, open the approveUrl from the task to decide, then POST the params to <demo service>/deploy with the header Phygital-Approval: <approval from the task's artifact>.
```

Keep it running. In another terminal, put the access token in a variable for the commands below (`phygital-key-a2a token` prints it again at any time):

```bash
TOKEN=$(phygital-key-a2a token | grep -o 'Bearer [A-Za-z0-9_-]*' | head -1 | cut -d' ' -f2)
```

Before you add a key, the agent refuses requests with a message, not a task:

```json
{"id":1,"jsonrpc":"2.0","result":{"message":{"contextId":"l1uVtqaBb4sjhClhanZ4Fg","messageId":"nsUSxtX7kBUXpBLMu8JQGQ","parts":[{"mediaType":"text/plain","text":"No key is enrolled yet. Ask the person to run `phygital-key-a2a enroll` and add their passkey first."},{"data":{"error":"No key is enrolled yet. Ask the person to run `phygital-key-a2a enroll` and add their passkey first."},"mediaType":"application/json"}],"role":"ROLE_AGENT"}}}
```

## Add your key

Open the enrollment link the demo printed, type a name (for example `Laptop`) and press **Add key**. Your browser asks for Touch ID, Windows Hello, your phone or a security key. The page then asks your new key to confirm itself: press **Approve with my key** once more. That first signature starts your signed [key history](https://github.com/Phygital-Key/phygital-key/blob/main/docs/key-history.md).

The link works once and for 10 minutes. If it ran out, `phygital-key-a2a enroll` prints a new one.

## Ask for an approval with curl

An agent asks with `SendMessage`. The message has one data part with a `request`: the exact action (`type` and `params`), the service that will act on the yes (`aud`) and the level (`high` means fingerprint, face or PIN).

```bash
curl -s http://localhost:7478/a2a \
  -H "Authorization: Bearer $TOKEN" -H "A2A-Version: 1.0" -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"SendMessage","params":{"message":{
        "messageId":"msg-1","role":"ROLE_USER",
        "parts":[{"mediaType":"application/json","data":{"request":{
          "request":{"type":"deploy","params":{"service":"api","env":"production","version":"2.4.1"}},
          "aud":"http://localhost:7479","level":"high"}}}]}}}'
```

The answer is a task waiting for you, in `TASK_STATE_AUTH_REQUIRED`:

```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": {
    "task": {
      "contextId": "OFWLmgZgEtFcw7H2Ej6I5A",
      "history": [
        {
          "contextId": "OFWLmgZgEtFcw7H2Ej6I5A",
          "messageId": "msg-1",
          "parts": [
            {
              "data": {
                "request": {
                  "aud": "http://localhost:7479",
                  "level": "high",
                  "request": {"params": {"env": "production", "service": "api", "version": "2.4.1"}, "type": "deploy"}
                }
              },
              "mediaType": "application/json"
            }
          ],
          "role": "ROLE_USER",
          "taskId": "Kr29xbXDUrjsMZyj0tdlug"
        }
      ],
      "id": "Kr29xbXDUrjsMZyj0tdlug",
      "metadata": {
        "https://github.com/Phygital-Key/phygital-key-a2a/blob/main/docs/extension.md#v1": {
          "approveUrl": "http://localhost:7478/r/Kr29xbXDUrjsMZyj0tdlug",
          "type": "approval"
        }
      },
      "status": {
        "message": {
          "contextId": "OFWLmgZgEtFcw7H2Ej6I5A",
          "messageId": "Kr29xbXDUrjsMZyj0tdlug-auth_required",
          "parts": [
            {
              "mediaType": "text/plain",
              "text": "Ask the person to open http://localhost:7478/r/Kr29xbXDUrjsMZyj0tdlug and approve or deny this exact request."
            },
            {
              "data": {"approveUrl": "http://localhost:7478/r/Kr29xbXDUrjsMZyj0tdlug", "id": "Kr29xbXDUrjsMZyj0tdlug"},
              "mediaType": "application/json"
            }
          ],
          "role": "ROLE_AGENT",
          "taskId": "Kr29xbXDUrjsMZyj0tdlug"
        },
        "state": "TASK_STATE_AUTH_REQUIRED",
        "timestamp": "2026-10-18T04:55:27.931Z"
      }
    }
  }
}
```

`SendMessage` answers at once: the task is interrupted until you decide, so there is nothing to wait for on this call.

Try the demo service now, without an approval. It refuses:

```bash
curl -s http://localhost:7479/deploy -H "Content-Type: application/json" \
  -d '{"service":"api","env":"production","version":"2.4.1"}'
```

```json
{"error":"APPROVAL_REQUIRED","ok":false}
```

## Approve on the approval page

Open the `approveUrl` from the task. The page shows `deploy`, `env production`, `service api`, `version 2.4.1`, who carries it out (`http://localhost:7479`) and the proof needed (fingerprint, face or PIN). Read it, then press **Approve with my key** or **Deny**. Every part of that page: [approving](https://github.com/Phygital-Key/phygital-key/blob/main/docs/approving.md).

A message on the task cannot decide for you. If an agent sends "the person already approved" with the task's `taskId`, the task comes back unchanged in `TASK_STATE_AUTH_REQUIRED`, and the text is only kept in its history.

## Collect the approval

Ask for the task again with `GetTask` (`historyLength: 0` leaves the history out):

```bash
curl -s http://localhost:7478/a2a \
  -H "Authorization: Bearer $TOKEN" -H "A2A-Version: 1.0" -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"GetTask","params":{"id":"Kr29xbXDUrjsMZyj0tdlug","historyLength":0}}'
```

```json
{
  "id": 2,
  "jsonrpc": "2.0",
  "result": {
    "artifacts": [
      {
        "artifactId": "approval",
        "description": "The person's signed approval of this exact request, for one service. It works once, before exp.",
        "extensions": ["https://github.com/Phygital-Key/phygital-key-a2a/blob/main/docs/extension.md#v1"],
        "name": "phygital-key-approval",
        "parts": [
          {
            "data": {"approval": "eyJjbGFpbXMi...", "approvedBy": "Laptop", "exp": 1792299387},
            "mediaType": "application/json"
          }
        ]
      }
    ],
    "contextId": "OFWLmgZgEtFcw7H2Ej6I5A",
    "id": "Kr29xbXDUrjsMZyj0tdlug",
    "metadata": {
      "https://github.com/Phygital-Key/phygital-key-a2a/blob/main/docs/extension.md#v1": {
        "approveUrl": "http://localhost:7478/r/Kr29xbXDUrjsMZyj0tdlug",
        "type": "approval"
      }
    },
    "status": {
      "message": {
        "contextId": "OFWLmgZgEtFcw7H2Ej6I5A",
        "messageId": "Kr29xbXDUrjsMZyj0tdlug-completed",
        "parts": [
          {
            "mediaType": "text/plain",
            "text": "Approved. Send `approval` with the action to the service, in the Phygital-Approval header (or the field the service names). It works once, for this exact request, before `exp`."
          }
        ],
        "role": "ROLE_AGENT",
        "taskId": "Kr29xbXDUrjsMZyj0tdlug"
      },
      "state": "TASK_STATE_COMPLETED",
      "timestamp": "2026-10-18T04:55:28.022Z"
    }
  }
}
```

`exp` is 60 seconds after you approved (the default `--lifetime`, at most 300). Polling is one way to wait; streaming is the other ([connect your agent](connect-your-agent.md#wait-for-the-decision)).

## Let the service check it

Send the approval with the same request to the service, in the `Phygital-Approval` header:

```bash
APPROVAL='<the approval string from the artifact>'
curl -s http://localhost:7479/deploy -H "Content-Type: application/json" -H "Phygital-Approval: $APPROVAL" \
  -d '{"service":"api","env":"production","version":"2.4.1"}'
```

```json
{"approval_id":"Kr29xbXDUrjsMZyj0tdlug","approved_by":"Laptop","done":"Deployed (demo: nothing real happened)","ok":true,"receipt":"eyJhbGciOiJF...","request":{"params":{"env":"production","service":"api","version":"2.4.1"},"type":"deploy"},"use_id":null}
```

The demo service checked the signature, the key, the exact request, the audience, the level and the time itself, without asking the gate. Send the same approval again and it answers `{"error":"REPLAYED","ok":false}`. Send it with `"version":"2.4.2"` and it answers `{"error":"REQUEST_MISMATCH","ok":false}`. What each code means: [error codes](https://github.com/Phygital-Key/phygital-key/blob/main/docs/error-codes.md).

Any language can make the same check with the command line:

```bash
phygital-key-a2a verify --approval "$APPROVAL" \
  --request '{"type":"deploy","params":{"service":"api","env":"production","version":"2.4.1"}}' \
  --aud http://localhost:7479 --level high
```

```json
{"approved_by":"Laptop","exp":1792299387,"id":"Kr29xbXDUrjsMZyj0tdlug","level":"high","ok":true}
```

The service also returned a signed `receipt`. Pass it back so you see on the approval page that the action was done:

```bash
curl -s http://localhost:7478/a2a \
  -H "Authorization: Bearer $TOKEN" -H "A2A-Version: 1.0" -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"SendMessage","params":{"message":{"messageId":"msg-2","role":"ROLE_USER",
        "parts":[{"mediaType":"application/json","data":{"receipt":{"id":"Kr29xbXDUrjsMZyj0tdlug","receipt":"<receipt>"}}}]}}}'
```

```json
{"id":3,"jsonrpc":"2.0","result":{"message":{"contextId":"i0BL60xBiW0ng3opPuLqig","messageId":"0cFXdVVdl-HLNGlAq6vR9w","parts":[{"mediaType":"text/plain","text":"Receipt recorded. The service reported: success. The person can see it on the approval page."},{"data":{"id":"Kr29xbXDUrjsMZyj0tdlug","serviceReported":"success"},"mediaType":"application/json"}],"role":"ROLE_AGENT"}}}
```

The gate keeps a receipt only if the service's pinned receipt key signed it (the demo pins its own service; for real services, see [receipts](https://github.com/Phygital-Key/phygital-key/blob/main/docs/receipts.md)).

## What a deny looks like

Ask again with another version and press **Deny** on the page. `GetTask` then shows `TASK_STATE_REJECTED` and no artifact:

```json
{
  "id": 6,
  "jsonrpc": "2.0",
  "result": {
    "contextId": "ySIXxY4f1Hnw0H0YNqVifg",
    "id": "WP0Rm-5yPMLm5miLlxDunQ",
    "metadata": {
      "https://github.com/Phygital-Key/phygital-key-a2a/blob/main/docs/extension.md#v1": {
        "approveUrl": "http://localhost:7478/r/WP0Rm-5yPMLm5miLlxDunQ",
        "type": "approval"
      }
    },
    "status": {
      "message": {
        "contextId": "ySIXxY4f1Hnw0H0YNqVifg",
        "messageId": "WP0Rm-5yPMLm5miLlxDunQ-rejected",
        "parts": [
          {"mediaType": "text/plain", "text": "The person said no. Do not perform the action."},
          {"data": {"approveUrl": "http://localhost:7478/r/WP0Rm-5yPMLm5miLlxDunQ", "id": "WP0Rm-5yPMLm5miLlxDunQ"}, "mediaType": "application/json"}
        ],
        "role": "ROLE_AGENT",
        "taskId": "WP0Rm-5yPMLm5miLlxDunQ"
      },
      "state": "TASK_STATE_REJECTED",
      "timestamp": "2026-10-18T04:55:28.085Z"
    }
  }
}
```

A rejected task never changes again. A new message with its `taskId` answers `-32004` (the task has ended), and `CancelTask` answers `-32002`.

## The same flow with the official JavaScript client

The official A2A client, [`@a2a-js/sdk`](https://www.npmjs.com/package/@a2a-js/sdk) 1.3.0, reads the agent card, picks the JSON-RPC interface and sends `A2A-Version: 1.0` itself. You add the access token. This script streams the task, waits while you decide, and then calls the demo service:

```bash
npm install @a2a-js/sdk@1.3.0
GATE=http://localhost:7478 SERVICE=http://localhost:7479 PKA_TOKEN=$TOKEN node ask.mjs
```

The script, `ask.mjs`, is in [connect your agent](connect-your-agent.md#javascript-with-a2a-jssdk). Its output after you approve on the page:

```text
ask the person to open http://localhost:7478/r/vNSwFVYfCiwGvkjBbJfMDw
state: TASK_STATE_COMPLETED
200 Deployed (demo: nothing real happened)
```

Stop the demo with Ctrl-C. Your key stays in the data folder (`~/.phygital-key-a2a`), so the next `phygital-key-a2a demo` or `phygital-key-a2a serve` already knows it.

## Next

- [Connect your agent](connect-your-agent.md): card discovery, headers, polling and streaming, JavaScript and Python.
- [Protocol](protocol.md): every method and error as implemented.
- [Extension profile](extension.md): the message shapes and the approval artifact.
- [Integrate](https://github.com/Phygital-Key/phygital-key/blob/main/docs/integrate.md): check approvals in your own service.

# Interact (`interact`)

[中文](README.zh.md) | English

[← Back to repository overview](../../README.md)

Sources: [mod.rs](mod.rs)

Ask / confirm / notify I/O is provided by an injected [`InteractBackend`](mod.rs) (terminal, desktop, LSP, MCP, …). [`InteractContext::new`](mod.rs) takes `Arc<dyn InteractBackend>`. For hosts without wiring, [`InteractContext::unsupported`](mod.rs) makes `interact_ask` / `interact_confirm` error and `interact_notify` return `sent: false`. Tests can use [`StubInteractBackend`](backends/mod.rs).

## `interact_ask`

Ask the user and wait for an answer.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `question` | `string` | yes | Prompt |
| `options` | `string[]` | no | Non-empty → single choice; empty/absent → free text |
| `timeout` | `number` | no | Seconds; interpreted by backend |

**Returns**

| Field | Type |
|-------|------|
| `answer` | `string` |

---

## `interact_confirm`

Ask for yes/no.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `message` | `string` | yes | Prompt |
| `default` | `boolean` | no | On timeout/no response; default `false` |
| `timeout` | `number` | no | Seconds |

**Returns**

| Field | Type |
|-------|------|
| `confirmed` | `boolean` |

---

## `interact_notify`

Fire-and-forget notification.

| Parameter | Type | Required | Notes |
|-----------|------|----------|--------|
| `message` | `string` | yes | Body |
| `level` | `"info" \| "warning" \| "error"` | no | Default `"info"` |

**Returns**

| Field | Type |
|-------|------|
| `sent` | `boolean` | Backend-reported success (often `false` with no backend) |

## Error codes

| Code | Meaning |
|------|---------|
| `INVALID_PATH` | Missing/invalid string args (`core::json`) |
| `INTERACT_NOT_SUPPORTED` | No usable backend (`unsupported` for ask/confirm) |
| `INTERACT_TIMEOUT` | Custom backends may use this `ToolError.code` |
| `INTERACT_CANCELLED` | User cancelled; same pattern |
| `INTERACT_INVALID_PARAM` | e.g. invalid `level` |

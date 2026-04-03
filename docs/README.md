# agentool documentation

[中文](README.zh.md) | English

This folder indexes human-written docs for the **agentool** crate. API details also appear in `rustdoc` (`cargo doc --all-features --no-deps --open`).

## Where things live

| Topic | Location |
|--------|-----------|
| Repository overview & quick start | [README.md](../README.md) · [README.zh.md](../README.zh.md) |
| Per-feature tool specs (parameters, return fields, error codes) | `src/<feature>/README.md` (English) · `README.zh.md` (Chinese) |
| Rust types and traits | Source and `rustdoc` |

## Conventions

- **Cargo features** map to optional dependencies (see root `Cargo.toml`): enable only the tool modules you use to keep compile time and transitive deps small.
- **Default language** for top-level and feature READMEs is **English** (`README.md`).
- **Chinese** variants are **`README.zh.md`** in the same directory, linked from the English page.
- Error `code` strings are stable API; see each feature’s `error.rs` and its README tables.

## Reading paths

**New to the crate**

1. [README.md](../README.md) — install, features, response envelope  
2. Pick a feature table row and open `src/<feature>/README.md`  
3. Wire a context (`FsContext`, `WebContext`, …) as shown in that module’s docs  

**Looking up a tool**

- Use the feature README for JSON parameter names, types, and error codes.  
- Tool names are lowercase with underscores (e.g. `grep_search`, `memory_write`).

## Publishing

Tagged releases run `fmt`, `clippy`, and tests in CI, then `cargo publish`. See [.github/workflows/cargo-publish.yml](../.github/workflows/cargo-publish.yml).

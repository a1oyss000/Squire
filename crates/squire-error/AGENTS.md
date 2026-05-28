<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# squire-error

## Purpose
Shared error types for the entire workspace. Defines `SquireError` enum and the `Result<T>` type alias used by all crates.

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Single dependency: `thiserror` |
| `src/lib.rs` | `SquireError` enum with variants for each subsystem |

## For AI Agents

### Working In This Directory
- Add new error variants here when introducing new failure modes
- All variants use `#[error("...")]` for Display impl via thiserror
- Other crates depend on this — changes here affect the entire workspace

### Common Patterns
- Variant naming: `SubsystemName(String)` for generic errors
- Structured variants (e.g., `MatchFailed { confidence, threshold }`) for actionable errors
- `#[from]` for automatic conversion from std types (e.g., `std::io::Error`)

<!-- MANUAL: -->

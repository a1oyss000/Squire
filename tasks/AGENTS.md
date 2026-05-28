<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# tasks

## Purpose
YAML task definition files loaded by the engine at runtime. Each file defines a named automation sequence with steps, targets, and failure strategies.

## Key Files

| File | Description |
|------|-------------|
| `nikke-daily-mail.yaml` | Daily mail collection task for NIKKE |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `templates/` | Template images referenced by task steps (see `templates/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- Task files use the schema defined in `crates/squire-engine/src/config.rs`
- Template paths in steps are resolved relative to the project root at runtime
- Each task must have at least one step with a non-zero timeout

### Common Patterns
- Steps target by: `template` (image match), `ocr` (text find), or `coordinate` (fixed point)
- Actions: `click`, `wait`, `swipe`
- Failure strategies: `retry` (default), `skip`, `pause`

<!-- MANUAL: -->

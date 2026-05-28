<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-05-29 | Updated: 2026-05-29 -->

# templates

## Purpose
Image template files used by the vision matcher for locating UI elements on screen. Organized by game/application.

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `nikke/` | Template images for NIKKE game automation |

## For AI Agents

### Working In This Directory
- Templates are PNG images cropped from game screenshots
- Referenced by task YAML files via relative path (e.g., `templates/nikke/mail_button.png`)
- Keep templates small and distinctive for reliable matching
- Threshold defaults to 0.8 — lower for variable UI elements

<!-- MANUAL: -->

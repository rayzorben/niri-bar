# Configuration

YAML is the source of truth. It is validated against `src/niri-bar-yaml.schema.json`.

Key sections:
- `application.theme`: active CSS theme.
- `application.modules`: global module defaults (use anchors for DRY).
- `application.layouts`: reusable layouts (columns → `{ modules: [...], overflow: hide|kebab }`).
- `application.monitors`: ordered list of regex entries with `match`, `enabled`, `layout`, `modules`.

Monitor matching:
- Patterns like `^eDP-1$`, `^DP-.*$`, `.*`.
- Most specific pattern wins (exact > anchored wildcard > wildcard).

Merging rules:
- Start from global `modules`, overlay most-specific monitor `modules`.
- Layout resolved from most-specific matching monitor with non-empty columns, otherwise fall back to the first `application.layouts` entry.


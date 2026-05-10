# Contributing Documentation

OpenCrust’s docs live under the `docs/` directory. Follow these conventions to keep the documentation consistent and searchable.

## File naming

- Use **lower‑case‑kebab** for filenames, e.g. `architecture.md`, `keybinds.md`.
- Keep the main entry point `README.md` at the repository root.
- All top‑level guides (setup, development, troubleshooting) should be placed directly under `docs/`.

## Front‑matter (optional)

If you want the file to appear in generated site indexes, add a YAML block at the top:

```yaml
---
title: "Architecture"
description: "High‑level system diagram and component responsibilities"
date: 2026-05-10
---
```

## Linking & includes

- **Never duplicate large blocks** (architecture diagram, keybind table, module list). Store the canonical version in its own file and reference it with a markdown link, e.g.:

  ```markdown
  See the full keybind reference: [docs/KEYBINDS.md](./docs/KEYBINDS.md)
  ```

- For short reusable snippets you can use the project’s include syntax (e.g. `{{#include path/to/snippet.md}}`) if your rendering pipeline supports it.

## Structure recommendations

- **Tier‑1 (Essential)**: `README.md`, `CONTRIBUTING.md`, `docs/MODULES.md`
- **Tier‑2 (Deep‑dives)**: `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/CONFIGURATION.md`, `docs/PERFORMANCE.md`
- **Tier‑3 (Practical)**: `docs/EXAMPLES.md`, `docs/TESTING.md`, `docs/TROUBLESHOOTING.md`

Keep the tier taxonomy only in `docs/README.md` (or a dedicated `docs/DOCS_STRUCTURE.md`). Other files should link to it rather than duplicating the list.

## CI checks

All markdown files are validated in CI:

- **Link checker** (`scripts/check_md_links.sh`) – ensures every internal link points to an existing file.
- **Formatting** – run `cargo fmt` on Rust code; markdown files should follow consistent heading hierarchy (H1 → H2 → H3, no skipping).

## Quick checklist before submitting a PR

- [ ] Run `scripts/check_md_links.sh` – no broken links.
- [ ] Check that any new file follows the naming convention (kebab‑case).
- [ ] Verify that large blocks (diagrams, tables) are not duplicated – link to the canonical source instead.
- [ ] Preview the rendered markdown locally (e.g. with a markdown preview extension) to catch formatting issues.

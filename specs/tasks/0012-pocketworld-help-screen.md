---
id: 0012-pocketworld-help-screen
status: draft
scope: [src/pocketworld/effect.rs, src/pocketworld/render.rs]
created: 2026-10-13
source: user request 2026-10-13: no text on the world render — the LAYER HUD row goes away; a separate full help screen (keys, views) toggles on 'h', mirroring the legend page on 'l'
---
# Task: pocketworld help screen

## Plan

The world render carries a text HUD (`LAYER: … [j/k] legend:l`) burned into row 0 — it pollutes the render and the analyzer census. Replace it with a proper help page on its own screen.

- [ ] Remove the HUD row from `get_diff` — world renders show only the world.
- [ ] `h` toggles a full-screen help page (own page like the legend on `l`): keys (`q` quit, `j`/`k` cycle views, `l` legend, `h` help), the seven view names, seed if cheap to show.
- [ ] Legend page (`l`) unchanged; both pages toggleable from each other.
- [ ] Keys stay display-only (no simulation controls).

## Acceptance

- [ ] `cargo fmt --check`
- [ ] `cargo test --lib`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `tarts --analyze pocketworld --view biome --frames 60 --dump` shows no `LAYER:` text on row 0
- [ ] Manual: `h` shows the help page, `h` again returns; `l` legend still works (user look)

## Log

- 2026-10-13 created: pocketworld help screen
- 2026-10-13 set source user request 2026-10-13: no text on the world render — the LAYER HUD row goes away; a separate full help screen (keys, views) toggles on 'h', mirroring the legend page on 'l'
- 2026-10-13 scope add src/pocketworld/effect.rs — remove the HUD row from world renders; add an 'h'-toggled full help screen
- 2026-10-13 scope add src/pocketworld/render.rs — remove the HUD row from world renders; add an 'h'-toggled full help screen

---
id: 0013-effect-analyzer-skill
status: draft
scope: [.agents/skills/effect-analyzer/SKILL.md, src/analyze.rs, src/main.rs, AGENTS.md]
created: 2026-10-14
source: user request 2026-10-14: a skill that helps agents debug effects with the --analyze subcommand
---
# Task: effect analyzer skill

## Plan

The agent cannot see renders. `tarts --analyze` is its eyes, but each session re-learns how to read it. A skill writes the workflow down once. Two small analyzer additions close the blind spots found in the 0006 diagnosis (2026-10-14). `.agents/skills/` is the tracked skill home; `.claude/skills/` is a generated mirror.

- [ ] `.agents/skills/effect-analyzer/SKILL.md` — frontmatter that triggers on debugging or tuning an effect's look, analyzer, or census. Sections: why (measure, don't guess); build (`cargo build --release`, `./target/release/tarts`); baseline protocol (seeds 7 + 42, frames 300/1500/3000, every view the change touches, numbers into the task Findings); reading the report (census, distinct, `--dump`, CHANGE vs FLICKER); cross-view reasoning (dump two views at the same seed/frames/size and line them up by row/col — e.g. height `▓` under biome forest exposed the missing rock tier); before/after (same command, `diff` the dumps in a scratch dir); gates as census thresholds.
- [ ] Skill pitfalls section: HUD text on row 0 until 0012; the analyzer ignores user config (focus mode); only pocketworld is seeded (0009); glyph-only, no color; default canvas 100x40.
- [ ] Analyzer `FLICKER` line: mean full-cell change between consecutive frames over the run. This is shimmer, separate from CHANGE (drift between far-apart samples). It reuses the shadow-buffer loop in `analyze()`.
- [ ] Analyzer `--exclude <GLYPHS>`: drops those glyphs from the census and its total (`--exclude "≈~"` gives land-only shares). Parsed in `main.rs` next to `--view`; help text updated.
- [ ] Unit tests in `analyze.rs`: flicker is 0 for a static effect; exclude shrinks the total and re-normalizes shares.
- [ ] AGENTS.md: one line pointing to the skill.

## Acceptance

- [ ] `cargo fmt --check`
- [ ] `cargo test --lib`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `tarts --analyze pocketworld --view biome --frames 300 --exclude "≈~"` reports a census total below 4000
- [ ] `tarts --analyze pocketworld --view biome --frames 300` prints a `FLICKER` line

## Log

- 2026-10-14 created: effect analyzer skill
- 2026-10-14 set source user request 2026-10-14: a skill that helps agents debug effects with the --analyze subcommand
- 2026-10-14 scope add .agents/skills/effect-analyzer/SKILL.md — analyzer debugging skill + frame-to-frame flicker metric + census exclude flag
- 2026-10-14 scope add src/analyze.rs — analyzer debugging skill + frame-to-frame flicker metric + census exclude flag
- 2026-10-14 scope add src/main.rs — analyzer debugging skill + frame-to-frame flicker metric + census exclude flag
- 2026-10-14 scope add AGENTS.md — analyzer debugging skill + frame-to-frame flicker metric + census exclude flag

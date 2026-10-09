# Ideas

Uncommitted possibilities — noted but not decided to do right now. Promote to `roadmap.md` only when we decide to pursue something.

## WASM showcase page

Feasible. Effects use crossterm only for `Color`/`Attribute` enums; rendering goes through `src/buffer.rs` (`Buffer`/`Cell`) and `update()` logic is pure. Path: wasm-pack build of the lib with a headless render backend that replays Buffer diffs to a `<canvas>`/`<pre>`; keep `common::run_loop` native-only. Needs a small platform split (crossterm behind a feature flag or trait). Watch: `ThreadRng` (needs getrandom js feature), `std::time`, and any effect writing outside its Buffer.

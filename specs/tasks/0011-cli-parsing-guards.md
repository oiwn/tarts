---
id: 0011-cli-parsing-guards
status: draft
scope: [src/main.rs, tests/cli_parsing.rs]
created: 2026-10-13
source: user report 2026-10-13: 'cargo run -- analyze pocketworld ...' (missing dashes) silently launched the interactive effect — unknown positionals and malformed/missing flag values are swallowed instead of erroring
---
# Task: cli parsing guards

## Plan

Bad parses must die loudly. Today an unknown positional is swallowed (last one wins), a value-taking flag at end of argv is ignored, and a malformed `--frames`/`--seed` value falls back to the default — every typo produces plausible-but-wrong behavior.

- [ ] Positionals: validate each against `VALID_SAVERS` and error `unknown screen saver '<name>'` (covers both the interactive and `--check`/`--analyze` positional). Typo case to kill: `tarts analyze pocketworld ...`.
- [ ] `--effect <NAME>`: validate its value against `VALID_SAVERS` too.
- [ ] Value-taking flags (`--effect`, `--frames`, `--seed`, `--view`): missing value → error naming the flag; `--frames`/`--seed` non-numeric → error echoing the bad value.
- [ ] All errors through the existing `Error parsing args:` path (exit 1, no alternate screen entered).
- [ ] `tests/cli_parsing.rs`: integration test via `env!("CARGO_BIN_EXE_tarts")` — unknown positional exits non-zero with the message; missing/malformed values exit non-zero; a valid `--analyze` invocation still exits 0.

## Acceptance

- [ ] `cargo fmt --check`
- [ ] `cargo test --lib`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --test cli_parsing`
- [ ] `tarts analyze pocketworld --view height` (missing dashes) errors with `unknown screen saver: analyze` and exits non-zero

## Log

- 2026-10-13 created: cli parsing guards
- 2026-10-13 set source user report 2026-10-13: 'cargo run -- analyze pocketworld ...' (missing dashes) silently launched the interactive effect — unknown positionals and malformed/missing flag values are swallowed instead of erroring
- 2026-10-13 scope add src/main.rs — harden parse_args: validate positionals and --effect against VALID_SAVERS, error on missing/unparseable flag values; integration test via CARGO_BIN_EXE
- 2026-10-13 scope add tests/cli_parsing.rs — harden parse_args: validate positionals and --effect against VALID_SAVERS, error on missing/unparseable flag values; integration test via CARGO_BIN_EXE

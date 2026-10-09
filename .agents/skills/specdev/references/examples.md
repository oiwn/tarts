# Examples: Spec Editing and Compression

## Example 1: Addressing a remark

**Before** (user added a remark to `ctx.md`):

```md
## Next Steps

- [ ] Implement user authentication
- [ ] Add session management

^^^ Should we use JWT or session cookies here?
```

**After** (agent addresses the remark):

```md
## Next Steps

- [ ] Implement user authentication
- [ ] Add session management

^^^ Should we use JWT or session cookies here?
&&& Addressed: session cookies. JWT adds complexity for rotation and
revocation that we don't need. Cookies with `httpOnly` + `sameSite` are
simpler and sufficient for this project.
```

## Example 2: Addressing with a clarifying question

**Before:**

```md
## Architecture

The API gateway routes requests to microservices.

^^^ Not sure about microservices — is this project big enough?
```

**While waiting:** Leave the spec unchanged and ask the user in conversation
what scale they expect. The remark remains unresolved, so do not add an
`&&&` answer yet.

## Example 3: Compression

**Before compression** (all remarks resolved):

```md
## Data Pipeline

We need to import CSV files from the legacy system.

^^^ What format are the CSV files in?
&&& Addressed: standard RFC 4180 CSV with UTF-8 encoding, semicolon
delimiters, and a header row.

The import runs nightly.

^^^ Should we validate before or after import?
&&& Addressed: validate before import. Reject the entire batch if any row
fails validation, and log the failing rows for manual review.

^^^ How do we handle duplicates?
&&& Addressed: upsert on a composite key of (source_id, timestamp). Existing
records get updated; new records get inserted.
```

**After compression** (user requested "compress this spec"):

```md
## Data Pipeline

Import CSV files from the legacy system. Files use standard RFC 4180 format
with UTF-8 encoding, semicolon delimiters, and a header row.

The import runs nightly. Validation happens before import: reject the entire
batch if any row fails, and log failing rows for manual review. Handle
duplicates via upsert on composite key (source_id, timestamp). Existing
records are updated; new records are inserted.
```

The dialogue is gone. The decisions remain as clear prose.

## Example 4: Partial compression

**Before compression** (one remark still open):

```md
## API Design

^^^ REST or GraphQL?
&&& Addressed: REST. The API surface is small and well-defined. GraphQL
adds complexity we don't need.

^^^ Should the API be versioned?
&&& Addressed: yes, URL-based versioning (/v1/, /v2/).

^^^ What about rate limiting?
```

**After compression:**

```md
## API Design

REST API with URL-based versioning (/v1/, /v2/). The API surface is small
and well-defined, so GraphQL is not needed.

^^^ What about rate limiting?
```

The resolved dialogue is compressed. The open question stays as `^^^`.

## Example 5: Routing a cluttered ctx.md

The most common failure mode: an agent dumps scope, deferred items, roadmap
pointers, and gotchas into `ctx.md`. The fix is **not** to rewrite the prose —
it is to route each piece to its right home.

**Before** (cluttered `ctx.md`):

```md
Active task: **portal overview page (v1)** — implementation not started.
Roadmap pointer: this supersedes/absorbs the dashboard part of v1.8 in
`specs/roadmap.md`.

Scope:
1. PriceTicker on / — reuse `latest_price_snapshots("usd")`.
2. Trending-entities table — top entities in a 7-day window anchored to the
   latest record's `publication_date`, not wall-clock "now".

Deferred, own follow-up discussions:

- LLM-generated daily content — how to form the LLM's context needs a
  dedicated discussion before any subcommand is designed.
- Charts — deferred; v1 uses tables instead.

v1.9 (calendar/daily-digest page) remains separately unstarted.

## SurrealDB quirks

- `FETCH in` silently fails on relation tables — use `in.field AS alias`
- Case-insensitive search: `string::lowercase(field) CONTAINS $query`
```

**After** — content routed to its home:

`specs/ctx.md` (only the current task, as a checkbox plan):

```md
# Current Task Context: portal overview page (v1)
State: in progress

## Plan
- [ ] PriceTicker on / — reuse `latest_price_snapshots("usd")`
- [ ] Trending-entities table — 7-day window anchored to latest publication_date

## Context
Need a date-bounded variant of top_entity_summaries (repository.rs). Window
anchors to the latest record's publication_date, not wall-clock now.

## Next
Implement the trending-entities query.
```

`specs/roadmap.md` (committed future direction):

```md
- v1.9 calendar/daily-digest page (separate, not yet started)
```

`specs/ideas.md` (uncommitted possibilities):

```md
- LLM-generated daily content: needs a discussion on how to form the LLM's
  context before designing a wisdom subcommand.
- Charts: deferred for v1 (tables instead); revisit rendering approach if back
  in scope.
```

`specs/overview.md` → Gotchas / Knowledge:

```md
## Gotchas / Knowledge

### SurrealDB
- `FETCH in` silently fails on relation tables — use `in.field AS alias`.
- Case-insensitive search: `string::lowercase(field) CONTAINS $query`.
```

Nothing was lost. Everything went where it belongs, and `ctx.md` is now narrow
enough that any agent can resume the task in one read.

## Example 6: Small task, immediate approval

Task 0007 changes one caption string. Acceptance holds the repo's hook commands; there are no Manual checks.

1. `task new caption-parens`, `task scope 7 add src/components/caption.rs --reason "the literal"`, write Plan, Acceptance, Summary. User: "go".
2. `advance 7` (ready), `advance 7` (implement), edit, tick Plan, run Acceptance and tick, write one Review line, then `advance 7 --to approval` — one call walks verify and review, checking each gate.
3. Agent: "Caption now reads `… (circles scaled linearly by radius)`. Checks pass."
4. User: "looks good".
5. `specdev task done 7 --approval "looks good"`, point `ctx.md` at the next task. No further questions.

## Example 7: Feedback adds a requirement

Task 0008 is in approval. User: "headings look good, but translate the pagination too".

- That is a revision, not acceptance. `advance 8 --to fix` (logged as a revision; it doesn't use the repair budget).
- Add a Plan step for pagination; if it touches new files, `task scope 8 add <paths>... --reason "pagination labels"`.
- Replace the Findings/Review paragraphs with the current state; don't add a new paragraph per round.
- Verify, review, approval, show the user again.

## Example 8: Batch acceptance and closing before the PR commit

Tasks 0009, 0011, and 0013 are in approval. Agent: "0009 icon, 0011 counter links, 0013 abbreviations are ready." User: "all three look good".

1. `specdev task done 9 11 13 --dry-run` → shows any blocker per task. 0011 has a Manual check left: "zh-CN mobile layout".
2. Agent asks once: "0011 still has the zh-CN mobile check — did you look, or waive it?" User: "skip it".
3. Tick: `- [x] zh-CN mobile layout — waived by user: "skip it"`.
4. `specdev task done 9 11 13 --approval "all three look good"`. Reconcile `ctx.md`.
5. Agent: "Closed 0009, 0011, 0013. Worktree passes `specdev check` and the hooks; ready for you to commit." The user commits (version bump included), merges, deploys.

## Example 9: A required check fails after it looked done

Task 0010 is in verify. The focused tests passed, but the repo's hook command `cargo test --workspace --all-features` fails.

- `advance 10 --to fix` from verify counts as a repair attempt. Fix, rerun the *full* command, tick, continue.
- Add the full command to Acceptance (and to `[acceptance] default` in `specdev.toml`) so the next task runs it.

## Example 10: Defect found after a task closed

0012 is in `tasks/done/`; the user reports a broken sitemap link.

- Don't reopen 0012. `task new sitemap-link-fix`, `task set 15 depends 0012-seo-audit-fixes`, `task set 15 source "user report: broken sitemap link"`.
- The history of 0012 stays intact; the fix is its own task.

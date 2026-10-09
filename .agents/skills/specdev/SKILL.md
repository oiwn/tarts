---
name: specdev
description: >
  Use when working in a project that uses specification-driven development.
  Start from specs/overview.md and specs/ctx.md, treat specs as the active
  task context, keep ctx.md concise and decision-relevant, route content to
  its right home (changelog/roadmap/ideas/overview), address user remarks
  marked with ^^^ and answer with &&&. Drive task files in specs/tasks/
  through the specdev CLI (task new/advance/scope/done, check) instead of
  editing their state by hand. Trigger when the user asks to work on specs,
  continue from specs, pick up or create a task, address remarks, compress
  specs, or when entering a project with a specs/ directory.
---

Teaches the agent to work with project specs as the primary planning, coordination, and memory surface. Specs are Markdown files in a `specs/` directory. The agent reads them, updates them, and uses them as context before writing code. The `specdev` CLI owns the structured parts (task state, logs, indexes) and checks everything; the agent owns the prose.

## Invariants

Hold these at all times. They exist because agents silently drift on exactly these points.

- **Task identity is immutable by default.** The `ctx.md` title and a task file's `# Task:` title name the work. Don't rename, narrow, or replace it without explicit user intent.
- **Never bulk-replace an active `ctx.md` or task file.** Make the smallest surgical edit that achieves the change.
- **Done needs acceptance.** A task closes after it is verified, reviewed, and accepted by the user, not just because the steps are done or tests pass.
- **Never manufacture evidence.** Tick only what you ran or what the user said they checked. "Looks good" accepts the result; it doesn't prove each listed check was performed.
- **Preserve decision-relevant state.** "Concise" means concise and decision-relevant, not minimal.
- **Structured state goes through commands.** Never hand-edit task frontmatter, the `## Log` section, or `specs/tasks/_index.md`. `specdev check` detects hand edits from the task's own log and fails.

**Authorization.** "Do this task" authorizes the routine steps inside it: planning, scope inside the stated goal, implementing, verifying, state commands, ticking boxes, closing after acceptance, and tidying `ctx.md`. Don't ask again for those. Ask for real decisions: unclear or growing scope, a genuine failure, anything destructive, and anything public. Git (staging, commits, branches, PRs, merges) and deploys belong to the user unless they delegate them.

## Session startup

When entering a project, or when the user says "continue from specs":

1. Read `specs/overview.md` (durable project backdrop + gotchas).
2. Read `specs/ctx.md` (the current focus). If it points at a task file, read that file and run `specdev task show <id>` for its state and next step.
3. Run `specdev check` to see what is broken or drifting before you change anything.
4. If there is no active work (`ctx.md` is just the `# Current Task Context` header and no task is in progress), ask the user what to do, or take the next item from `specs/roadmap.md` or the task queue (`specdev task list`). The header-only state is valid, not an error.
5. Read `specs/ideas.md` only for brainstorming or roadmap work, or when `ctx.md` points there. Don't read `specs/cleanup.md` unless the user asks.
6. Identify any `^^^` remarks in relevant specs (see below).

## Content routing — put each thing in its right home

Content is routed by *when* it lives. This keeps `ctx.md` and task files clean.

- **`specs/ctx.md`** — the current focus: a pointer to the active task file and the next action, or a design/fuzzy task with its own checkbox plan. No history.
- **`specs/tasks/<id>.md`** — one task, sized to land as one commit: plan, scope, acceptance, review, log.
- **`CHANGELOG.md`** (root) — finished work. `specdev task done` writes entries for task files; ctx tasks are moved here by hand under the archival gate.
- **`specs/roadmap.md`** — committed future direction.
- **`specs/ideas.md`** — noted but not-now ideas.
- **`specs/overview.md`** — architecture, data flow, durable knowledge, gotchas and quirks.
- **`specs/cleanup.md`** — code smells and refactors spotted.

`CHANGELOG.md` and `AGENTS.md` live at the project root, not under `specs/`. Content routing does not expand the user's edit scope: if the user limits edits to named files, record follow-ups in the active task and leave other files untouched.

**Forbidden inside `ctx.md`:** `Deferred` sections, `Roadmap pointer` lines, `Gotchas`/`Quirks` blocks, architecture essays, and "completed work is not repeated here" meta-notes. Route them to their home instead.

## The working loop

Content matures in one direction, then archives: `ideas.md` (uncommitted) → `roadmap.md` (committed) → `ctx.md` or `specs/tasks/` (active) → `CHANGELOG.md` (done).

- An idea moves to `roadmap.md` when decided; a roadmap item becomes active work when picked up.
- Small, well-defined work (one commit) becomes a **task file** and runs through the task pipeline below. A roadmap item bigger than one commit is split into several task files ordered by `depends`.
- Design work and big, fuzzy work run directly in **`ctx.md`** with a checkbox plan.
- When nothing is active, the next item comes from `roadmap.md` or the task queue.

## ctx.md — the current focus (soft template)

`ctx.md` is **concise and decision-relevant, not minimal**. Recommended shape:

```md
# Current Task Context: <one line>
State: <not started | in progress | blocked>
## Plan
- [ ] step one
- [ ] step two
## Findings      # optional — concise research/results when the next step is evaluate/discuss
## Context
<what the current unchecked step needs; or: "Active task: specs/tasks/0007-slug.md">
## Next
<the immediate next action>
```

- The file always starts with `# Current Task Context`; append `: <task>` when work is active. The header **is the task identity**. The no-task state is header-only.
- When the work is a task file, keep `ctx.md` to the one-line focus plus a pointer to `specs/tasks/<id>.md` and the next action; the plan lives in the task file. specdev never writes `ctx.md`; `check` warns when it points at a task that isn't active.
- **Reconcile after each state change, without asking:** update the pointer and `## Next`, drop stale pointers, status lines, verification counts, and handoff notes that the task file or CHANGELOG already records. Never drop open decisions, `^^^` remarks, or findings not recorded elsewhere.
- `## Plan` steps are children, not the task: completing one is never permission to reset or archive.
- `## Findings` (optional): compact research, decisions, results. Never delete without explicit reset intent.
- Tick boxes (`[ ]` -> `[x]`) as you implement, with surgical edits.

## Task files and the pipeline

A task file is `specs/tasks/<id>.md`: flat-YAML frontmatter (owned by specdev), a Markdown body (yours), and a `## Log` at the end (owned by specdev). The contract (required sections, extra fields, attempts cap, acceptance defaults, quality limits) is in `specdev.toml`.

```md
---
id: 0007-status-freshness
status: in-progress
stage: verify
scope: [src/status.rs, tests/cli.rs]
created: 2026-10-02
source: https://github.com/org/repo/issues/12
---
# Task: warn when ctx.md is stale while in progress

## Plan              checkboxes — the steps
## Acceptance        checkboxes — exact commands you run at verify
## Manual checks     optional checkboxes — required human-visible checks; gate `done`
## Suggested checks  optional plain bullets — nice-to-have; never gate anything
## Summary           one line — becomes the CHANGELOG entry
## Context / Findings / Review / Spec updates   optional
## Log               specdev only
```

**Who owns what:**

- **Commands own:** `status`, `stage`, `scope`, `attempts`, `blocked_reason`, `source`, `depends`, the `## Log`, the file's location, and `_index.md`.
- **You own:** the title (set once at creation), the prose sections, ticking Plan and Acceptance boxes, ticking Manual checks with evidence (see "Approval and closing"), and closing the task once the user accepts it.
- **The user owns:** acceptance, waivers, and git: branches, commits, the PR, the merge, the deploy. specdev never writes to git and never runs project commands.

**Writing a task file — keep it short:**

- Acceptance items are exact commands with their flags (`cargo test --workspace --all-features`), not prose. Include what the repo's commit hooks and CI run (read `prek.toml`, `.pre-commit-config.yaml`, CI workflows); `[acceptance] default` in `specdev.toml` should hold those.
- Manual checks: at most three short items, only what a human must see (a layout, a flow). Everything else is a Suggested check or nothing.
- Findings and Review hold the *current* state. When a revision supersedes an entry, replace it; don't prepend another paragraph per round. History lives in the log and git.
- Don't restate test counts, dates, or approvals in several sections. Say it once.

**Stages, in order** (happy path: draft → ready → implement → verify → review → approval → done):

1. **draft** — `specdev task new <slug>`; `specdev task set <id> source "<issue or request>"`; add files with `specdev task scope <id> add <path> --reason "..."`; write Plan and Acceptance (`task new` pre-fills project defaults). The human reviews with `^^^`; you answer with `&&&`. When the human approves, compress the dialogue into the plan, then `specdev task advance <id>` → ready. It refuses while `^^^` remarks are open, and it records the approved scope.
2. **ready** — queued. `specdev task advance <id>` → in-progress/implement; refused while another task is in progress (one in the works at a time; tasks waiting in approval don't count).
3. **implement** — code only inside `scope`. Need another file? `specdev task scope <id> add <path>... --reason "..."` *before* touching it; a large expansion means the plan was wrong — say so. Tick Plan boxes; `specdev task advance <id>` → verify (refused while Plan boxes are unticked).
4. **verify** — run every Acceptance command yourself and tick each one that passes; specdev only checks the ticks. During development use focused checks; before ticking, run the full scope the repo's hooks and CI run (same features, targets, flags). Don't rerun a passing broad suite unless code changed since. All pass → `specdev task advance <id>` → review. Any fail → `specdev task advance <id> --to fix`.
5. **review** — judge the diff against Plan and scope (ideally a fresh session); write `## Review` (required to leave review). Approve → `specdev task advance <id>` → approval. Changes needed → `--to fix`.

**One call instead of three.** When the work is done — Plan ticked, Acceptance run and ticked, `## Review` written — move straight there: `specdev task advance <id> --to approval`. It walks implement → verify → review → approval, checks every gate on the way, logs every step, and writes nothing if a gate fails ("stopped at verify → review: …"). It also works from `ready` (`--to implement` is a plain step). It never walks out of `draft`: approving the draft is the user's step. Don't advance stage by stage when one walk does it, and don't run `check` between steps.
6. **fix** — address `## Review` notes, user feedback, or failures; `specdev task advance <id>` → verify. Only a *failed verify* (verify → fix) counts as a repair attempt; past `max_attempts` the task is blocked automatically. Review notes and user feedback (review/approval → fix) are revisions and are logged, not counted.
7. **approval** — write a one-line `## Summary`, then show the user what changed and what remains. Wait for their acceptance (see below). You may start another authorized task meanwhile; this one doesn't hold the slot.
8. **done** — once the user accepts: `specdev task done <id> --approval "<their words>"`. It logs the approval, archives to `specs/tasks/done/`, writes the CHANGELOG entry, and updates the index. It never touches git. Close tasks *before* the final PR commit, so the commit carries the finished docs; merge and deploy are release events, not part of done. Refused while Manual checks are unticked or the Summary is missing.

**Blocked** means a real obstacle stops progress: a missing decision, a broken dependency, an exhausted repair budget. Waiting for review, acceptance, a merge, or a deploy is *not* blocked. `specdev task block <id> --reason "..."` from any open stage; the user decides where it goes next: `specdev task advance <id> --to <draft|ready|implement|verify|review|fix>`.

## Approval and closing

**What counts as acceptance:** clear positive feedback on the delivered task when no requested work is left: "looks good", "ok done", "ship it", "let's move to the next task". Record it; don't ask the user to repeat it in another form.

- Partial praise is not acceptance: "the icon looks good, but fix the filter" → revision (`--to fix`), not done.
- Acceptance covers what the user saw. If you change code afterwards, re-verify; if the change is material, show it and get acceptance again.
- A batch approval ("all of these look good") covers only the tasks you just presented, by id. Never unrelated, future, or unfinished tasks. Name the ids back in your reply.

**Manual checks** gate `done`; each must be ticked with how it was resolved:

- `- [x] <check> — agent: <what you ran or looked at>` — you did it (e.g. browser automation, curl).
- `- [x] <check> — user: "<their words>"` — the user said they did this specific check.
- `- [x] <check> — waived by user: "<their words>"` — the user explicitly skipped it.

General acceptance ("looks good") does not tick specific checks by itself. If required checks remain, list them in one short line and ask once: check, or waive? Don't make the user walk a long checklist; that's why Manual checks stay at three items or fewer.

**Closing** (agent-owned after acceptance, no further permission): `specdev task done <ids...> --approval "<quote>"`, use `--dry-run` first for a batch, then reconcile `ctx.md`. The worktree is now ready for the user to commit. "Ready to commit" means the whole worktree passes `specdev check` and the repo's hooks, not just the task.

**Defects after close:** the task stays done. Create a follow-up task (`task new`, then `task set <new> depends <old>`) with the defect as its source. Before close, use `--to fix` on the open task instead.

**Useful anytime:** `specdev task show <id>` (state, progress, log, next step), `specdev task list` (queue order), `specdev task index` (regenerate `_index.md`). Ids accept the full id, the number (`7`), or the slug.

## specdev check

State commands already refuse changes that would fail `check`, so you don't need `check` after each one. Run it once per logical step (after implementing, before handing work back, before the user commits). Projects can also run `specdev check --staged` as a pre-commit hook (scope violations become warnings there). Errors exit 1; warnings don't. Size warnings for files you didn't touch are summarized in one line; they aren't regressions, so don't chase them mid-task (`--verbose` lists them).

- **Contract:** required sections, `stage` only while in progress, `blocked_reason` only while blocked, `depends` exist, id matches the file name. Empty scope, Plan, or Acceptance is a warning in draft and an error after.
- **Log consistency** (`log-*`, `log-mismatch`): the frontmatter must equal what the `## Log` replays to. A mismatch means someone edited state by hand. Don't "repair" it by editing the log; tell the user what differs and let them decide.
- **Scope** (`out-of-scope`): changed files (from git) outside the active task's scope. Add them with `task scope add --reason`, or revert the change. Anything under `specs/` and the project's `[scope] always_allowed` globs never count. Files already changed when the task started (earlier uncommitted work) are recorded as its baseline and don't count until you edit them again — never widen scope just to cover someone else's changes. Prefer exact paths; use a glob only when the task really edits most of what it matches.
- **Repo:** one task in progress (`active-tasks`; tasks in approval don't count), `_index.md` up to date (`index-stale` → `specdev task index`).
- **Quality** (`quality-*`, `md-table`, `ascii-diagram`): files over their line or word limits, too many plan steps, too large a scope. Act on them: split the task, route content out of `ctx.md`, compress older CHANGELOG entries.
- **Format** (`unformatted`): run `specdev fmt` rather than re-wrapping by hand.

**Refusals are the safety net.** A state command that would leave the task failing `check` refuses and prints the diagnostics ("refused: this change would make `specdev check` fail"). Fix the cause in the prose or with the right command; never work around a refusal by editing frontmatter or the log.

## CLI tools

- `specdev init` — scaffold `specs/` (overview, ctx, roadmap, ideas, cleanup, `tasks/`, `tasks/done/`), `specdev.toml`, and root `CHANGELOG.md`/`AGENTS.md` (append-aware). Safe to re-run.
- `specdev task new|list|show|index|advance|block|set|scope|done` — the task pipeline above.
- `specdev task advance <id> [--to <stage>]` — next step, or walk forward to a later stage in one call (every gate checked).
- `specdev task done <id>... [--approval "<quote>"] [--dry-run]` — close one or several accepted tasks; checks them all before writing anything.
- `specdev task scope <id> add <path>... --reason "..."` — several paths in one call.
- `specdev check [--staged] [--verbose]` — everything above.
- `specdev fmt [<file>...]` — normalize Markdown in `specs/` and open task files: soft-wrap paragraphs, `-` bullets, blank lines around headings. Code, tables, links, and `^^^`/`&&&` lines stay as written.
- `specdev scan` — `^^^`/`&&&` remarks across `specs/` and open task files, open vs resolved.
- `specdev status` — health summary: markers, ctx progress, ctx warnings.
- `specdev list [--stats]` — spec files with header and size; `--stats` adds headings, checkboxes, remarks, words, code blocks, tables.
- `specdev skill install [--local]` / `specdev skill check` — install or check this skill.
- `--format json|toon` (global) — machine-readable output for `check`, `task`, `scan`, `status`, `list`, `fmt`.

## Formatting specs

Specs are often read in a terminal editor where rendered Markdown is unavailable. Write for plain text:

- **Plain, simple English; say less.** Short sentences, common words, no filler or restating. Keep decisions, facts, and next steps; drop narration, hedging, and background the reader already has. If `check` reports `quality-words`, cut before you split.
- **No Markdown tables.** One bullet per item, key in bold: `- **verify** — run Acceptance; pass → review, fail → fix.`
- **No ASCII diagrams.** Describe flows as numbered steps; a one-line `draft → ready → done` is fine.
- **Bullets and short sections** over long paragraphs. One idea per bullet.
- **Soft wraps.** One line per paragraph or bullet; `specdev fmt` fixes hard wraps.
- **Code blocks only for literal content** — file formats, frontmatter, commands, config.
- **Shallow headings.** `##` for sections, `###` sparingly.

## Marker protocol

`^^^` and `&&&` are line-start markers used **only inside spec and task files**:

- `^^^` = user remark, question, correction, or TODO.
- `&&&` = agent answer addressing a nearby `^^^`.

Rules:

- Never write `^^^`; that marker belongs to the human. Answer with `&&&`.
- Add the `&&&` answer adjacent to its `^^^`; don't delete the `^^^` line.
- Leave a remark you can't resolve as `^^^` with no answer.
- Never use these markers in code, comments, or conversation.

## Addressing remarks

1. Read the file and find all `^^^` remarks.
2. For each, answer it with an adjacent `&&&`, or leave it open if you need the user. When a remark corrects nearby prose, update that prose too.
3. Don't start code while relevant `^^^` remarks are unresolved (for task files, `advance` enforces this at draft → ready).
4. Reply with what was addressed, what remains open, and the next step.

## Context compression

Compress only on **explicit user request** ("compress / fold / condense the `^^^`/`&&&` dialogue"). Approving a draft task counts as that request for that task file. A general "rewrite for clarity" is not.

- Fold each resolved `^^^`/`&&&` pair into the adjacent prose, removing the markers and the back-and-forth.
- Keep each decision next to the assertion it concerns; never move pairs into a detached Q&A section.
- Keep unresolved `^^^` remarks as-is, and the rest of the plan, findings, and context at their existing detail.

See `references/examples.md` for before/after examples.

## Archival

- **Task files:** `specdev task done <id>` once the user accepts the result, before the final PR commit (see "Approval and closing"). That is the only way a task file finishes.
- **ctx.md tasks:** archive only when all three hold: every Plan box is `[x]`; the titled task itself (not one step) is finished; and the user explicitly asks to archive, close, or reset. `specdev status`'s archive nudge means *ask*, not archive. Then move the task into `CHANGELOG.md` as a dated block at the top, move durable findings to `overview.md` and remaining work to `roadmap.md`, and reset `ctx.md` to the header.

## Before mutating a spec

Classify the change in one line before editing:

- **tick a step** — `[ ]` -> `[x]`; nothing else.
- **update findings/context/review** — decision-relevant state for the current work.
- **task state change** — use the `specdev task …` command, never an edit.
- **add a future idea** — to `ideas.md` or `roadmap.md`.
- **compress** — fold resolved `^^^`/`&&&` dialogue into prose.
- **archive** — per "Archival".

Ask first (never infer): renaming a task, archiving a `ctx.md` design task, editing `CHANGELOG.md` by hand, deleting findings, compressing. Routine `ctx.md` reconciliation after a state change needs no permission. Make the smallest edit: re-read the section, then patch it.

## Behavior

**Do:**

- Start with `overview.md`, `ctx.md`, and `specdev check`.
- Keep `ctx.md` concise and route other content home.
- Put one-commit work in task files and move them only with `specdev task` commands.
- Declare scope before touching a file; run the Acceptance commands yourself and tick only what passed.
- Run `specdev check` once per logical step and before handing work back.
- Record the user's acceptance from the conversation and close the task yourself.
- Preserve user wording, `^^^`/`&&&` pairs (until compression), and open questions.

**Do not:**

- Hand-edit task frontmatter, `## Log`, or `_index.md`; move or delete task files by hand.
- Work around a refusal or a `log-mismatch` by editing state.
- Tick Acceptance boxes you didn't verify, or Manual checks without the evidence suffix.
- Close a task without the user's acceptance, or park finished work as `blocked`.
- Create branches, commits, PRs, merges, or deploys unless the user asks.
- Rename, narrow, or archive the task without explicit intent; bulk-replace an active `ctx.md`.
- Delete findings or `^^^` remarks; relocate resolved pairs into a Q&A transcript.
- Put deferred items, roadmap pointers, gotchas, or architecture essays in `ctx.md`.
- Use Markdown tables or ASCII diagrams in specs.

## Gotchas

- `^^^`/`&&&` are for spec and task files only.
- `ctx.md` should represent reality; if it is stale, update it before starting.
- The no-task state (`ctx.md` header only, no active task) is valid.
- Only one task is in progress at a time; `advance` refuses a second. Tasks in approval don't block the next one, but close accepted tasks before moving on.
- `check`'s scope test reads uncommitted changes vs `HEAD`, minus the task's start baseline. Several tasks may share one commit; the user decides.
- `roadmap.md` is committed direction; `ideas.md` is not. Don't blur them.
- Other spec files can use whatever structure fits; the core workflow is stable.

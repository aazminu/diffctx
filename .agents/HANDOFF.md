# Handoff — 1.20.0 cycle, five lanes landed (2026-10-15)

## Objective

Close every issue in milestone 1.20.0. The work follows the approved plan in
the private audits repo, `diffctx/issues/PLAN_1.20.0_2026-10-14.md`. The cycle
runs under the coordinator session `nikolay-2f`:

- Report to it at every step boundary and before every push.
- The owner raised the weekly ceiling to 75%. The stop line is 74%, and the
  budget resets 2026-10-20 03:00 local.
- Work in cut-line order: A members, F, E1, then the B lanes by issue count.
- No release without the owner's explicit "yes". The epoch squash happens
  once, at the end of the cycle, right before the 1.20.0 release. The
  release needs that "yes".

## Verified state

**Commits on `main`, pushed to both remotes:**

| commit | what | gate (red on the base head, green after) |
|---|---|---|
| `550dfd6a` | #527: a recording line is answered from git's record; a search on the same line answers only when nothing landed | `a_commit_line_that_then_greps_its_log_is_still_reviewed` (red on 97d6bf57) |
| `f0d5a170` | #524: an older plugin pin says once per session that a newer version is installed beside it (`DIFFCTX_PLUGIN_ROOT` from the wrapper); every `hook.log` line ends with its version | `an_older_pin_says_once_per_session_that_a_newer_one_is_installed`; `test_a_session_on_an_older_plugin_hears_once_…` under /bin/bash 3.2 and bash 5 |
| `2609f714` | #529 diagnosis: Windows tests run with `RUST_BACKTRACE` and `PYTHONFAULTHANDLER`; on failure a pwsh step reruns `--lf` in one process and prints the 32-bit exit code | diagnostic only |
| `c5d59f74` | #529 diagnosis: WER LocalDumps for python.exe, the Application Error events printed on failure, minidumps uploaded as `crash-dumps-<os>-py<ver>` | diagnostic only |
| `9c56595b` | #532: a line whose last command fails (PostToolUseFailure) is reviewed like any other; `diffctx hook posttoolusefailure` | `test_a_commit_line_that_ends_in_a_failing_command_is_still_reviewed` (both bashes), manifest test (red on c5d59f74) |
| `8fcb63a8`, `22a201f2` | #510 Step 1: `git_faults.rs` and `test_concurrent_deadlines.py` assert outcomes, never elapsed time; the fake stalls are unbounded, a Fixture drop kills leftovers, and the Rust test jobs carry `timeout-minutes: 60` | mutations: `kill(-pgid)` removed → survivor check red; `token.cancel()` removed → "kept going" red |
| `4251ee7e` | #529: the Windows "worker crashed" was pytest-timeout's thread method `os._exit()`ing the xdist worker past the global 30 s; the global timeout, all 11 marks and the `pytest-timeout` dependency are gone, and the test job carries `timeout-minutes: 30` | `TestSuiteTiming` (red on d33ae6c6's pyproject); the mechanism reproduced on macOS with `--timeout-method=thread` |
| `d33ae6c6` | bitcheck: 30 cells, impact per range and three `--symbol` lookups | record then check on one build: all 30 identical |
| `ec819602` | #493: the header counts untested callers only beside a caller, labelled `caller(s) without a static test link` | assertion in `a_changed_definition_no_test_reaches_is_counted` (red on c5d59f74) |

- **Local results before the push:**
  - `cargo test --release --no-fail-fast`: 3557 passed, 0 failed, YAML
    corpus included.
  - pytest in one process: 992 passed, 4 skipped.
  - Plugin-hook and manifest tests after the final wrapper: 66 passed.
  - pre-commit: clean on every commit.
- **Tracker:**
  - Issues 527 and 524 are closed.
  - The 13 issues are in the milestone.
  - Root T was created as #530, and #328 was promoted to root C.
  - The close text of #505 and #507 re-homes #278 and #279.
  - The workstream map is posted on all 9 roots.
- **Stale worktrees** of session 74928e34 were removed. Its unreachable commits
  are in `~/backups/diffctx/worktrees-74928e34-2026-10-14.bundle`, and its
  uncommitted change is in `worktrees-74928e34-measure-dirty.patch` beside it.

## Step 0 re-verification (2026-10-15, PyPI 1.19.1 = the plugin pin)

Each issue carries a comment with its fixture and output.

- **Reproduces (15):** #389 #393 #420 #426 #427 #430 #432 #453 #459 #488
  #522 #523 #525 #526 #528.
- **Partly fixed, narrowed (4):** #504 (`<Ns.Member/>`, `memo(Comp)` left),
  #477 (method through an untyped helper return left), #503 (Python
  module-level binding left), #493 (header wording, fixed in `ec819602`).
- **Not reproduced (1):** #517 with conventional test paths; the external
  reporter was asked for the path.
- **Unchanged (1):** #320.
- **Measured:** #312, #352, #461 (counts on each issue, `blocked:measurement`
  removed).
- **New in the milestone:** #532 and #534 under D (#508), #533 under B
  (shell).

## Step 2 and A members (landed 2026-10-15)

| commit | what |
|---|---|
| `899ab4d1` | the dependency freeze (`toml`, `tree-sitter-sequel`, `proptest`); lanes never reopen Cargo.toml |
| `5c11ce42` | `bindings/` split into mod/python/js |
| `69cdd8c6` | the language registry: `src/lang/`, `LanguageFacts` with opaque `DefId` handles; impact.rs names no language type, a lane adds one file under `src/lang` |
| `a0388ad5` | proptest 1.12.0 |
| `4bfccaac` | #389: builtin and literal receivers are no caller |
| `1639964c` | #399: untyped reads are counted |
| `2e867319` | #427: embedded code in shell scripts is named |
| `c7d4ef02` | `unresolved: Option<Unknown>` and one `proves_empty` (E-class) |
| `31ae9ed6` | #393, #523: `named_by` lists the lines that run or ship a changed file by path |
| `f6b24f71` | #430: names too common to trace fold into one md line |
| `02a50893`, `5996e03f`, `cac8f723` | Renovate bumps (libc, python, sbom-action) applied on main; Forgejo PRs 58–60 close against them |
| `e289c38d` | two graph string wrappers only tests called, deleted |

Gates live in `crates/diffctx-native/tests/unknowns.rs`. Each commit body
records its differential (old vs new over real commits).

## Lanes

Each lane is a subagent in a scratch worktree on a local-only branch
`lane/<name>`. The owner rebases it onto `main`, reads its diff and its
diffctx impact, runs the full `cargo test --release` (corpus included) and
pytest on the merged tree, and pushes.

- Every lane follows the same rules:
  - one test file per lane;
  - each gate is red on the lane's base binary and green after;
  - decoys plus a fully-modelled control;
  - each flipped corpus case is read;
  - `CARGO_BUILD_JOBS=4`;
  - corpus runs one at a time, under a lock;
  - `cargo clean` once the lane is green.

**Merged and pushed (2026-10-15), up to `9255d0a4`.** Merged tree: cargo
3594/0, pytest 1067 passed / 4 skipped.

| lane | commits | closes | notes |
|---|---|---|---|
| E1 (#507) | `dac3dd51`, `19086745` | #438, #320 | one Rust `artifact::fit` for all three doors; #320's plugin-not-used half is #551 |
| F (#510) | `3153163e`, `8a2a64af`, `9ef5463c` | none | the remaining unbounded loops are listed on #510; #544 has a stack dump and a suspected fix |
| PY (#506 B) | `5cadec96`, `01cbfbed`, `f7534aa4`, `0066fb59` | #398, #477, #525 | Python halves of #420, #488, #426; #400 deferred (conflict with the #388 tests, on #400) |
| TS (#506 B) | `ea0ed217`, `0b96cdf3`, `bd12bee5` | #504, #357 | JS/TS uses off the syntax; Refs #503 #476 #372 (rest on #372); #461 #312 deferred to #530 |
| RS (#506 B) | `1745325f`, `09672128`, `cba68f36`, `9255d0a4` | #547, #552, #528, #545 | capped callers count; inline `#[test]`; trait rows fold; Refs #540 (narrowed) #484; #548 needs the Rust model |

`92b1506a` also removes the #529 Windows diagnostics, after three green
Windows runs.

No lane is running. `4881edc1` makes the fit contract's MCP door skip
where the mcp extra is absent (Windows on ARM).

## Remaining work, in plan order

The session stopped at 71% of the weekly budget (stop line 74%, reset
2026-10-20 03:00 local). Every proven change is pushed; nothing waits on a
local branch.

1. **Root A: the Coverage ledger.**
   `Coverage { modelled: (lang × relation), lost: Vec<Loss> }`, with:
   - every `continue`, filter, cap and failed read in the core loop pushing
     a Loss;
   - `limits`, `unanalysed_files` and `layout_only` rendered from the
     ledger;
   - `proves_empty` requiring `lost.is_empty()`.

   The proof is bitcheck 30/30 plus the corpus plus the differential. The
   remaining A members (#542, #521, #432, #550, the #424 footer) then add
   `Unknown` variants.
2. **The Rust binding model** (`src/lang/rust.rs` as a `LanguageFacts`):
   #484's remaining recall, #548, the rest of #540; see the comments there.
3. **The remaining B halves:**
   - JS: #420, #488, #426; the rest of #503, #476 and #372 (on each issue).
   - Swift/Java: #526, #335, #517.
   - SQL: #459.
   - Python #400, deferred: its conflict with the #388 tests is on #400.
4. **The /qa pass of 2026-10-15** stopped unfinished. Its protocol is in
   the private audits repo, at `diffctx/issues/QA_2026-10-15.md`. Still to
   do: issue intake over the whole queue, and the stumble probe.
5. **The other roots:**
   - Shell and hook: #521 #533 #534 #453 #549, root #508.
   - C: #328.
   - T: #530, with #461 #312 #417 deferred to it.
   - S: #512.
   - E2: #331 #415.
   - R: #513 #500.
   - F rest: the unbounded loops listed on #510.
   - #544: watch the corpus watchdog dump.
   - #551: the plugin is never reached on a review prompt.
6. **The epoch squash.** On 2026-10-15 the owner had everything after
   `e8707440` (the plugin lock `plugin-release` points at) folded into one
   epoch. The commit SHAs this file names resolve only from the
   `pre-epoch-sweep-2026-10-15-*` bundle under `~/backups/diffctx/`.
   Before the 1.20.0 release, the commits after that epoch fold into the
   next one, only with the owner's "yes". The coordinator's four
   conditions:
   - every step before it is green;
   - lease on the exact fetched tip per remote, and all three heads end on
     one SHA with an empty diff to the pre-squash tip;
   - protections restored on both remotes and diffed against the backup;
   - the bundle backup kept until CI on the squashed head is green.

**CI:** run 38779610514 (`diffctx CI` on `e8b5e4df`, the last code
commit) is green on every cell, and so is 38770819847 on `9255d0a4` with
all five lanes (#529's run four). The Windows cross-platform Rust cell took
59.1 and 59.9 minutes before `e8b5e4df` raised its guard to 90, then 51;
the minutes are on #510.

## Known problems

- The Rust impact's false zeros and false callers seen during this cycle
  are filed as #547, #548 and #552, and are in the RS lane.
- Pre-commit fails with "files were modified by this hook" if the tree is
  edited while it runs. Keep the tree still during a commit.
- The restart notice (#524) reaches the agent only on calls the wrapper hands
  to the binary: git, gh, or a search over a pending change.

## Do not redo

- The 1.20.0 plan and its review: two adversarial passes, the coordinator's
  edits, and the owner's approval.
- The tracker restructure: the roots, the map comments and the milestone moves.

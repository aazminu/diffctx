# Handoff — 1.19.1 bug series (2026-10-14)

## Objective

Ship 1.19.1 with every open bug carrying a verdict. The series below is on
`main`. The release
has not been dispatched: it waits for the owner's explicit "yes".

## Verified state

| commit | what | production delta | tests + docs |
|---|---|---|---|
| `1bd80d66` | plugin `pyproject.toml` keeps the pin's full X.Y.Z under pyproject-fmt (main's pre-commit job was red since 1.19.0) | +4/−0 | — |
| `f57d4b9c` | 26 bugs, each with a gate that fails on `23a36c4a` and passes after: `Fixes` #318 #391 #401 #402 #410 #411 #413 #425 #428 #439 #440 #444 #449 #452 #456 #463 #472 #473 #474 #479 #481 #486 #496 #497 #519 #520; `Refs` #484 #521 | +809/−133 | +1314/−29 |
| `c46fd079` | Go callers from syntax facts (`go_bindings.rs`), root #506; no filed bug depends on it | +646/−20 | +85/−0 |

Before the series:

- State B (tree after `f57d4b9c`): full `cargo test --release` 3554 passed, 0
  failed (YAML corpus 2943/2943); `pre-commit run --all-files` clean.
- State C (tree after `c46fd079`, the full state): 3555 passed, 0 failed; pytest
  990 passed / 4 skipped; pre-commit clean.

## Remaining work

1. **CI, not yet checked.** Look at the GitHub `diffctx CI` run for `c46fd079`
   (`gh run list -R nikolay-e/diffctx --commit
   c46fd079806370ba87bc6745b1e31a4b7a6877ac`).
   GitHub runs only the pushed head, so B is covered by the run on C.
2. **Ledger.** The verdict for each of the 75 milestone bugs lives in the
   session scratchpad as
   `ledger_1191.md`. Copy it into the private audits repo as
   `diffctx/issues/LEDGER_1.19.1_2026-10-14.md`, commit there, and push. It
   records 26 fixed,
   13 closed with evidence, 2 partial, 7 roots, and 27 deferred (each with its
   mechanism).
3. **Close-out comments.**
   - #318: the impact half is fixed; the pack half is #331's class.
   - #484: what landed and what remains (recall of files over the 100 KB cap).
   - #521: what landed; the bash grammar failure is upstream.
   - Confirm that the 26 `Fixes` issues auto-closed on GitHub.
4. **Release 1.19.1, only on the owner's "yes".**
   1. Epoch-squash the commits after the 1.19.0 release into one epoch. No
      session trailer.
      Back up a bundle first. Lift branch protection on both remotes and restore
      it afterwards.
      Re-pin the bitcheck tags in the same sweep.
   2. Dispatch `cd.yml`.
   3. Verify PyPI, npm, crates, ghcr, Docker Hub, the MCP registry and the
      GitHub release assets.
   4. Update the `diffctx@diffctx` plugin and check, under `/bin/bash`, the MCP
      `serverInfo`
      version and one PreToolUse and one PostToolUse hook run.
   5. `cargo clean` the stale profiles in `target/`.
5. **Leftovers.**
   - The 1.18.4 plugin cache directory and the uv cache: clear both once no live
     session runs
     1.18.4 (`uv cache clean diffctx`).
   - `diffctx@synced`, still at 1.18.2, comes from the claude.ai account; only
     the owner can
     update or remove it.
   - A dirty-tree build is installed as a global uv tool (`diffctx`,
     `diffctx-mcp`). After the
     release, remove it (`uv tool uninstall diffctx`) so the plugin is again the
     only install.

## Known problems

- Deferred bugs with their starting points are in the ledger. The largest are
  #331 (pack/locate
  run no reverse discovery), #398 (Python module-level assignments are not
  definitions), and
  #484 (files over the size cap are named but not read).
- About #398: turning module assignments into fragments flipped the corpus case
  `builders/python_import_module_path` and moved 447 fragment hashes. It needs
  an oracle decision.

## Do not redo

- The 26 fixes and their gates.
- The re-verification of the 63 pre-existing bugs (scout fixtures are in the
  session scratchpad).
- The global install sweep across every surface.

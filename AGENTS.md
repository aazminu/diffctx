# diffctx

## Ultimate Goal

**Maximize the speed and depth of understanding textual information — for any
reader, in any scenario.**

Whether the consumer is an LLM filling a context window or a human reviewing a
change, the job is to extract the maximum signal from a codebase in the densest
clear form. The single lens for every trade-off is **comprehension-per-token**:
understanding gained per unit of attention spent. That is why Markdown is the
default output — ~7% cheaper than YAML on real diffs (#104).

The theory behind diff-context selection is the paper
([source in `paper/v2/`, snapshot at tag `paper-v2`](https://doi.org/10.5281/zenodo.18824579)).
Setup, suite shape and the per-case YAML corpus gate
(`crates/diffctx-native/tests/known_below_threshold.txt`, bidirectional: a
listed case that starts passing also fails) are in `CONTRIBUTING.md`.

## Performance changes: E-class or Q-class

Every change is **E-class** (identical selection output on identical input) or
**Q-class** (output-changing). Q-class changes are frozen during an evaluation
cycle (calibration → validation → sweep), because they invalidate the
calibration and force a full rerun. The shared token corpus, the per-blob token
cache and the two-pass edge cap were E-class; fragmentation and scoring changes
are Q-class.

- **Per commit:** `scripts/bitcheck.sh record` on the pre-change build, `check`
  after — 24 cells against a worktree pinned at a fixed SHA, so editing the code
  cannot edit the input. It drives the native binary and says nothing about
  `writer.py`, `_native/` or `mcp/`; a Python-side E-class claim needs its own
  byte-diff through the installed wheel. The full corpus
  (`cargo test --release --test yaml_cases`) is the only net over the
  per-language edge and parser tables bitcheck never reaches.
- **Per release:** `python -m eval equivalence --a <old-run> --b <new-run>` on
  the stratified 40-instance sample (`results/sweep_v2_local/equiv/manifests`)
  plus a double-run determinism check. It needs two full eval outputs and a
  local `results/` tree, so no CI job runs it.

## Reading test and oracle results

- **Garbage injection is opt-in.** Only the few YAML cases that set
  `fixtures.auto_garbage` get unrelated `GARBAGE_*` files and the leak assertion
  in `yaml_cases.rs`. The relevance filter is actually guarded by the Python
  corpus in `tests/garbage_data.py`, injected by `test_mcp.py` and
  `test_e2e_cli_scenarios.py`. Turning the flag on broadly is an unpulled lever,
  not existing coverage.
- `changed_files` in a case YAML is the **final state of every file**, not the
  diff; a path listed there is not evidence the tool must emit it.
- `forbidden_rate` saturates: one wrong file trips its path entry plus several
  anchor-only entries and drives `score = recall * (1 - forbidden_rate)` to
  zero. 100% means "at least one wrong file" (median: one extra file), not
  "emitted all the garbage".
- The harness passes `DEFAULT_PPR_ALPHA` and an unspecified tau (`None`) so the
  gate measures the shipped configuration — it once passed `tau=0.0`, which hid
  119 cases in both directions (#175). Never replace either with a literal or
  with the tau constant: an unspecified tau resolves per scorer (the ungated
  ones ship at a raised threshold), so naming the constant pins them wrong.

# Decisions

## 2026-10-14 — The hook reviews what git recorded (#508)

**Decision.** The plugin hook answers a commit, merge, cherry-pick, revert
or push in `PostToolUse`, from git's own record: `PreToolUse` stores the refs
of each repository the line names, and once the line ran the hook reads the
range the reflog shows moved (`HEAD`'s last entry; a remote-tracking ref's
"update by push"). Nothing about what a line will stage is parsed any more.
The opt-in strict gate is the one review before git runs, and it reviews only
what needs no prediction: the index a plain `git commit` records. A commit
line that stages for itself (an add on the same line, paths, `-a`, `-i`,
`--pathspec-from-file`) is denied with the instruction to stage as its own
command.

**Reason.** The planner replayed a line's staging against a copy of the
index, read from the command text. Fifteen parser commits in nine days
(members #405, #422, #429, #436 and #501 among them) did not close the
class, and `--pathspec-from-file` reviewed another session's staged file
instead of the paths the commit took. Claude Code delivers a `PreToolUse`
note together with the tool's result anyway (#429), so the preview bought no
earlier reading, only a chance to be wrong.

**Rejected.** A git `pre-commit` hook in the reviewed repository: it would
see the exact index, but installing hooks into users' repositories is out of
the plugin's scope and the agent never reads its output. Keeping the planner
and patching each shape: the corpus shows every patch followed by a sibling.

**Consequences.** Without the gate, the impact arrives after the commit
landed: a caller it names is fixed in a follow-up commit, not before. A push
of several refs reviews the first and counts the rest. A line whose shell
cannot run it (bash 3.2 on a heredoc inside `$(…)`) records nothing and gets
nothing. The command parser remains for the gate, for `cd`/`-C` resolution
and for inspections and searches, where no record exists to read.

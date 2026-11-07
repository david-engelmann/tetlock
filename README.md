# tetlock

Before anything irreversible runs, the agent states in writing what it believes is about to happen. Then reality grades the sentence.

## What this is

Approvals in agent tools ask the wrong question. They show you a command and ask whether it may run. Nobody can hold a command's consequences in their head at a glance. Tetlock asks for a confession instead. Before a destructive call executes, the agent states in one plain sentence what it believes is about to happen, and the operator approves the sentence. Later, the outcome is graded against what actually happened. Tetlock grades predictions, onrecord keeps memories.

Confessions are onrecord claims with due dates, living in the record file and nowhere else. The honesty ledger, the weekly wall, and the tally are views computed from the record on every run, never a second store. The ledger lists every confession as graded, awaiting, overdue, or superseded. The wall renders one ISO week: the confessions that broke that week, each with its sentence and what actually happened, and the week's overdue ungraded confessions as their own list. The tally counts the whole record at a stated time: the well-formed confessions by state and by grade, per action, every count with its denominator in the document and the arithmetic closed. In Round 1 the grade is supplied by the grader; classifying what kind of wrong a false confession was is a later round's work, and nothing here guesses at it.

## Status

Round 1 is closed. Confessions, outcomes, the
ledger, and the wall.

Round 2 is closed. The tally view, and the malformed
test made whole: a confession claim is now checked
against all of WW-5, sentence rule included, in every
view. Setup landed earlier in October 2026.

Round 3 is closed. The tool is a Rust library crate
with a thin binary now, reproducing Rounds 1 and 2
behavior exactly. The record layer is consumed from
the onrecord crate at a pinned revision, the line's
first cross-repo crate consumption, instead of a
local copy, and the date drift the copy had kept
against the record's own contract is retired with
it. The Python leaves the tree in this round.

## Programme status

- Closed perfection rounds, 7 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, WW-1 to WW-12 as amended, met for the Round 1 scope.
  WW-13 to WW-16 with amendment WW-8c, written ahead of
  the code and met in Round 2. WW-17 to WW-23, written
  ahead of the code and met in Round 3.
- Local gate, python3 scripts/codepoint_scan.py and
  cargo test, with cargo clippy --all-targets -- -D warnings
  and cargo fmt --check. CI runs the same commands.

## Licence

MIT. See LICENSE.

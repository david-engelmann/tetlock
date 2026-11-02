# wallofwrong

Before anything irreversible runs, the agent has to say what it believes is about to happen. Then reality grades the sentence.

## What this is

Approvals in agent tools ask the wrong question. They show you a command and ask whether it may run. Nobody can hold a command's consequences in their head at a glance. Wallofwrong asks for a confession instead. Before a destructive call executes, the agent states in one plain sentence what it believes is about to happen, and the operator approves the sentence. Later, the outcome is graded against what actually happened. Wallofwrong grades predictions, onrecord keeps memories.

Confessions are onrecord claims with due dates, living in the record file and nowhere else. The honesty ledger and the weekly Wall of Wrong are views computed from the record on every run, never a second store. The ledger lists every confession as graded, awaiting, overdue, or superseded. The wall renders one ISO week: the confessions that broke that week, each with its sentence and what actually happened, and the week's overdue ungraded confessions as their own list. In Round 1 the grade is supplied by the grader; classifying what kind of wrong a false confession was is a later round's work, and nothing here guesses at it.

## Status

Round 1 is built and in review on its branch. Setup landed earlier in October 2026.

## Programme status

- Closed perfection rounds, 1 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, WW-1 to WW-12 as amended, met for the Round 1 scope.
- Local gate, python3 scripts/codepoint_scan.py and
  python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.

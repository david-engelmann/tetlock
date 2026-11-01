# wallofwrong

Before anything irreversible runs, the agent has to say what it believes is about to happen. Then reality grades the sentence.

## What this is

Approvals in agent tools ask the wrong question. They show you a command and ask whether it may run. Nobody can hold a command's consequences in their head at a glance. Wallofwrong asks for a confession instead. Before a destructive call executes, the agent states in one plain sentence what it believes is about to happen, and you approve the sentence. The tool then watches what actually happens and grades the claim. Every confident sentence that turned out false goes into an honesty ledger, and once a week the worst of them are built into the Wall of Wrong, each one paired with a drafted rule that would have caught it.

## Status

Seeded in October 2026. Local setup is in progress on the setup
branch. The remote seeding pass is waiting on a token access fix, so
work is banked locally until pushes clear. This repo is in phase 3
of the Retrospective Line and its build waits on the phase gate, the
main chat confirmation that the Gate Truth evidence pack schema has
merged and passed acceptance. Design work may proceed, build does
not start before the gate.

## Programme status

- Closed perfection rounds, 0 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, stub in place, not yet met.
- Local gate, python3 scripts/codepoint_scan.py and, once tests
  exist, python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.

# tetlock

Before anything irreversible runs, the agent states in writing what it believes is about to happen. Then reality grades the sentence.

Approvals in agent tools ask the wrong question. They show a command and ask whether it may run, and nobody can hold a command's consequences in their head at a glance. Tetlock asks for a confession instead. Before a destructive call runs, the agent states in one plain sentence what it believes is about to happen, and the operator approves the sentence. Later the outcome is graded against what actually happened. Tetlock grades predictions, onrecord keeps memories. A confession is an onrecord claim with a due date, and the ledger, the wall, and the tally are views computed from that record on every run, never a second store.

![ci](https://github.com/david-engelmann/tetlock/actions/workflows/ci.yml/badge.svg)

```text
{
  "total": 2,
  "by_state": {
    "graded": 1,
    "awaiting": 1,
    "overdue": 0,
    "superseded": 0
  },
  "graded": 1,
  "by_grade": {
    "held": 0,
    "broke": 1,
    "unverifiable": 0
  },
  "actions": [
    {
      "action": "deploy-staging",
      "graded": 1,
      "held": 0,
      "broke": 1,
      "unverifiable": 0
    }
  ],
  "malformed": 0
}
```

## Install

The install needs Git and a Rust toolchain.

```sh
git clone https://github.com/david-engelmann/tetlock.git
cd tetlock
export CARGO_HOME="$PWD/.cargo-home"
export PATH="$CARGO_HOME/bin:$PATH"
cargo install --path .
```

## Quickstart

Run a view over the sample record that ships in the test fixtures. The record holds two confessions, the outcome that graded one of them, and one plain claim, and the tally counts the record at a stated time.

```sh
cp tests/fixtures/record_synthetic.jsonl record.jsonl
tetlock tally --now 2026-01-09T00:00:00Z record.jsonl
```

```text
{
  "total": 2,
  "by_state": {
    "graded": 1,
    "awaiting": 1,
    "overdue": 0,
    "superseded": 0
  },
  "graded": 1,
  "by_grade": {
    "held": 0,
    "broke": 1,
    "unverifiable": 0
  },
  "actions": [
    {
      "action": "deploy-staging",
      "graded": 1,
      "held": 0,
      "broke": 1,
      "unverifiable": 0
    }
  ],
  "malformed": 0
}
```

## Usage

Two confessions go into a fresh record. The first command creates the record file, and each confession prints the id of the claim it wrote.

```sh
tetlock confess record.jsonl --action deploy --sentence "The staging deploy finishes without downtime." --due 2026-06-04 --at 2026-06-01T09:00:00Z --asserter agent --approval "ticket 41"
tetlock confess record.jsonl --action migrate --sentence "The legacy table drops cleanly in one pass." --due 2026-06-05 --at 2026-06-01T09:05:00Z --asserter agent --approval "ticket 42"
```

```text
claim-239fdfa797bd1d2f
claim-e1d9ba7b5df0e427
```

The first confession is graded once its outcome is known. The grade names what happened, the evidence points at the run that showed it, and the command prints the id of the outcome claim it wrote.

```sh
tetlock grade record.jsonl --confession claim-239fdfa797bd1d2f --grade broke --happened "The deploy stopped for twelve minutes while the migration ran." --at 2026-06-03T15:00:00Z --asserter operator --evidence trace:run-31
```

```text
claim-4f8d9685876dede9
```

The ledger lists every confession in the record with its state at a stated time. One confession stands graded and the other stands overdue.

```sh
tetlock ledger --now 2026-06-06T00:00:00Z record.jsonl
```

```text
{
  "confessions": [
    {
      "id": "claim-239fdfa797bd1d2f",
      "action": "deploy",
      "sentence": "The staging deploy finishes without downtime.",
      "due": "2026-06-04",
      "grade": "broke",
      "state": "graded"
    },
    {
      "id": "claim-e1d9ba7b5df0e427",
      "action": "migrate",
      "sentence": "The legacy table drops cleanly in one pass.",
      "due": "2026-06-05",
      "grade": null,
      "state": "overdue"
    }
  ]
}
```

The wall renders one ISO week, the confessions that broke that week and the week's ungraded confessions.

```sh
tetlock wall --date 2026-06-03 record.jsonl
```

```text
{
  "week": "2026-W23",
  "broke": [
    {
      "action": "deploy",
      "sentence": "The staging deploy finishes without downtime.",
      "happened": "The deploy stopped for twelve minutes while the migration ran.",
      "outcome": "claim-4f8d9685876dede9"
    }
  ],
  "ungraded": [
    {
      "action": "migrate",
      "sentence": "The legacy table drops cleanly in one pass.",
      "due": "2026-06-05"
    }
  ]
}
```

The tally counts the whole record at a stated time, every count with its denominator in the document.

```sh
tetlock tally --now 2026-06-06T00:00:00Z record.jsonl
```

```text
{
  "total": 2,
  "by_state": {
    "graded": 1,
    "awaiting": 0,
    "overdue": 1,
    "superseded": 0
  },
  "graded": 1,
  "by_grade": {
    "held": 0,
    "broke": 1,
    "unverifiable": 0
  },
  "actions": [
    {
      "action": "deploy",
      "graded": 1,
      "held": 0,
      "broke": 1,
      "unverifiable": 0
    }
  ],
  "malformed": 0
}
```

## Status

All five commands are built and run as a Rust binary, with the Python of the first two rounds standing as the behavior reference the port is pinned against. The record layer is consumed from the onrecord crate at a pinned revision, and the tape values the views read come from the flighttape crate at a pinned revision. The grade on an outcome is supplied by the grader. Classifying what kind of wrong a false confession was is not built, and nothing here guesses at it.

Closed perfection rounds, 7 of 100. A round counts only when a full cycle of work, audit, review, red team, and approval has closed.

## License

MIT. See LICENSE.

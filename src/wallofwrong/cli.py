"""WW-12. A thin command line: confess, grade, ledger,
wall. Failures are named errors with exit 1."""
from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from pathlib import Path

from .record import Store, StoreError
from .wall import confess, grade, ledger, wall


def _parse_evidence(items) -> list[dict]:
    evidence = []
    for item in items or []:
        source, sep, reference = item.partition(":")
        if not sep or not source or not reference:
            raise StoreError(
                f"evidence must name a source and a reference, "
                f"{item}")
        evidence.append({"source": source, "reference": reference})
    return evidence


def _write_atomically(path: Path, text: str) -> None:
    fd, tmp = tempfile.mkstemp(dir=str(path.parent),
                               prefix=path.name + ".")
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.replace(tmp, path)
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def _emit(document, out) -> int:
    text = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    try:
        if out is None:
            sys.stdout.write(text)
            sys.stdout.flush()
        else:
            _write_atomically(Path(out), text)
    except (OSError, UnicodeEncodeError):
        if out is not None:
            print(f"error, cannot write the view, {out}",
                  file=sys.stderr)
        else:
            print("error, cannot write to stdout", file=sys.stderr)
            try:
                sys.stdout.close()
            except OSError:
                pass
            sys.stdout = open(os.devnull, "w", encoding="utf-8")
        return 1
    return 0


def _emit_id(claim_id: str) -> int:
    try:
        sys.stdout.write(claim_id + "\n")
        sys.stdout.flush()
    except (OSError, UnicodeEncodeError):
        print("error, cannot write to stdout", file=sys.stderr)
        try:
            sys.stdout.close()
        except OSError:
            pass
        sys.stdout = open(os.devnull, "w", encoding="utf-8")
        return 1
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="wallofwrong")
    sub = parser.add_subparsers(dest="command", required=True)
    cf = sub.add_parser("confess", help="append a confession")
    cf.add_argument("record")
    cf.add_argument("--action", required=True)
    cf.add_argument("--sentence", required=True)
    cf.add_argument("--due", required=True)
    cf.add_argument("--at", required=True)
    cf.add_argument("--asserter", required=True)
    cf.add_argument("--approval", required=True)
    gr = sub.add_parser("grade", help="append an outcome")
    gr.add_argument("record")
    gr.add_argument("--confession", required=True)
    gr.add_argument("--grade", required=True)
    gr.add_argument("--happened", required=True)
    gr.add_argument("--at", required=True)
    gr.add_argument("--asserter", required=True)
    gr.add_argument("--evidence", action="append")
    gr.add_argument("--supersedes")
    led = sub.add_parser("ledger", help="the ledger view")
    led.add_argument("record")
    led.add_argument("--now", required=True)
    led.add_argument("-o", "--output")
    wal = sub.add_parser("wall", help="the wall view")
    wal.add_argument("record")
    wal.add_argument("--date", required=True)
    wal.add_argument("-o", "--output")
    args = parser.parse_args(argv)
    store = Store(Path(args.record))
    try:
        if args.command == "confess":
            return _emit_id(confess(
                store, action=args.action, sentence=args.sentence,
                due=args.due, at=args.at, asserter=args.asserter,
                approval=args.approval))
        if args.command == "grade":
            return _emit_id(grade(
                store, confession_id=args.confession,
                grade_value=args.grade, happened=args.happened,
                at=args.at, asserter=args.asserter,
                evidence=_parse_evidence(args.evidence),
                supersedes=args.supersedes))
        if args.command == "ledger":
            return _emit(ledger(store, now=args.now), args.output)
        if args.command == "wall":
            return _emit(wall(store, on=args.date), args.output)
    except StoreError as exc:
        print(f"error, {exc}", file=sys.stderr)
        return 1
    except OSError as exc:
        print(f"error, {exc}", file=sys.stderr)
        return 1
    return 1


if __name__ == "__main__":
    raise SystemExit(main())

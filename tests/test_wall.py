"""WW-5 to WW-12. Confessions, outcomes, and the views."""
import json
import shutil
from pathlib import Path

import pytest

from wallofwrong.cli import main
from wallofwrong.digest import digest_of
from wallofwrong.record import Store, StoreError
from wallofwrong.wall import confess, grade, ledger, wall

FIX = Path(__file__).parent / "fixtures"
GOLDEN = "cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3"
CONF_A = "claim-b0e7464b9254d2cb"
CONF_B = "claim-1008d801474cbae0"


def fresh_record(tmp_path):
    target = tmp_path / "record.jsonl"
    shutil.copy(FIX / "record_synthetic.jsonl", target)
    return Store(target)


def test_the_digest_is_pinned_to_the_family_golden():
    tape = json.loads((FIX / "tape_synthetic.json").read_text())
    assert digest_of(tape) == GOLDEN


def test_the_onrecord_minted_fixture_loads_sound(tmp_path):
    store = fresh_record(tmp_path)
    assert store.check() == []
    claims = store._load()
    assert [c["id"] for c in claims] == [
        CONF_A, CONF_B, "claim-2df18080a5b05ac3",
        "claim-1fddf8798a175d0d"]


def test_the_sentence_rule_is_mechanical(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    base = {"action": "deploy", "due": "2026-02-01",
            "at": "2026-01-05T09:00:00Z", "asserter": "agent",
            "approval": "op-1"}
    with pytest.raises(StoreError, match="one line"):
        confess(store, sentence="Two\nlines.", **base)
    with pytest.raises(StoreError, match="end in a period"):
        confess(store, sentence="No terminator", **base)
    with pytest.raises(StoreError, match="internal sentence break"):
        confess(store, sentence="First. Then the rest.", **base)
    with pytest.raises(StoreError, match="internal sentence break"):
        confess(store, sentence="Ships version 1.5 today.", **base)
    cid = confess(store, sentence="The deploy finishes cleanly.",
                  **base)
    assert cid.startswith("claim-")


def test_ledger_states_from_the_fixture(tmp_path):
    store = fresh_record(tmp_path)
    document = ledger(store, now="2026-01-15T00:00:00Z")
    states = {c["action"]: (c["state"], c["grade"])
              for c in document["confessions"]}
    assert states == {"deploy-staging": ("graded", "broke"),
                      "drop-table": ("awaiting", None)}
    later = ledger(store, now="2026-01-25T00:00:00Z")
    states = {c["action"]: c["state"] for c in later["confessions"]}
    assert states["drop-table"] == "overdue"


def test_the_wall_for_the_fixtures_week(tmp_path):
    store = fresh_record(tmp_path)
    document = wall(store, on="2026-01-07")
    assert document["week"] == "2026-W02"
    assert document["broke"] == [{
        "action": "deploy-staging",
        "sentence": "The staging deploy finishes without downtime.",
        "happened": "The deploy rolled back after nine minutes down.",
        "outcome": "claim-2df18080a5b05ac3"}]
    assert document["ungraded"] == []
    week3 = wall(store, on="2026-01-20")
    assert week3["week"] == "2026-W04"
    assert week3["broke"] == []
    assert week3["ungraded"] == [{
        "action": "drop-table",
        "sentence": "The legacy table drops cleanly in one pass.",
        "due": "2026-01-20"}]


def test_grading_once_and_corrections_supersede(tmp_path):
    store = fresh_record(tmp_path)
    evidence = [{"source": "trace", "reference": "run-9"}]
    outcome = grade(store, confession_id=CONF_B, grade_value="held",
                    happened="The table dropped in one pass.",
                    at="2026-01-19T09:00:00Z", asserter="grader",
                    evidence=evidence)
    with pytest.raises(StoreError, match="already graded"):
        grade(store, confession_id=CONF_B, grade_value="broke",
              happened="Actually it failed.",
              at="2026-01-19T10:00:00Z", asserter="grader",
              evidence=evidence)
    grade(store, confession_id=CONF_B, grade_value="broke",
          happened="Actually it failed.",
          at="2026-01-19T11:00:00Z", asserter="grader",
          evidence=evidence, supersedes=outcome)
    document = ledger(store, now="2026-01-19T12:00:00Z")
    grades = {c["action"]: c["grade"] for c in document["confessions"]}
    assert grades["drop-table"] == "broke"


def test_grading_unknown_or_superseded_confessions_is_refused(tmp_path):
    store = fresh_record(tmp_path)
    evidence = [{"source": "trace", "reference": "run-9"}]
    with pytest.raises(StoreError, match="no confession"):
        grade(store, confession_id="claim-ghost",
              grade_value="held", happened="x.",
              at="2026-01-19T09:00:00Z", asserter="g",
              evidence=evidence)
    store.add(text="The plan changed and the deploy is off.",
              asserter="operator", asserted_at="2026-01-06T10:00:00Z",
              evidence=[{"source": "note", "reference": "n-1"}],
              supersedes=CONF_A)
    with pytest.raises(StoreError, match="cannot be graded"):
        grade(store, confession_id=CONF_A, grade_value="held",
              happened="x.", at="2026-01-19T09:00:00Z",
              asserter="g", evidence=evidence)
    document = ledger(store, now="2026-01-19T12:00:00Z")
    states = {c["action"]: c["state"] for c in document["confessions"]}
    assert states["deploy-staging"] == "superseded"


def test_views_are_deterministic(tmp_path):
    store = fresh_record(tmp_path)
    assert ledger(store, now="2026-01-15T00:00:00Z") == \
        ledger(store, now="2026-01-15T00:00:00Z")
    first = wall(store, on="2026-01-07")
    assert first == wall(store, on="2026-01-08")


def test_a_defective_record_is_refused_with_defects_named(tmp_path):
    target = tmp_path / "record.jsonl"
    shutil.copy(FIX / "record_synthetic.jsonl", target)
    with open(target, "a", encoding="utf-8") as handle:
        handle.write('{"id": "claim-forged", "text": "x"}\n')
    with pytest.raises(StoreError, match="defects"):
        ledger(Store(target), now="2026-01-15T00:00:00Z")


def test_the_cli_round_trip(tmp_path, capsys):
    path = str(tmp_path / "r.jsonl")
    assert main(["confess", path, "--action", "deploy",
                 "--sentence", "The deploy finishes cleanly.",
                 "--due", "2026-02-01", "--at", "2026-01-05T09:00:00Z",
                 "--asserter", "agent", "--approval", "op-1"]) == 0
    cid = capsys.readouterr().out.strip()
    assert main(["grade", path, "--confession", cid,
                 "--grade", "broke", "--happened", "It rolled back.",
                 "--at", "2026-01-30T09:00:00Z", "--asserter", "g",
                 "--evidence", "trace:run-2"]) == 0
    capsys.readouterr()
    assert main(["wall", path, "--date", "2026-01-30"]) == 0
    document = json.loads(capsys.readouterr().out)
    assert document["broke"][0]["action"] == "deploy"
    assert main(["confess", path, "--action", "x",
                 "--sentence", "Two. Sentences.",
                 "--due", "2026-02-01", "--at", "2026-01-05T09:00:00Z",
                 "--asserter", "agent", "--approval", "op-1"]) == 1
    assert "internal sentence break" in capsys.readouterr().err

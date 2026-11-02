"""WW-5 to WW-16. Confessions, outcomes, and the views."""
import json
import shutil
from pathlib import Path

import pytest

from wallofwrong.cli import main
from wallofwrong.digest import digest_of
from wallofwrong.record import Store, StoreError
from wallofwrong.wall import (confess, grade, ledger,
                              tally, wall)

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
    store.add(text="The plan changed and the migration is off.",
              asserter="operator", asserted_at="2026-01-06T10:00:00Z",
              evidence=[{"source": "note", "reference": "n-1"}],
              supersedes=CONF_B)
    with pytest.raises(StoreError, match="cannot be graded"):
        grade(store, confession_id=CONF_B, grade_value="held",
              happened="x.", at="2026-01-19T09:00:00Z",
              asserter="g", evidence=evidence)
    document = ledger(store, now="2026-01-19T12:00:00Z")
    states = {c["action"]: c["state"] for c in document["confessions"]}
    assert states["drop-table"] == "superseded"


def test_a_graded_confession_keeps_its_grade_when_superseded(tmp_path):
    store = fresh_record(tmp_path)
    store.add(text="The deploy plan was replaced wholesale.",
              asserter="operator", asserted_at="2026-01-08T10:00:00Z",
              evidence=[{"source": "note", "reference": "n-2"}],
              supersedes=CONF_A)
    document = ledger(store, now="2026-01-19T12:00:00Z")
    entries = {c["action"]: c for c in document["confessions"]}
    assert entries["deploy-staging"]["state"] == "graded"
    assert entries["deploy-staging"]["grade"] == "broke"
    walled = wall(store, on="2026-01-07")
    assert [b["action"] for b in walled["broke"]] == ["deploy-staging"]


def test_an_outcome_superseded_by_a_plain_claim_still_governs(tmp_path):
    store = fresh_record(tmp_path)
    store.add(text="A remark about the deploy outcome.",
              asserter="operator", asserted_at="2026-01-08T09:00:00Z",
              evidence=[{"source": "note", "reference": "n-3"}],
              supersedes="claim-2df18080a5b05ac3")
    document = ledger(store, now="2026-01-15T00:00:00Z")
    entries = {c["action"]: c for c in document["confessions"]}
    assert entries["deploy-staging"]["grade"] == "broke"


def test_two_live_outcomes_refuse_the_view_by_name(tmp_path):
    store = fresh_record(tmp_path)
    store.add(text="It held after all.",
              asserter="second-grader",
              asserted_at="2026-01-08T09:00:00Z",
              evidence=[{"source": "trace", "reference": "run-40"}],
              subject=f"confession:{CONF_A}",
              extra={"kind": "outcome", "grade": "held"})
    with pytest.raises(StoreError, match="two live outcomes"):
        ledger(store, now="2026-01-15T00:00:00Z")


def test_a_malformed_confession_is_named_and_never_walled(tmp_path):
    store = fresh_record(tmp_path)
    forged = store.add(
        text="The cache clears itself nightly.",
        asserter="mallory", asserted_at="2026-01-05T09:00:00Z",
        evidence=[{"source": "hunch", "reference": "none"}],
        subject="cache-clear", due="2026-01-06",
        extra={"kind": "confession"})
    document = ledger(store, now="2026-01-15T00:00:00Z")
    assert [c["action"] for c in document["confessions"]] == [
        "deploy-staging", "drop-table", "cache-clear"]
    assert document["malformed"] == [
        {"id": forged, "reasons": ["no approval evidence"]}]
    walled = wall(store, on="2026-01-06")
    assert walled["ungraded"] == []
    with pytest.raises(StoreError, match="not a confession"):
        grade(store, confession_id=forged, grade_value="broke",
              happened="It did not.",
              at="2026-01-07T09:00:00Z", asserter="g",
              evidence=[{"source": "trace", "reference": "r-1"}])


def test_week_boundaries_and_utc_reading(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    cid = confess(store, action="deploy",
                  sentence="The Sunday deploy holds.",
                  due="2026-01-11", at="2026-01-04T09:00:00Z",
                  asserter="agent", approval="op-7")
    grade(store, confession_id=cid, grade_value="broke",
          happened="It failed late Sunday.",
          at="2026-01-11T23:30:00Z", asserter="g",
          evidence=[{"source": "trace", "reference": "r-5"}])
    assert wall(store, on="2026-01-11")["broke"]
    assert wall(store, on="2026-01-12")["broke"] == []
    # The same instant written with a +14:00 offset reads
    # in UTC and stays in the same week.
    store2 = Store(tmp_path / "r2.jsonl")
    cid2 = confess(store2, action="deploy",
                   sentence="The Sunday deploy holds.",
                   due="2026-01-11", at="2026-01-04T09:00:00Z",
                   asserter="agent", approval="op-7")
    grade(store2, confession_id=cid2, grade_value="broke",
          happened="It failed late Sunday.",
          at="2026-01-12T13:30:00+14:00", asserter="g",
          evidence=[{"source": "trace", "reference": "r-5"}])
    assert wall(store2, on="2026-01-11")["broke"]


def test_held_confessions_never_reach_the_wall(tmp_path):
    store = fresh_record(tmp_path)
    grade(store, confession_id=CONF_B, grade_value="held",
          happened="The table dropped in one pass.",
          at="2026-01-19T09:00:00Z", asserter="grader",
          evidence=[{"source": "trace", "reference": "run-9"}])
    document = wall(store, on="2026-01-20")
    assert document["broke"] == []
    assert document["ungraded"] == []


def test_a_view_never_overwrites_the_record(tmp_path, capsys):
    target = tmp_path / "record.jsonl"
    shutil.copy(FIX / "record_synthetic.jsonl", target)
    before = target.read_bytes()
    assert main(["wall", str(target), "--date", "2026-01-07",
                 "-o", str(target)]) == 1
    assert "never overwrites the record" in capsys.readouterr().err
    assert target.read_bytes() == before


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


NOW = "2026-03-01T00:00:00Z"
EVIDENCE = [{"source": "trace", "reference": "run-9"}]


def _confess(store, action, sentence, due):
    return confess(store, action=action, sentence=sentence,
                   due=due, at="2026-01-05T09:00:00Z",
                   asserter="agent", approval="op-1")


def _grade(store, confession_id, grade_value):
    return grade(store, confession_id=confession_id,
                 grade_value=grade_value,
                 happened="What happened, stated plainly.",
                 at="2026-02-20T09:00:00Z", asserter="grader",
                 evidence=EVIDENCE)


def _population(tmp_path):
    store = Store(tmp_path / "r.jsonl")
    held = _confess(store, "deploy", "The deploy holds.",
                    "2026-02-01")
    _grade(store, held, "held")
    broke = _confess(store, "deploy", "The deploy is quick.",
                     "2026-02-01")
    _grade(store, broke, "broke")
    _confess(store, "deploy", "The deploy waits.",
             "2026-04-01")
    changed = _confess(store, "deploy", "The plan ships.",
                       "2026-02-01")
    store.add(text="The plan changed and is off.",
              asserter="operator",
              asserted_at="2026-01-06T10:00:00Z",
              evidence=[{"source": "note", "reference": "n-1"}],
              supersedes=changed)
    unv = _confess(store, "drop-table",
                   "The table drops in one pass.",
                   "2026-02-01")
    _grade(store, unv, "unverifiable")
    _confess(store, "drop-table", "The index rebuilds.",
             "2026-01-10")
    store.add(text="No approval stands behind this.",
              asserter="agent",
              asserted_at="2026-01-05T09:00:00Z",
              evidence=EVIDENCE, subject="deploy",
              due="2026-02-01", extra={"kind": "confession"})
    return store


def test_the_tally_counts_states_grades_and_actions(tmp_path):
    document = tally(_population(tmp_path), now=NOW)
    assert document["total"] == 6
    assert document["by_state"] == {
        "graded": 3, "awaiting": 1, "overdue": 1,
        "superseded": 1}
    assert document["graded"] == 3
    assert document["by_grade"] == {
        "held": 1, "broke": 1, "unverifiable": 1}
    assert document["actions"] == [
        {"action": "deploy", "graded": 2, "held": 1,
         "broke": 1, "unverifiable": 0},
        {"action": "drop-table", "graded": 1, "held": 0,
         "broke": 0, "unverifiable": 1}]
    assert document["malformed"] == 1


def test_the_tallys_arithmetic_closes(tmp_path):
    document = tally(_population(tmp_path), now=NOW)
    assert sum(document["by_state"].values()) == \
        document["total"]
    assert sum(document["by_grade"].values()) == \
        document["graded"]
    for grade in ("held", "broke", "unverifiable"):
        assert sum(row[grade]
                   for row in document["actions"]) == \
            document["by_grade"][grade]


def test_the_tally_of_an_empty_record_is_all_zeroes(tmp_path):
    path = tmp_path / "empty.jsonl"
    path.write_text("")
    document = tally(Store(path), now=NOW)
    assert document == {
        "total": 0,
        "by_state": {"graded": 0, "awaiting": 0,
                     "overdue": 0, "superseded": 0},
        "graded": 0,
        "by_grade": {"held": 0, "broke": 0,
                     "unverifiable": 0},
        "actions": [], "malformed": 0}


def test_the_tally_command_and_its_refusals(
        tmp_path, capsys):
    store = _population(tmp_path)
    out = tmp_path / "tally.json"
    assert main(["tally", str(store.path), "--now", NOW,
                 "-o", str(out)]) == 0
    document = json.loads(out.read_text())
    assert document["total"] == 6
    assert main(["tally", str(tmp_path / "missing.jsonl"),
                 "--now", NOW]) == 1
    assert "no such record" in capsys.readouterr().err
    assert main(["tally", str(store.path),
                 "--now", "next week"]) == 1
    assert "not an ISO timestamp" in capsys.readouterr().err

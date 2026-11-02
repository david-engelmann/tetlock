"""WW-5 to WW-16. Confessions, outcomes, and the views."""
from __future__ import annotations

from datetime import date, datetime, timedelta, timezone

from .record import Store, StoreError

GRADES = ("held", "broke", "unverifiable")
STATES = ("graded", "awaiting", "overdue", "superseded")
TERMINATORS = (".", "?", "!")


def sentence_defect(text: str) -> str | None:
    """The mechanical sentence rule (WW-5, WW-5a, WW-5b)."""
    if "\n" in text:
        return "the sentence is not one line"
    if not text.endswith("."):
        return "the sentence does not end in a period"
    if any(mark in text[:-1] for mark in TERMINATORS):
        return "the sentence holds an internal sentence break"
    if not any(ch.isalnum() for ch in text[:-1]):
        return "the sentence is empty"
    return None


def confessions(claims: list[dict]) -> list[dict]:
    return [c for c in claims if c.get("kind") == "confession"]


def confession_defects(claim: dict) -> list[str]:
    """What WW-5 requires of a confession, checked on
    claims the write path did not mint (WW-8b)."""
    defects = []
    evidence = claim.get("evidence") or []
    if not any(isinstance(e, dict)
               and e.get("source") == "approval"
               for e in evidence):
        defects.append("no approval evidence")
    subject = claim.get("subject")
    if not isinstance(subject, str) or not subject.strip():
        defects.append("no subject naming the action")
    if not claim.get("due"):
        defects.append("no due date")
    defect = sentence_defect(claim.get("text") or "")
    if defect:
        defects.append(defect)
    return defects


def _superseded_ids(claims) -> set:
    return {c["supersedes"] for c in claims if c.get("supersedes")}


class OutcomeIndex:
    """Live outcomes resolved once per view (WW-7b).

    Only an outcome claim supersedes an outcome. More
    than one live outcome on a confession, or a grade
    outside the fixed set, refuses by name."""

    def __init__(self, claims: list[dict]):
        outcomes = [c for c in claims
                    if c.get("kind") == "outcome"]
        outcome_ids = {c["id"] for c in outcomes}
        superseded_by_outcome = {
            c["supersedes"] for c in outcomes
            if c.get("supersedes")}
        self._live: dict[str, dict] = {}
        for outcome in outcomes:
            if outcome["id"] in superseded_by_outcome:
                continue
            subject = outcome.get("subject") or ""
            if not subject.startswith("confession:"):
                continue
            confession_id = subject.split(":", 1)[1]
            if confession_id in self._live:
                raise StoreError(
                    f"confession {confession_id} holds two live "
                    f"outcomes, {self._live[confession_id]['id']} "
                    f"and {outcome['id']}; graded once is the "
                    f"model and the record stands outside it")
            grade = outcome.get("grade")
            if grade not in GRADES:
                raise StoreError(
                    f"outcome {outcome['id']} carries a grade "
                    f"outside the fixed set, {grade}")
            self._live[confession_id] = outcome

    def get(self, confession_id: str) -> dict | None:
        return self._live.get(confession_id)


def live_outcome(claims, confession_id: str) -> dict | None:
    return OutcomeIndex(claims).get(confession_id)


def confess(store: Store, *, action, sentence, due, at, asserter,
            approval) -> str:
    defect = sentence_defect(sentence)
    if defect:
        raise StoreError(defect)
    return store.add(
        text=sentence, asserter=asserter, asserted_at=at,
        evidence=[{"source": "approval", "reference": approval}],
        subject=action, due=due, extra={"kind": "confession"})


def grade(store: Store, *, confession_id, grade_value, happened,
          at, asserter, evidence, supersedes=None) -> str:
    if grade_value not in GRADES:
        raise StoreError(
            f"grade must be one of held, broke, unverifiable, "
            f"{grade_value}")
    claims = store.claims()
    by_id = {c["id"]: c for c in claims}
    target = by_id.get(confession_id)
    if target is None or target.get("kind") != "confession":
        raise StoreError(
            f"no confession with id {confession_id}")
    defects = confession_defects(target)
    if defects:
        raise StoreError(
            f"confession {confession_id} is not a confession "
            f"under WW-5: {defects[0]}")
    index = OutcomeIndex(claims)
    current = index.get(confession_id)
    if current is None and confession_id in _superseded_ids(claims):
        raise StoreError(
            f"confession {confession_id} has been superseded "
            f"and cannot be graded")
    if current is not None and supersedes != current["id"]:
        raise StoreError(
            f"confession {confession_id} is already graded; a "
            f"correction must supersede outcome {current['id']}")
    if supersedes is not None and (
            current is None or supersedes != current["id"]):
        raise StoreError(
            f"supersedes names no live outcome of confession "
            f"{confession_id}, {supersedes}")
    return store.add(
        text=happened, asserter=asserter, asserted_at=at,
        evidence=evidence, supersedes=supersedes,
        subject=f"confession:{confession_id}",
        extra={"kind": "outcome", "grade": grade_value})


def _utc_date(timestamp: str) -> date:
    if not isinstance(timestamp, str):
        raise ValueError("not an ISO timestamp")
    parsed = datetime.fromisoformat(
        timestamp.replace("Z", "+00:00"))
    if parsed.tzinfo is not None:
        parsed = parsed.astimezone(timezone.utc)
    return parsed.date()


def _confession_state(claims, index, confession,
                      now_date: date) -> dict:
    superseded = _superseded_ids(claims)
    entry = {"id": confession["id"],
             "action": confession.get("subject"),
             "sentence": confession["text"],
             "due": confession.get("due"),
             "grade": None}
    outcome = index.get(confession["id"])
    if outcome is not None:
        entry["state"] = "graded"
        entry["grade"] = outcome.get("grade")
        return entry
    if confession["id"] in superseded:
        entry["state"] = "superseded"
        return entry
    due = confession.get("due")
    if due and date.fromisoformat(due) < now_date:
        entry["state"] = "overdue"
    else:
        entry["state"] = "awaiting"
    return entry


def _now_view(store: Store, now: str):
    """The shared preamble of the views judged at a
    caller-supplied now: the record must exist, the
    time must parse, the outcomes must resolve."""
    if not store.path.exists():
        raise StoreError(f"no such record, {store.path}")
    claims = store.claims()
    try:
        now_date = _utc_date(now)
    except ValueError:
        raise StoreError(
            f"now is not an ISO timestamp, {now}") from None
    return claims, now_date, OutcomeIndex(claims)


def ledger(store: Store, *, now: str) -> dict:
    claims, now_date, index = _now_view(store, now)
    entries = []
    malformed = []
    for confession in confessions(claims):
        entries.append(
            _confession_state(claims, index, confession, now_date))
        defects = confession_defects(confession)
        if defects:
            malformed.append({"id": confession["id"],
                              "reasons": defects})
    document = {"confessions": entries}
    if malformed:
        document["malformed"] = malformed
    return document


def tally(store: Store, *, now: str) -> dict:
    """WW-13 to WW-16: the counts of the record, by
    state and by grade, per action, arithmetic closed."""
    claims, now_date, index = _now_view(store, now)
    by_state = {state: 0 for state in STATES}
    by_grade = {grade: 0 for grade in GRADES}
    actions: dict[str, dict] = {}
    malformed = 0
    total = 0
    for confession in confessions(claims):
        if confession_defects(confession):
            malformed += 1
            continue
        total += 1
        entry = _confession_state(claims, index, confession,
                                  now_date)
        by_state[entry["state"]] += 1
        if entry["state"] == "graded":
            by_grade[entry["grade"]] += 1
            row = actions.get(entry["action"])
            if row is None:
                row = actions[entry["action"]] = {
                    "action": entry["action"], "graded": 0,
                    **{grade: 0 for grade in GRADES}}
            row["graded"] += 1
            row[entry["grade"]] += 1
    return {"total": total, "by_state": by_state,
            "graded": by_state["graded"],
            "by_grade": by_grade,
            "actions": [actions[name]
                        for name in sorted(actions)],
            "malformed": malformed}


def wall(store: Store, *, on: str) -> dict:
    if not store.path.exists():
        raise StoreError(f"no such record, {store.path}")
    claims = store.claims()
    try:
        day = date.fromisoformat(on)
    except ValueError:
        raise StoreError(f"not an ISO date, {on}") from None
    iso = day.isocalendar()
    monday = day - timedelta(days=day.weekday())
    sunday = monday + timedelta(days=6)
    index = OutcomeIndex(claims)
    well_formed = [c for c in confessions(claims)
                   if not confession_defects(c)]
    broke = []
    for confession in well_formed:
        outcome = index.get(confession["id"])
        if outcome is None or outcome.get("grade") != "broke":
            continue
        landed = _utc_date(outcome["asserted_at"])
        if monday <= landed <= sunday:
            broke.append({
                "action": confession.get("subject"),
                "sentence": confession["text"],
                "happened": outcome["text"],
                "outcome": outcome["id"]})
    ungraded = []
    for confession in well_formed:
        if index.get(confession["id"]) is not None:
            continue
        if confession["id"] in _superseded_ids(claims):
            continue
        due = confession.get("due")
        if due and monday <= date.fromisoformat(due) <= sunday:
            ungraded.append({
                "action": confession.get("subject"),
                "sentence": confession["text"],
                "due": due})
    return {"week": f"{iso.year}-W{iso.week:02d}",
            "broke": broke, "ungraded": ungraded}

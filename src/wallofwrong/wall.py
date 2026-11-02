"""WW-5 to WW-10. Confessions, outcomes, and the views."""
from __future__ import annotations

from datetime import date, datetime, timedelta

from .record import Store, StoreError

GRADES = ("held", "broke", "unverifiable")
TERMINATORS = (".", "?", "!")


def sentence_defect(text: str) -> str | None:
    """The mechanical sentence rule (WW-5, WW-5a)."""
    if "\n" in text:
        return "the sentence is not one line"
    if not text.endswith("."):
        return "the sentence does not end in a period"
    if any(mark in text[:-1] for mark in TERMINATORS):
        return "the sentence holds an internal sentence break"
    return None


def confessions(claims: list[dict]) -> list[dict]:
    return [c for c in claims if c.get("kind") == "confession"]


def outcomes(claims: list[dict]) -> list[dict]:
    return [c for c in claims if c.get("kind") == "outcome"]


def _superseded_ids(claims) -> set:
    return {c["supersedes"] for c in claims if c.get("supersedes")}


def live_outcome(claims, confession_id: str) -> dict | None:
    superseded = _superseded_ids(claims)
    subject = f"confession:{confession_id}"
    live = [c for c in outcomes(claims)
            if c.get("subject") == subject
            and c["id"] not in superseded]
    return live[-1] if live else None


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
    claims = store._load()
    by_id = {c["id"]: c for c in claims}
    target = by_id.get(confession_id)
    if target is None or target.get("kind") != "confession":
        raise StoreError(
            f"no confession with id {confession_id}")
    if confession_id in _superseded_ids(claims):
        raise StoreError(
            f"confession {confession_id} has been superseded "
            f"and cannot be graded")
    current = live_outcome(claims, confession_id)
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


def _confession_state(claims, confession, now_date: date) -> dict:
    superseded = _superseded_ids(claims)
    entry = {"id": confession["id"],
             "action": confession.get("subject"),
             "sentence": confession["text"],
             "due": confession.get("due"),
             "grade": None}
    if confession["id"] in superseded:
        entry["state"] = "superseded"
        return entry
    outcome = live_outcome(claims, confession["id"])
    if outcome is not None:
        entry["state"] = "graded"
        entry["grade"] = outcome.get("grade")
        return entry
    due = confession.get("due")
    if due and date.fromisoformat(due) < now_date:
        entry["state"] = "overdue"
    else:
        entry["state"] = "awaiting"
    return entry


def ledger(store: Store, *, now: str) -> dict:
    claims = store._load()
    try:
        now_date = datetime.fromisoformat(
            now.replace("Z", "+00:00")).date()
    except ValueError:
        raise StoreError(
            f"now is not an ISO timestamp, {now}") from None
    return {"confessions": [
        _confession_state(claims, c, now_date)
        for c in confessions(claims)]}


def _iso_week(day: date):
    iso = day.isocalendar()
    monday = day - timedelta(days=day.weekday())
    return iso, monday, monday + timedelta(days=6)


def wall(store: Store, *, on: str) -> dict:
    claims = store._load()
    try:
        day = date.fromisoformat(on)
    except ValueError:
        raise StoreError(f"not an ISO date, {on}") from None
    iso, monday, sunday = _iso_week(day)
    superseded = _superseded_ids(claims)
    broke = []
    for confession in confessions(claims):
        if confession["id"] in superseded:
            continue
        outcome = live_outcome(claims, confession["id"])
        if outcome is None or outcome.get("grade") != "broke":
            continue
        landed = datetime.fromisoformat(
            outcome["asserted_at"].replace("Z", "+00:00")).date()
        if monday <= landed <= sunday:
            broke.append({
                "action": confession.get("subject"),
                "sentence": confession["text"],
                "happened": outcome["text"],
                "outcome": outcome["id"]})
    ungraded = []
    for confession in confessions(claims):
        if confession["id"] in superseded:
            continue
        if live_outcome(claims, confession["id"]) is not None:
            continue
        due = confession.get("due")
        if due and monday <= date.fromisoformat(due) <= sunday:
            ungraded.append({
                "action": confession.get("subject"),
                "sentence": confession["text"],
                "due": due})
    return {"week": f"{iso.year}-W{iso.week:02d}",
            "broke": broke, "ungraded": ungraded}

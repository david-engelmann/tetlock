"""WW-6, WW-6a. The onrecord record, consumed at its
pinned discipline.

Wallofwrong keeps no store of its own. This module
reads and appends the onrecord record format exactly
as onrecord's contract (OR-5 as amended) states it:
the claim shape, the id derivation under canonical
form 1, the defect discipline (an unverifiable id is
a defect, never a skip), and the append rule
(completing an unterminated final line, never gluing).
The fixture record in the test suite was minted by
onrecord's own store, so any drift between the two
readers fails here.
"""

from __future__ import annotations

import json
from datetime import date, datetime
from pathlib import Path

from .digest import DigestError, digest_of

REQUIRED = ("id", "text", "asserter", "asserted_at",
            "evidence", "supersedes")


class StoreError(ValueError):
    """A named refusal."""


def derive_id(claim_without_id: dict, position: int) -> str:
    payload = {"claim": claim_without_id, "position": position}
    return "claim-" + digest_of(payload)[:16]


def _is_iso_timestamp(value) -> bool:
    if not isinstance(value, str):
        return False
    try:
        datetime.fromisoformat(value.replace("Z", "+00:00"))
        return True
    except ValueError:
        return False


def _is_iso_date(value) -> bool:
    if not isinstance(value, str):
        return False
    try:
        date.fromisoformat(value)
        return True
    except ValueError:
        return False


def claim_defects(claim, line_no: int, position: int) -> list[str]:
    """Shape defects of one parsed line (OR-5, OR-5a, OR-11a).
    Every defect is named; one bad field never hides another."""
    where = f"line {line_no}"
    if not isinstance(claim, dict):
        return [f"{where}: not a claim object"]
    defects = []
    for field in REQUIRED:
        if field not in claim:
            defects.append(f"{where}: missing field {field}")
    if "id" in claim and (not isinstance(claim["id"], str)
                          or not claim["id"]):
        defects.append(f"{where}: id must be a non-empty string")
    if "text" in claim and (not isinstance(claim["text"], str)
                            or not claim["text"]):
        defects.append(f"{where}: text must be a non-empty string")
    if "asserter" in claim and (
            not isinstance(claim["asserter"], str)
            or not claim["asserter"]):
        defects.append(
            f"{where}: asserter must be a non-empty string")
    if "asserted_at" in claim \
            and not _is_iso_timestamp(claim["asserted_at"]):
        defects.append(f"{where}: asserted_at is not an ISO timestamp")
    if "evidence" in claim:
        evidence = claim["evidence"]
        if not isinstance(evidence, list):
            defects.append(f"{where}: evidence must be a list")
        elif not evidence:
            defects.append(
                f"{where}: evidence must hold at least one entry")
        else:
            for i, entry in enumerate(evidence):
                if not isinstance(entry, dict) \
                        or not isinstance(entry.get("source"), str) \
                        or not entry.get("source") \
                        or not isinstance(entry.get("reference"), str) \
                        or not entry.get("reference"):
                    defects.append(
                        f"{where}: evidence[{i}] must name a source "
                        f"and a reference")
    if "supersedes" in claim and claim["supersedes"] is not None \
            and not isinstance(claim["supersedes"], str):
        defects.append(f"{where}: supersedes must be an id or null")
    if "subject" in claim and claim["subject"] is not None \
            and not isinstance(claim["subject"], str):
        defects.append(f"{where}: subject must be a string")
    if "due" in claim and claim["due"] is not None \
            and not _is_iso_date(claim["due"]):
        defects.append(f"{where}: due is not an ISO date")
    if all(field in claim for field in REQUIRED) \
            and isinstance(claim.get("id"), str):
        without = {k: v for k, v in claim.items() if k != "id"}
        try:
            expected = derive_id(without, position)
        except (DigestError, RecursionError):
            defects.append(
                f"{where}: the claim cannot be rendered under "
                f"canonical form 1, so its id cannot be verified")
            expected = None
        if expected is not None and claim["id"] != expected:
            defects.append(
                f"{where}: id does not match its derivation")
    return defects


def load_record(path: Path):
    """Return (claims, defects): every claim in order and
    every defect in the record, line numbers attached."""
    claims: list[dict] = []
    defects: list[str] = []
    try:
        raw = path.read_bytes()
    except OSError:
        return [], [f"cannot read the record, {path}"]
    if not raw:
        return [], []
    try:
        text = raw.decode("utf-8-sig")
    except UnicodeDecodeError:
        return [], ["the record is not valid UTF-8"]
    lines = text.split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    def no_duplicate_keys(pairs):
        seen = set()
        for key, _ in pairs:
            if key in seen:
                raise ValueError(f"duplicate key {key}")
            seen.add(key)
        return dict(pairs)

    parsed: list = []
    for index, line in enumerate(lines):
        line_no = index + 1
        if not line.strip():
            defects.append(f"line {line_no}: blank line")
            parsed.append(None)
            continue
        try:
            claim = json.loads(line,
                               object_pairs_hook=no_duplicate_keys)
        except RecursionError:
            defects.append(
                f"line {line_no}: nesting too deep to read")
            parsed.append(None)
            continue
        except ValueError as error:
            if "duplicate key" in str(error):
                defects.append(f"line {line_no}: {error}")
            else:
                defects.append(f"line {line_no}: not JSON")
            parsed.append(None)
            continue
        parsed.append((line_no, claim))
        defects.extend(claim_defects(claim, line_no, index))
    # Chain level defects need the whole record, and they
    # carry the line number of the claim at fault.
    numbered = [entry for entry in parsed
                if entry is not None
                and isinstance(entry[1], dict)]
    claims = [claim for _, claim in numbered]
    line_of = {}
    ids = {}
    for line_no, claim in numbered:
        cid = claim.get("id")
        if isinstance(cid, str):
            if cid in ids:
                defects.append(
                    f"line {line_no}: id {cid} appears twice")
            ids[cid] = claim
            line_of[cid] = line_no
    superseded_by: dict[str, list] = {}
    for line_no, claim in numbered:
        target = claim.get("supersedes")
        if isinstance(target, str):
            if target not in ids:
                defects.append(
                    f"line {line_no}: supersedes names no claim "
                    f"in the record, {target}")
            superseded_by.setdefault(target, []).append(
                (line_no, claim.get("id")))
    for target, sources in superseded_by.items():
        if len(sources) > 1:
            defects.append(
                f"line {sources[1][0]}: claim {target} (line "
                f"{line_of.get(target, '?')}) is superseded by "
                f"two claims, {sources[0][1]} and "
                f"{sources[1][1]}, a fork")
    return claims, defects


def _require_sound(claims, defects) -> None:
    if defects:
        raise StoreError("the record has defects: "
                         + "; ".join(defects))


class Store:
    def __init__(self, path):
        self.path = Path(path)

    def claims(self):
        """The record's claims, sound or refused (public
        read surface for the views)."""
        return self._load()

    def _load(self):
        claims, defects = load_record(self.path) \
            if self.path.exists() else ([], [])
        _require_sound(claims, defects)
        return claims

    def add(self, *, text, asserter, asserted_at, evidence,
            supersedes=None, subject=None, due=None,
            extra=None) -> str:
        claims = self._load()
        if supersedes is not None:
            by_id = {c["id"]: c for c in claims}
            if supersedes not in by_id:
                raise StoreError(
                    f"supersedes names no claim in the record, "
                    f"{supersedes}")
            if any(c.get("supersedes") == supersedes for c in claims):
                raise StoreError(
                    f"claim {supersedes} has already been superseded")
        claim = {"text": text, "asserter": asserter,
                 "asserted_at": asserted_at, "evidence": evidence,
                 "supersedes": supersedes}
        if subject is not None:
            claim["subject"] = subject
        if due is not None:
            claim["due"] = due
        if extra:
            for key, value in extra.items():
                if key == "id" or key in claim:
                    raise StoreError(
                        f"extra field {key} collides with a claim "
                        f"field")
                claim[key] = value
        # Position is the count of lines in the record; a sound
        # record holds one claim per line, so the loaded claims
        # are that count (the invariant OR-6a stands on).
        position = len(claims)
        try:
            claim_id = derive_id(claim, position)
        except (DigestError, RecursionError):
            raise StoreError(
                "the claim cannot be rendered under canonical "
                "form 1") from None
        stored = {"id": claim_id, **claim}
        defects = claim_defects(stored, position + 1, position)
        if defects:
            raise StoreError(defects[0].split(": ", 1)[1])
        line = json.dumps(stored, ensure_ascii=False,
                          separators=(",", ":")) + "\n"
        prefix = ""
        try:
            if self.path.exists() and self.path.stat().st_size:
                with open(self.path, "rb") as handle:
                    handle.seek(-1, 2)
                    if handle.read(1) != b"\n":
                        prefix = "\n"
            with open(self.path, "a", encoding="utf-8") as handle:
                handle.write(prefix + line)
                handle.flush()
        except OSError as error:
            raise StoreError(
                f"cannot append to the record, {self.path}: "
                f"{error.strerror or error}") from None
        return claim_id

    def check(self) -> list[str]:
        if not self.path.exists():
            raise StoreError(f"no such record, {self.path}")
        _, defects = load_record(self.path)
        return defects

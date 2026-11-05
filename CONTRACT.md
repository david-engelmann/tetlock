# wallofwrong Contract, stub

The full Contract is the first build act of this repo phase. This
stub records the obligations the setup already knows it owes. Each
obligation becomes executable before any feature code that depends
on it lands.

- WW-1. Before anything irreversible runs, the agent states in one sentence what it believes is about to happen, and the operator approves the sentence.
- WW-2. Confessions are onrecord claims with due dates. The wall is a view over the record, never a second store.
- WW-3. Every confession is graded against what actually happened, and false confessions stay on the wall.
- WW-4. Wallofwrong grades predictions, onrecord keeps memories.

## Round 1. Confessions, outcomes, and the wall as a view (WW-5 to WW-12)

Built in Round 1, after onrecord, whose record format
this round consumes at its pinned discipline. The
round also added the amendments below (WW-5a to WW-9a
before code, WW-5b, WW-7b, WW-8b, and WW-9b after the
reviews, before the repair code).

- WW-5. A confession is an onrecord claim in the OR-5
  shape: the claim text is the confession sentence, the
  subject key names the action it predicts, a due date
  states when the outcome should be known, and the
  evidence names the approval the stub's WW-1 requires.
  The sentence rule is mechanical: one line, ending in a
  period, with no internal sentence break. A text that
  breaks the rule is refused by name. Test,
  tests/test_wall.py.
- WW-6. Confessions and outcomes live only in the
  onrecord record file, appended under its rules. This
  repo keeps no store of its own; the ledger and the wall
  are views computed from the record on every run.
  Test, tests/test_wall.py.
- WW-7. An outcome is a claim whose subject names the
  confession it grades, carrying a grade from a fixed set
  (held, broke, unverifiable) and the evidence of what
  actually happened. An outcome naming an unknown
  confession is refused by name. A confession is graded
  once; a correction arrives as a superseding outcome
  claim, never an edit. In Round 1 the grade is supplied
  by the grader, human or operator tooling; classifying
  what kind of wrong a broke confession was is the
  decision-model work of a later round, and nothing here
  guesses at it. Test, tests/test_wall.py.
- WW-8. The ledger view lists every confession with its
  state: graded with its grade, awaiting outcome, or
  overdue when its due date has passed ungraded, with the
  current time supplied by the caller so the view is
  reproducible. Nothing is dropped from the ledger.
  Test, tests/test_wall.py.
- WW-9. The wall view renders one ISO week: the broke
  confessions whose outcomes landed that week, each with
  its sentence and what actually happened, then the
  overdue ungraded confessions of the week as their own
  list, named ungraded. Held and unverifiable
  confessions do not appear on the wall. Test,
  tests/test_wall.py.
- WW-10. A confession whose chain was superseded before
  its outcome (the plan changed) is shown in the ledger
  as superseded and never reaches the wall; grading a
  superseded confession is refused by name. Test,
  tests/test_wall.py.
- WW-11. Both views are deterministic: the same record
  and the same stated time produce byte-identical
  output. Test, tests/test_wall.py.
- WW-12. Appends follow the record's atomic append rule;
  rendered views go to stdout or a named path, written
  atomically, and every failure is a named error with
  exit 1. Test, tests/test_wall.py.

## Round 1 amendments, the pinned shapes (recorded before code)

- WW-5a. A confession claim carries the extra field
  kind with the value confession. Its evidence holds
  at least one entry whose source is approval, naming
  the WW-1 approval in its reference. The sentence
  rule, mechanical: the text is one line, ends in a
  period, and holds no other sentence terminator
  (period, question mark, exclamation mark) anywhere
  before that final period. Test, tests/test_wall.py.
- WW-6a. The record is consumed at onrecord's pinned
  discipline: the same shape checks, the same id
  derivation under canonical form 1, the same append
  rule. A defective record is refused with its defects
  named. A fixture record minted by onrecord's own
  store pins the agreement. Test, tests/test_wall.py.
- WW-7a. An outcome claim carries kind with the value
  outcome, the field grade with a value from the set,
  and a subject of the word confession, a colon, and
  the confession's claim id. Its text states what
  actually happened. Grading a confession with a live
  outcome is refused; a correction supersedes that
  outcome claim. Test, tests/test_wall.py.
- WW-8a. The ledger document lists confessions in
  record order, each with id, action, sentence, due,
  state (graded, awaiting, overdue, superseded), and
  its grade or null. Overdue is judged against the
  caller-supplied current time: an ungraded,
  unsuperseded confession whose due date is before
  the current date. Test, tests/test_wall.py.
- WW-9a. The wall is selected by a caller-supplied
  date; its week is that date's ISO week. The broke
  list holds confessions whose live outcome grades
  broke and whose outcome was asserted in the week,
  each with action, sentence, and the outcome's text
  as what happened. The ungraded list holds
  confessions due in the week with no live outcome
  and no supersession. Test, tests/test_wall.py.

## Round 1 amendments, after the reviews (recorded before the repair code)

- WW-5b. A sentence must hold content: the text
  before the final period contains at least one letter
  or digit. A bare period is refused as empty.
  Test, tests/test_wall.py.
- WW-7b. Only outcome claims advance an outcome
  chain. An outcome superseded by a plain claim still
  governs. When more than one live outcome stands on
  a confession, or a live outcome carries a grade
  outside the fixed set, the views refuse, naming the
  confession and the outcomes: graded once is the
  model, and a record outside it is not silently
  resolved. Test, tests/test_wall.py.
- WW-8b. A confession graded before it was superseded
  keeps its graded state and its grade; superseded is
  the state only of a confession superseded while
  ungraded. A confession claim failing WW-5 (no
  approval evidence, no subject, no due date) is
  named in the ledger document's malformed list with
  its reasons and never reaches the wall; grading one
  is refused by name. Test, tests/test_wall.py.
- WW-9b. Dates are read in UTC: a timestamp carrying
  an offset is converted before its date is taken, so
  the same instant lands in the same week however it
  is written. The wall's broke entries also carry the
  outcome's claim id under outcome, naming the
  receipt. Test, tests/test_wall.py.

## Round 2. The tally (WW-13 to WW-16)

- WW-13. A third view, the tally: over the whole
  record at a caller-supplied now, an ISO timestamp
  read in UTC as the ledger reads it, the counts of
  the well-formed confessions by state (graded,
  awaiting, overdue, superseded) and, among the
  graded, by grade (held, broke, unverifiable).
  Counts ride with their denominators in the document
  itself, never as a bare rate: the state counts
  under the confessions total, the grade counts
  under the graded count. Every named count is
  always present, zero included, so the table can
  be checked by addition. Test, tests/test_wall.py.
- WW-14. The arithmetic closes. The state counts sum
  to the confessions total; the grade counts sum to
  the graded count. Malformed confessions (WW-8b)
  are excluded from every count above and reported
  as their own single count. Test,
  tests/test_wall.py.
- WW-15. Per action: one row for each action holding
  at least one graded well-formed confession, naming
  the action, its graded count, and its counts by
  grade, rows ordered by action name, and the rows'
  grade counts sum to the document's. An action with
  no graded confession earns no row; its confessions
  still count in the state totals. Test,
  tests/test_wall.py.
- WW-16. The tally stands on the ledger's own
  classification: the same states, the same outcome
  index, the same refusals, a missing record refused
  by name. The ledger and wall documents do not
  change. The command is tally, with the record, a
  required now, and the views' output discipline
  (WW-12). Test, tests/test_wall.py.

## Round 2 post-review amendments, recorded before the repairs

- WW-8c. The malformed test of WW-8b is the whole of
  WW-5. A confession claim whose text breaks the
  sentence rule is malformed, and a subject that is
  empty or all whitespace names no action and is
  malformed. Round 1 checked only approval, subject
  presence, and due date, so a forged confession with
  a two-sentence text stood as well-formed in every
  view and could be graded; the reviews drove it
  through the tally and the ledger alike. Every view
  and the grading path read the one definition, so
  the repair lands in the definition. Test,
  tests/test_wall.py.

## Round 3. The Rust port (WW-17 to WW-23)

- WW-17. The port reproduces the behavior of the
  Python of Rounds 1 and 2 exactly: every command's
  exit code, stdout, stderr, and file effects byte
  for byte, including the refusal and defect phrases
  in their order. Where exact reproduction is
  impossible or would import a defect of the
  interpreter rather than of this program, the
  deviation is named in the round's dossier, never
  silently absorbed.
- WW-18. A library crate with a thin binary. The
  record layer is consumed from the onrecord crate
  at a pinned revision (the Round 3 merge of that
  repo): the claim shape and defect discipline, the
  id derivation, the digest, and the record loader
  are the crate's functions, not copies. This
  repo's own Store surface (claims, add with the
  extra field rule, check) stands as an adapter
  over those primitives with its own append, and
  every place the adapter's behavior is this repo's
  rather than the crate's is named in the dossier.
  Test, tests/test_wall.rs.
- WW-18a, recorded before the port code. The
  record's due check follows the crate: this repo's
  Python copy carried an unpinned date rule (plain
  date.fromisoformat) where onrecord's OR-13a pins
  the extended calendar form, so the two shipped
  readers already disagree on a record holding a
  basic-form or week-form due. The module's own
  header claims the record is consumed exactly as
  OR-5 as amended states it, and WW-6a pins the
  consumption to onrecord's discipline, so the
  crate's rule governs and the drift retires with
  the copy; the dossier names the behavior change
  as a deviation (a record the Python here blessed
  can be refused by the port, in agreement with
  onrecord). The wall command's own date argument
  has no onrecord counterpart: it keeps the wide
  date.fromisoformat acceptance, the calendar and
  ISO week forms, extended and basic, implemented
  in this crate.
- WW-19. The loader budgets are this repo's own,
  probed per command at the shipped CLI before
  they are coded: the crate loader is called with
  the budget each entry point's Python frame chain
  earns, and the harness pins the boundary per
  command rather than inheriting onrecord's
  numbers.
- WW-20. Rendering reproduces the Python's JSON
  byte for byte: appended claim lines in compact
  separators with insertion key order, the views
  at indent two, non-ASCII unescaped. Test,
  tests/test_wall.rs.
- WW-21. The Python tests port to cargo tests file
  for file and remain the primary suite. The
  onrecord-minted fixture record stays the fixture,
  so drift between the two readers still fails
  here, and the family golden digest test still
  pins the derivation. Test, tests/test_wall.rs.
- WW-22. A differential harness twins the Rust
  binary against the Python over the whole command
  surface (confess, grade, ledger, wall, tally),
  comparing exit codes, streams, and files after
  every step, including the defect catalogue
  translated to confessions and the views' output
  discipline. Run at the round's checkpoint and
  reported in the dossier.
- WW-23. The round's dossier carries the port
  report: port or rewrite with reasons, the Rust
  versus Python line share on the landing tree,
  the bindings determination, and every non-Rust
  remainder with its justification.
- WW-24. Round 4 re-pins both crates: flighttape
  from ba8d4b550850179722f48b8f9c909e199efe3548 to
  eebce0be75a9c2ffa534635e37160c7cbef5d96e (its
  Round 12 merge) and onrecord from
  3df5c1fbcc027d9df71e4b65f6b1c51b63680815 to
  c6516073a190623c1109268c7505340405e72c0f (its
  Round 4 merge, which pins the same flighttape
  rev). The named deviation 2 of the Round 3
  dossier (the crate-owned timestamp stray digit)
  retires as it did in onrecord Round 4: the
  harness's deviation cells flip to agreement,
  and FT-60a's shapes are asserted through the
  surfaces that read timestamps. No other named
  deviation moves.
- WW-25. The output guard's resolver peels the
  last component of any kind, not only a file
  name: a path whose nonexistent tail ends in a
  ".." component behind a symlinked prefix
  resolves against the resolved prefix, as
  pathlib's realpath order does (the shape
  secondbill Round 3's spec probe proved
  divergent). The differential harness gains the
  interleave cells (a symlinked escape component,
  a nonexistent deep component, then "..", then
  the target name), on both the guard-refuses
  and guard-allows sides.
- WW-26. The round's verification is the full
  local gate plus the differential harness
  re-run over its standing corpus at zero
  failures, WW-24 and WW-25 cells included.
- WW-27. CodeRabbit's advisory pass on Round 3's
  PR 4 posted no findings (zero entries on the
  review endpoints; its summary shell and an
  action-not-completed notice stand on the PR),
  so no disposition is owed from it.
- WW-25a. Amendment, on the red team's Round 4
  Major: the resolver reports failure. When
  resolving the output path hits a symlink loop
  (ELOOP), the output counts as naming an input
  and the guard refuses it, as bluepencil's
  BP-16a counts an unresolvable path. The port
  never writes through a loop: a loop named as
  the output is refused at the guard, where the
  Python crashes uncaught (deviation 3's crash
  class). A record behind a loop stays in the
  crash class: the port's named load refusal
  against the oracle's traceback, exit 1 on both
  sides, nothing written. The harness gains the
  loop cells on both shapes plus the agreeing
  boundary (a loop component cancelled by a
  following ".." refuses at the guard on both
  sides, as before).
- WW-28. Round 5 discharges the two probes filed
  in this line's Round 5 owed file, and adds a
  third the sibling line just proved live. The
  recursion band: unheard's Round 4 measured
  pathlib's recursive walk crashing with
  RecursionError at 991 nested splices (990
  resolve); this crate's oracle crashes uncaught
  out of resolve while its port answers a splice
  failure as naming the record with a refusal
  phrase, so an output spelled through a chain
  near the band likely diverges in streams even
  where exit and effects agree. The probe pass
  measures the band at this crate's guard (chain
  lengths bracketing both siblings' constants,
  onto the record and away) on the oracle and
  the port and fixes the divergence set before
  any repair is written.
- WW-29. The deleted working directory: from a
  deleted cwd, relative and absolute output
  spellings onto and away, measured cell by cell
  against this crate's guard structure (its
  resolver is fallible per WW-25a and counts an
  unresolvable output as naming the record),
  before any repair is written.
- WW-30. The tailed output spelling: retroforge's
  Round 5 red team proved a pre-existing write-
  seam class in that crate (spellings ending in
  a separator or dot component write on the
  oracle and refuse on the port, because the
  port hands the raw spelling to rename while
  pathlib's construction drops the tail). This
  crate's write seam is probed for the same
  class in the same pass, on every command that
  writes (ledger, wall, tally, confess, grade as
  the surface allows), and the class boundary is
  measured before any repair is written.
- WW-31. Repairs follow the measurements under
  new clauses if a probe set expands; the
  differential harness gains the proven cells
  with control runs against the pre-repair
  binary failing exactly the new cells, and the
  standing harness stays clean.
- WW-32. README records 5 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner verification.
- WW-33. Amendment, on the WW-28 probe pass: the
  chain law at the guard. Resolution follows
  splices the way the interpreter's walk does,
  component by component in userspace, never by
  a whole-path canonicalize: a chain longer than
  the kernel's whole-path link cap is not a
  loop. Each splice recurses into the component
  that carries it, so depth is nesting (a splice
  inside another splice's resolution), not a
  count of splices in the spelling; 991 nested
  splices resolve and the 992nd fails the
  resolution, the constant of this CLI measured
  at its guard on head chains and prefix chains
  alike, never inherited from a sibling. A link
  target splices as the interpreter's does:
  absolute targets restart at the root, relative
  targets join the accumulated prefix, and a
  ".." in the spelling or in a target pops the
  accumulated resolved prefix lexically, without
  being read. A loop is what the walk's seen
  map convicts (a path revisited while its own
  resolution is still open and not cancelled
  by a following ".." in the finished path).
- WW-34. Amendment, failure postures under the
  walk. An output whose resolution fails for
  any reason (a loop, the depth budget) counts
  as naming the record under WW-25a and is
  refused at the guard, where the interpreter
  crashes uncaught: the refusal phrase against
  the traceback is the standing named deviation,
  exit and effects agreeing. Where the guard
  resolves a long prefix chain faithfully, the
  write then fails in the spelled parent past
  the kernel cap on both sides and the answer
  is the cannot-write phrase, not the guard's.
- WW-35. Amendment, on the WW-29 probe pass: the
  deleted working directory. A failed current-
  directory read fails the resolution of a
  relative spelling outright; an absolute
  spelling never consults it. On the output
  side the failure counts as naming the record
  (WW-25a's refusal). On the record side, in
  the guard, the failure takes the bare crash
  class at the guard itself, exit 1 with
  nothing printed and nothing written: the
  interpreter dies inside the guard expression,
  before any load, and its load would otherwise
  succeed through a ".." spelling and write.
  Where no output is named, no resolution is
  evaluated on either side and nothing changes:
  views to stdout and the confess and grade
  appends keep their measured agreement.
- WW-36. Amendment, on the WW-30 probe pass:
  the write consumes the constructed spelling.
  As pathlib's construction hands the path to
  _write_atomically, so here: construction
  drops every empty component and every "."
  component wherever it stands in the spelling;
  a ".." component and every other component
  stand whole; a leading pair of separators
  stands as the anchor and a longer leading run
  collapses to one separator; a spelling with
  no component left stands as "." (or as its
  anchor). The temp file is created in the
  constructed path's parent, its prefix is the
  constructed path's name and a dot, and the
  replace target is the constructed path
  itself. The guard's resolution and the
  cannot-write message keep the raw argv
  spelling, as the Python's do. The record
  seam already follows the same construction
  discipline and is measured agreeing; it does
  not change.
- WW-37. Amendment, the harness cells. The
  harness gains: head chains onto at 41 and
  991 (plain twins) and at 992 (a dev cell,
  the refusal phrase against the traceback),
  head chains away at 41 and 991 (plain twins,
  the view replacing the chain head) and at
  992 (dev), prefix chains away at 41 and 991
  (plain twins failing at the write on both
  sides) and at 992 (dev), the dotdot-target
  chain (a plain twin writing through the
  chain head), deleted-cwd cells through a
  launcher step (relative output away and
  onto as dev cells, a relative record with
  an absolute output as a crash-class dev
  cell, absolute onto and away as plain
  twins, a no-output view over a relative
  record as a plain twin), and the tailed
  outputs (trailing separator, trailing dot,
  trailing separator-and-dot mixtures, a
  nested path's tailed name, on ledger, wall,
  and tally) as plain twins, with tailed onto
  spellings and the boundary spellings
  (trailing "..", a mid-spelling "..", ".",
  the empty spelling) pinned as agreeing.
  Control runs against the pre-repair binary
  fail exactly the new cells.

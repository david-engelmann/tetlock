# tetlock Contract, stub

The full Contract is the first build act of this repo phase. This
stub records the obligations the setup already knows it owes. Each
obligation becomes executable before any feature code that depends
on it lands.

- WW-1. Before anything irreversible runs, the agent states in one sentence what it believes is about to happen, and the operator approves the sentence.
- WW-2. Confessions are onrecord claims with due dates. The wall is a view over the record, never a second store.
- WW-3. Every confession is graded against what actually happened, and false confessions stay on the wall.
- WW-4. Tetlock grades predictions, onrecord keeps memories.

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
- WW-38. Amendment, on a Round 4 harness cell
  the first repair failed (an output spelled
  through a loop component and a following
  ".."): the loop conviction is not the walk's
  seen map alone. Where the walk meets a path
  under its own open resolution it stops and
  returns the part resolved so far with the
  rest of the spelling standing; the resolution
  is the lexical normalization of that finished
  spelling, and the loop is convicted only by
  the interpreter's closing stat of the
  finished path, which raises on ELOOP and
  swallows every other failure. So a loop
  component a following ".." cancels out of
  the finished path resolves clean (the
  spelling writes or fails at the write, as
  its resolved self dictates), a loop left
  standing in the finished path takes the
  failure postures of WW-34 and WW-25a, and a
  resolution the walk completes is statted
  too, its dangling end no failure at all.
- WW-39. Round 6 probes the temp placement law
  at this crate's view write seam, the family
  measured in secondbill's Round 5 and
  retroforge's Round 6 and which this crate's
  seam predates. The oracle's view write
  creates its temp through tempfile.mkstemp,
  which absolutizes the parent lexically (cwd
  join, then normpath, no link resolution);
  this crate's write creates its temp under
  the constructed parent as spelled (WW-36).
  The probe set is the popped-link family at
  every command that writes a view (ledger,
  wall, tally): an output whose parent
  contains a symlink component that a
  following dotdot pops, across the existence
  boundary (the lexical parent exists as a
  directory, neither parent exists, the
  popped component is a real directory), in
  plain, tailed, absolute, deep, and
  escape-link spellings, each measured
  against the oracle before any repair is
  written. Repairs follow the measurements
  under new clauses; the harness gains the
  proven cells with controls against the
  pre-repair binary failing exactly the
  convicted cells, boundary pins ordered
  before convicted cells where a one-sided
  write could echo through later snapshots.
- WW-40. The CodeRabbit disposition owed from
  Round 5 discharges here on evidence: the
  Round 5 pull request carries zero inline
  findings on read-back (checked 2026-11-04 in
  the rotation-wide late sweep), so there is
  nothing to fix and the absence is the
  disposition.
- WW-41. README records 6 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner
  verification.
- WW-42. The view write's temp placement, the
  Round 6 repair. The temp is created under the
  lexically absolutized parent of the
  constructed path: a relative parent is
  joined to the working directory, then the
  components are normalized lexically (empty
  and "." components drop, ".." pops the
  spelled component before it, a leading
  separator pair stands as the anchor), with
  no link resolution anywhere in that step.
  The rename target remains the constructed
  path itself, and the cannot-write message
  keeps the raw output spelling. This governs
  every view command (ledger, wall, tally),
  whose writes share the one seam. The
  convicted family is the popped link: an
  output whose parent contains a symlink
  component that a following ".." pops, where
  the lexical parent differs in existence
  from the physically resolved parent; the
  pre-repair write landed the view at the
  physical target where the oracle's mkstemp
  refuses on the lexical parent. WW-42
  supersedes WW-36's silence on the temp's
  placement; the construction WW-36 states
  for the target stands unchanged.
- WW-43. The boundary agreements and the
  scope of WW-42. The pop family agrees where
  the lexical parent exists as a directory
  and the physical does not (the oracle's
  temp is created and its replace then
  fails; the write refuses on both sides),
  where neither parent exists, where both
  exist (the view lands at the physical
  target on both sides), and where the
  popped component is a real directory (the
  two parents are one). A target carrying
  its own ".." is convicted or agrees by the
  same existence test on the parents it
  produces. The append seam is outside the
  law: confess and grade open the record
  path directly, the kernel resolves the
  spelling physically on both sides, and the
  family is measured absent there. Inside
  the temp step, an absolute parent never
  consults the working directory; a failed
  working-directory read on a relative
  spelling fails the write with the
  cannot-write refusal, a posture that
  stands behind the guard's own deleted-cwd
  postures (WW-35), which decide every
  reachable deleted-cwd cell first.
- WW-44. The harness gains the temp
  placement cells: a poptree step that
  rebuilds the popped-link tree fresh in
  both case dirs before each action step,
  and one case per command family (ledger,
  wall and tally at the plain pop; ledger
  across the tailed, deep, sequence,
  dotdot-target, absolute, lexical-file, and
  overwrite shapes), each case ordering its
  boundary pins before its convicted cells
  so a one-sided write cannot echo through
  later snapshots. The true root-escape
  shape (a link target leaving the case
  tree) is probe-pinned in the lane record,
  not harness-celled, because a snapshot
  cannot carry a write outside its tree;
  its in-tree form (a link in a subdirectory
  whose target climbs out of the
  subdirectory) is celled as the deep pop.
  Control runs against the pre-repair binary
  fail exactly the convicted steps, and the
  boundary pins and every standing case pass
  the control.
- WW-45. Amendment, on the Round 6 red team's
  deleted-working-directory battery: WW-35's
  output-side arm corrected. A view output
  whose guard resolution fails on the failed
  current-directory read (a relative spelling
  from a deleted working directory) takes the
  bare crash class at the guard, exit 1 with
  nothing printed and nothing written: the
  interpreter's guard evaluates the output's
  resolve first and dies there, before any
  comparison can name the record, so the
  refusal phrase never fires for this failure.
  The boundary stands everywhere else: loop
  and budget failures on the output side keep
  WW-34's posture (counted as naming the
  record, refused with the guard phrase), a
  resolution that completes compares as
  before, and the record side is untouched
  (its failed-cwd arm already takes the crash
  class under WW-35, and its walk failures
  still fall through to the load). Absolute
  output spellings never consult the working
  directory and are unchanged. The harness's
  deleted-cwd case states the arm: relative
  outputs, plain and popped spellings, on
  ledger and tally against absolute and
  relative records take the crash-class
  expectation on this side against the
  oracle's traceback, and the absolute
  spellings from a deleted working directory
  stand as twinned pins; control runs against
  the pre-WW-45 binary fail exactly those
  steps.
- WW-46. The CodeRabbit disposition owed from
  Round 6 discharges here on evidence: the
  Round 6 pull request (PR 7) carries zero
  inline findings and zero reviews on
  read-back (checked 2026-11-06), and its
  summary shell records the review as failed
  because the pull request is closed, with a
  findings-free walkthrough, the standing
  shape since flighttape Round 13. There is
  nothing to fix and nothing to dismiss; the
  absence and the shell's exact state are the
  disposition. Any late findings on this
  round's pull request are owed to Round 8's
  contract by the same shape.
- WW-47. Round 7 measures this crate's temp
  suffix, among the last unmeasured seams in
  the line. The shipped generator builds the
  candidate as the constructed name, a dot,
  and the decimal digits of the process id,
  a per-process counter, and the attempt,
  concatenated, so its width is the digit
  count of three numbers and the name
  boundary moves with the width of the
  process id that happens to run. Measured
  against the oracle, whose suffix is
  exactly eight characters at every process
  id (writes at a 246-byte name, refuses at
  247): at process id 1 the port's suffix is
  three characters and it writes through a
  251-byte name, refusing at 252; at a
  three-digit process id the suffix is five
  and it writes through 249, refusing at
  250. Two further arms stand at the same
  seam. The candidate's name is built
  through a lossy conversion, so a name
  carrying one raw byte is lengthened by
  two bytes before the boundary is counted
  (at process id 1 the port writes a
  raw-byte name through 249 bytes where the
  plain name runs to 251; at a three-digit
  id, through 247 where the plain name runs
  to 249). And the draw is a public function
  of the process id, the counter, and the
  attempt, every term known or enumerable
  before the process runs: planting the one
  first candidate is absorbed by the redraw,
  as the oracle absorbs a plant, but
  planting all 65 candidates the retry loop
  can draw makes the port refuse a write
  the oracle lands. The observable law is
  the one the family settled: the candidate
  is the name, a dot, and exactly eight
  characters, drawn so the sequence cannot
  be computed from the process id, created
  exclusively under the lexically
  absolutized parent at mode 0600.
- WW-48. The repair, at the one view seam.
  The candidate is assembled from raw bytes
  end to end (the constructed name's own
  bytes; no lossy step anywhere in the
  construction), a dot, and exactly eight
  lowercase hexadecimal characters drawn
  from the first four bytes of a SHA-256
  hash over the process id, the wall-clock
  nanoseconds, the per-process counter, the
  attempt, the temp directory's bytes, and
  the name's bytes, so successive draws
  differ and the sequence is not a public
  function of the process id. The hash is
  the sha2 crate, already this crate's
  transitive dependency through flighttape
  and now named directly; the construction
  is the one flighttape's Round 15 repair
  settled for the same seam. The create
  stays exclusive with redraw on
  AlreadyExists inside the standing
  64-attempt budget, the mode stays 0600,
  the temp is removed on any failure, and
  the rename target and the cannot-write
  message are unchanged. WW-42's placement
  and WW-36's construction stand untouched.
- WW-49. The harness gains the suffix
  cells: boundary lengths on ledger in
  plain, popped, and raw-byte spellings,
  run in two process-id postures (process
  id 1, and a namespace with 200 processes
  burned so the binary runs at a
  three-digit id), plus the planted cells
  at process id 1 (one plant, absorbed;
  all 65 plants, refused before the repair
  and written after it). The ambient
  posture's boundary is not pinned: its
  drawn width varies with the ambient id,
  which is the defect, not a posture.
  Controls: the pre-repair binary fails
  exactly the convicted steps in each
  posture; a mutant drawing eighteen
  characters fails the refusal side of the
  boundary and a mutant keeping the lossy
  name fails exactly the raw-byte steps.
- WW-50. The re-pin. This crate consumes
  flighttape directly (its value model, and
  timeparse's parse at the wall view's two
  day computations, for the ledger and
  tally --now and for a claim's asserted
  time) and through onrecord's store, whose
  claim gate reads a timestamp's validity
  from the same parse. Both pins stand two
  rounds behind: flighttape at its Round 12
  merge eebce0be and onrecord at its Round 4
  merge c6516073, whose own flighttape pin
  is the same Round 12 head. Onrecord's
  Round 5 changed no source of its own,
  only its flighttape pin, so the whole
  two-jump delta is flighttape Round 13's
  timestamp law (the fraction window and
  the lone-NUL terminator) reaching three
  acceptance surfaces in this CLI: the
  record load's timestamp gate, the --now
  day, and the wall view's landing day.
  Triplet probes (the Python oracle, the
  shipped binary, the re-pinned binary)
  over sixteen fraction and separator forms
  on the --now and --at surfaces find the
  shipped binary diverging from the oracle
  on exactly three forms (a colon standing
  as the fraction separator, a fraction
  whose digits run past six into a leftover
  the boundary terminates, at Z and at an
  offset sign) and the re-pinned binary
  agreeing on every form. The round
  therefore moves both pins in one step:
  flighttape to 74a83298 (its Round 13
  merge) and onrecord to 9e41a7be (its
  Round 5 merge). No source of this crate
  changes for the re-pin; the render-cost
  model and every other surface stand as
  they were.
- WW-51. The harness gains the re-pin
  cells: seeded records whose asserted time
  carries the delta's shapes (the colon
  fraction, the leftover fraction, and a
  lone NUL standing as the whole tail after
  the seconds, the terminator's completion
  point), driven through the views that
  read them, and --now steps at the three
  convicted forms plus two agreement forms.
  The control run against the shipped
  binary fails exactly the convicted steps;
  every standing case passes both runs.
- WW-52. The record-side loop family,
  measured on the Round 6 red team's cell
  E1 and the shapes the owed file named. A
  record spelled through a bare self-loop
  parent, a mutual-loop parent, on ledger,
  tally, and wall, with absolute and
  relative outputs: the oracle raises
  RuntimeError out of the guard's record
  resolve, uncaught, and the port falls
  through to the load's named refusal,
  error, no such record, with the spelling
  as typed; exit, stdout, and trees are
  identical on every cell. A loop the
  spelling cancels (the parent looped, a
  following dotdot standing in the finished
  path) resolves clean on both sides,
  whether the record beneath exists or
  not, and the load then refuses by name
  on both sides in identical words, the
  kernel's loop error reading as no such
  record in the load on each. A plain link
  chain onto a missing record, a
  loop-shaped spelling whose target is the
  directory itself onto an existing record,
  and the same record named with no output
  at all agree outright. Decision: WW-25a's
  record-side arm stands. A record-side
  walk failure falls through to the load's
  named refusal, the crash class's named
  side, in every geometry the family
  measures; moving that arm to the bare
  crash class would rewrite WW-25a's
  pinned own-name cell and the guard's
  failure taxonomy to change no exit,
  stdout, or effect in any measured
  geometry, only which named text the port
  prints on one side of a deviation that
  is already named. The family is pinned
  as the harness measures it under WW-53.
- WW-53. The harness gains the record-side
  loop cells: the loop-parent shapes as
  dev cells in the WW-52 pairing (the
  port's named refusal against the
  oracle's traceback), and the cancelled,
  chain, self-target, and no-output shapes
  as plain twins. Control runs against the
  shipped binary pass the whole case: the
  round moves no record-side behavior, and
  the cells pin the posture so a future
  change to it fails loudly.
- WW-54. Amendment: WW-42's wording
  aligned. The clause said the temp's
  directory step keeps a leading separator
  pair as its anchor; the normalization
  that step performs rebuilds every
  spelling under a single leading
  separator (empty components drop, the
  pair with them), and the pair stands
  only in WW-36's construction and in the
  rename target it produces. The temp
  directory string reaches no output and
  names the same directory on Linux either
  way, so the alignment is to the words,
  not the behavior: WW-42 is to be read
  with its anchor sentence applying to the
  construction, and the temp's step stated
  as the single-separator rebuild it is.
- WW-55. README records 7 of 100 only in
  the closure commit written after this
  round's three reviews, fixes, and owner
  verification.
- WW-56. Amendment, on this round's three
  reviews. Four records the clauses above
  state too loosely or leave out.
  (a) WW-49's control sentence reads with
  the tree-echo qualification: the harness
  compares a full tree snapshot after
  every step, so once a convicted write
  lands on one side only, the case's later
  steps fail as echoes of that split. At
  433 steps the control's 26 failures are
  producing steps plus echoes. The
  process-id-1 suffix case fails steps 3
  to 11: producing at 3, 4, 6, 8, 10, and
  11 (the plain names at 247 and 251, the
  raw name at 249, the popped name at 247,
  and the added raw names at 245 and 246)
  and echoing at 5, 7, and 9. The
  three-digit case fails steps 2 to 8:
  producing at 2, 3, 5, 7, and 8 (the
  plain names at 247 and 249, the raw
  names at 247, 245, and 246) and echoing
  at 4 and 6. The planted case produces 1,
  the all-65 step. The re-pin case
  produces 9. The 53 standing cases and
  the loop family pass on the control.
  (b) The harness of record for this round
  is the rename-adapted differential
  (tetlock.cli against the staged renamed
  oracle, with the tetlock help text),
  extended in place. The goals-tree
  standing copy still carried the
  pre-rename wallofwrong form, and the
  owner has canonicalized it from this
  round's harness. Before the pairing was
  traced, it reproduced 32 failures in a
  verified-clean window, all on
  prog-bearing surfaces. The probe,
  triplet, and gate evidence was gathered
  on prog-free surfaces and stands.
  (c) The battery-design rule from mutant
  L: boundary cells belong at each single
  defect's own flip point, not at the
  shipped combination's. At fixed width
  eight, the lossy arm's flip sits at true
  basename lengths 245 and 246, outside
  the shipped combination's raw window
  (249 and 250 at process id 1, 247 and
  248 at three digits), so a battery
  placed only at the combination's flip
  passes a mutant that keeps the lossy
  name. The four cells added at 245 and
  246, at the end of each suffix case, are
  what convict that mutant. Future
  boundary batteries place cells per arm.
  (d) temp_suffix is this crate's first
  wall-clock read: SystemTime appears
  exactly once in src/, there, and record
  timestamps are caller-supplied and
  parsed, never read. WW-48's nanos term
  is justified by WW-47's requirement,
  convicted in this round by the 65-plant
  refusal, that the draw not be computable
  from the process id. That is why this
  crate takes the flighttape Round 15
  (FT-74) shape, while unheard's UH-43
  fitting, where determinism was filed
  rather than convicted, differed on its
  own facts.
- WW-57. Amendment, on CodeRabbit's
  finding on PR 9 (comment 4498348502),
  valid on the code as written. WW-48's
  nanos term was drawn as
  duration_since(UNIX_EPOCH) with a
  fallback of zero on error, so a clock
  reading before the epoch made the time
  term the constant zero and the draw a
  public function of the process id,
  counter, attempt, directory, and name
  again, the determinism WW-47 convicted.
  The repair draws the term on the error
  branch from the SystemTimeError's own
  duration, the magnitude of the
  reading's distance from the epoch,
  which varies with the actual clock
  reading instead of standing constant.
  The post-epoch path is unchanged. The
  observable law (width, retry, mode,
  placement) is untouched, because the
  term feeds only the hash mix.
- WW-58. The CodeRabbit disposition owed
  from Round 7 discharges here on
  evidence, read back in full in this
  round (checked 2026-11-07). The Round 7
  pull request (PR 9) carries three
  reviews, three review comments, and
  four issue comments. Its one inline
  finding (comment 4498348502, the
  pre-epoch constant fallback) was valid
  on the code as written and was repaired
  inside Round 7 as WW-57, with the
  owner's factual reply standing on the
  comment (4499472514,
  2026-11-07T01:55:59Z, naming the repair
  as landed at the pull request's head).
  The summary shell (6697493145) stands
  in completed walkthrough form, last
  updated 2026-11-07T01:55:48Z, before
  the 01:57:14Z merge, with no Review
  failed marker; the remaining
  shell-side comment (6697607791) is the
  action notice, Already reviewed the
  last commit. One further item arrived
  after the owner's reply and fifty
  seconds before the merge, and stands
  unanswered on the pull request:
  CodeRabbit's follow-up (comment
  4499474239, review 5809330881,
  2026-11-07T01:56:24Z), which WW-59
  files. Any late findings on this
  round's pull request are owed to
  Round 9's contract by the same shape.
- WW-59. Amendment, on CodeRabbit's
  follow-up on PR 9 (comment 4499474239),
  filed here exactly rather than left
  unanswered. CodeRabbit accepts the
  WW-57 repair and states a residual
  concern: on a clock that is fixed or
  predictable the error-branch term is
  predictable with it, a hash adds no
  randomness, and an actor who knows the
  other inputs could pre-create the
  candidates; it asks for an
  unpredictable input, such as operating
  system randomness, for pre-epoch draws
  if that threat is in scope. The
  concern is sound inside its premise,
  and its premise is the scope line.
  WW-48's law is that the draw cannot be
  computed from the process id, and the
  clock term varies with the actual
  reading (WW-57); on a clock frozen at
  one reading any clock-derived term
  stands still, on either side of the
  epoch, and computability returns for
  an actor who knows that reading to the
  nanosecond. No clause of this contract
  claims the draw survives a frozen and
  known clock. Whether operating system
  randomness enters the hash mix is a
  source question for a later round, to
  be taken up with the threat decided;
  this round changes no behavior, and
  the item stands as the named candidate
  the next source round at this seam
  answers first.
- WW-60. The CodeRabbit disposition for
  the CI shaping pull request (PR 10)
  discharges here on the same read-back.
  PR 10 carries one review, one review
  comment, and one issue comment, its
  summary shell (6700624230) in completed
  walkthrough form, last updated
  2026-11-07T01:55:28Z, before the
  01:57:38Z merge. Its one finding
  (comment 4499468420, Minor) observes
  that creating a merge group would run
  the gate twice, once for the push
  event on the queue branch and once for
  merge_group, and asks that the push
  run skip queue branches. The
  observation is accurate and the repair
  is not taken in this round, on two
  recorded grounds. The workflow is the
  fleet's uniform shaping step, landed
  the same day on all ten of the line's
  repos, and a one-repo divergence would
  trade a fleet property for a local
  one. And merge queues cannot be
  enabled on user-owned repos at all
  (the ruleset API rejects the
  merge_queue rule there), so no merge
  group can form on this repo today and
  the duplicate run cannot occur. The
  exclusion belongs to the fleet pass
  that enables queues, and this item is
  the finding's answer of record.
- WW-61. The README takes the fleet
  storefront standard in this round. The
  first screen stands in the standard's
  order: the title, the lede byte
  identical to the repository description
  read back through the API (David's
  sentence, standing as the pinned
  description in the fleet form), one
  paragraph carrying the boundary
  sentences, the one ci badge, and one
  captured output. Install and
  Quickstart run verbatim from a clean
  staging, Quickstart over the synthetic
  record that ships in the test
  fixtures; Usage shows captured output
  for all five commands, every capture
  taken from this round's binary over
  synthetic inputs. The badge is claimed
  in workflow-state form only: the
  workflow (403263182) stands active and
  its latest run on main at this round's
  base completed success
  (2026-11-07T01:57:40Z); this lane's
  fetch of the badge image timed out,
  and no rendered state is claimed. The
  Status section tracks reality and the
  rounds line stays at 7 of 100 in this
  round's commits (WW-64).
- WW-62. Item 3 of the storefront
  checklist does not pass and stands
  recorded open. Cargo.toml carries no
  description field, so the one-line
  description stands in two of its three
  homes, the repository description and
  the README lede, byte identical in
  both. Writing the field is a separate
  pass; no description is invented
  here.
- WW-63. The gate and the harness on
  this round's tree. The Python
  codepoint scan over all tracked files
  returns U+2014=0 and U+2013=0; cargo
  fmt --check is clean; cargo clippy
  --all-targets with warnings denied is
  clean; cargo test passes 28 with 0
  failed, the Round 7 baseline held on
  a tree that changes no source. The
  standing differential harness (the
  goals-tree copy the owner
  canonicalized in Round 7) runs against
  this round's binary at 58 cases, 433
  steps, 0 failures, the flat shape a
  round that changes no source owes as
  evidence it touched nothing
  behavioral.
- WW-64. README records 8 of 100 only
  in the closure commit written after
  this round's three reviews, fixes,
  and owner verification.
- WW-65. The red team's one Minor in
  this round's reviews, and its repair.
  Two shipped behaviors stood nowhere
  in the README. Grade carries
  --supersedes, the correction path,
  and the three views carry -o and
  --output, writing the document to a
  named file instead of printing it.
  The tally capture the page printed
  carries a superseded count, and no
  command the page documented could
  produce a superseded confession.
  The repair stands in the closure
  amendments commit, after the three
  reviews, and extends Usage with both
  behaviors, each demonstrated against
  this round's binary over the Usage
  record itself. A second grade on the
  graded deploy confession refuses
  with the error naming the outcome a
  correction must supersede. The same
  grade with --supersedes lands and
  prints the new outcome's id. The
  tally then counts the confession
  once, under its corrected grade, and
  the replaced outcome counts nowhere.
  The superseded count stands at zero
  in that tally, exactly as the binary
  prints it. That count is the WW-10
  state, a confession superseded while
  ungraded, which a later claim in the
  record brings about and no command
  in this tool writes (WW-8b). A
  ledger run with -o prints nothing
  and writes the document to the named
  file, and the README shows the
  file's opening lines as the capture,
  framed in the prose as the file's
  opening. Every new capture is byte
  identical to the run that produced
  it, verified against the captured
  files. The rounds line moves only
  in the closure commit (WW-64).
- WW-66. CodeRabbit's review of this
  round's pull request (PR 11) posted
  four findings, and this item
  disposes of them. It lands after
  the closure commit by the round's
  sequence, and the rounds line the
  closure set is untouched and still
  exact. Two findings are dismissed
  with their reasons and no edit. The
  Install block's scratch CARGO_HOME
  and its session PATH entry stand as
  the fleet's uniform self-contained
  verification form, verified
  verbatim in that form by this
  round's reviews. The block performs
  the default registry behavior, and
  a reader with their own Cargo
  configuration can run the same
  commands under it. The PATH point
  is the shape already dismissed in
  flighttape's FT-87 and unheard's
  UH-56. Two findings are repaired.
  The Quickstart copies the sample
  fixture over record.jsonl, and the
  lane demonstrated with a sentinel
  record that the copy replaces the
  file silently, with no prompt. The
  Quickstart lead-in now says the
  copy replaces any record.jsonl
  already in the directory, and that
  a record holding confessions of
  your own belongs under a different
  name before the Quickstart is run
  again. The last finding pressed
  what --now controls, and the lane
  demonstrated the answer against
  this round's binary over the Usage
  record. A ledger run at
  2026-06-02T00:00:00Z, before the
  grade's own timestamp of
  2026-06-03T15:00:00Z, still reports
  the deploy confession graded, so
  the views read the whole record as
  it stands, and --now does not make
  a view a snapshot of the record at
  that time. The ungraded migrate
  confession, due 2026-06-05, reads
  awaiting at 2026-06-02 and at
  2026-06-04 and overdue at
  2026-06-06, so --now sets only the
  moment an ungraded confession is
  judged overdue against. The ledger
  lead-in in Usage now states both
  halves in one sentence.
- WW-67. The CodeRabbit disposition owed
  from Round 8 discharges here on
  evidence, read back in full in this
  round (checked 2026-11-08). The Round 8
  pull request (PR 11) carries twelve
  review comments, nine reviews, and
  four issue comments. The owner's
  trigger stands at 6718018882
  (2026-11-08T06:03:09Z); the summary
  shell (6718019962) posted at
  06:03:17Z and stands in completed
  walkthrough form, with no Review
  failed marker, last updated
  2026-11-08T06:19:18Z, before the
  06:26:24Z merge at
  d3ed8422ff071322d6ed745b657eba94e430a239.
  The one review carrying findings
  (5816856733, COMMENTED, 06:09:10Z)
  posted the four findings at
  06:09:08Z, and the remaining
  shell-side comment (6718072325,
  06:09:26Z) is the action notice,
  Already reviewed the last commit.
  The owner's four replies (4505271922,
  4505272059, 4505272266, 4505272460,
  from 06:19:46Z to 06:20:00Z) disposed
  of the findings as WW-66 records.
  Four CodeRabbit acknowledgment
  replies followed on the four threads,
  all before the merge: 4505272581
  (06:20:03Z) accepts the CARGO_HOME
  dismissal, 4505272885 (06:20:11Z)
  withdraws the PATH finding, 4505273092
  (06:20:18Z) accepts the --now repair,
  and 4505273164 (06:20:21Z) accepts the
  Quickstart warning, noting the warning
  informs rather than prevents, which
  restates that repair's own terms and
  raises no new claim. The remaining
  eight reviews are the empty COMMENTED
  shells the replies ride. No second
  shell posted, nothing arrived after
  the merge but the owner's landing
  comment (6718221203, 06:26:55Z), and
  no finding stands undisposed. Any
  late findings on this round's pull
  request are owed to Round 10's
  contract by the same shape.
- WW-68. The oracle's posture at this
  seam, from evidence. The Python
  writes its temp through
  tempfile.mkstemp (the oracle's
  _write_atomically), which draws each
  candidate name from tempfile's
  _RandomNameSequence: eight characters
  per candidate, each a fresh draw from
  a random.Random held for the process.
  A Random constructed with no seed
  argument is seeded from the operating
  system's randomness source when one
  is available, which is the seed
  contract the interpreter on this box
  (3.12.3) states in Lib/random.py, and
  the sequence's own docstring in
  Lib/tempfile.py calls its strings
  unpredictable. No input an observer
  can hold (the process id, the time,
  the directory, the name) determines
  any candidate, because the seed is
  operating system entropy and is never
  exposed. The port's draw (WW-48) is
  the opposite shape: a public function
  of inputs an observer can know or
  fix, rescued from the process id
  alone by a clock term that a frozen
  and known clock stands still, on
  either side of the epoch (WW-59).
  The threat WW-59 left open is the
  whole distance between the two
  postures at this seam.
- WW-69. The adjudication, and the
  repair. WW-59's question is answered
  yes: operating system randomness
  enters the hash mix. temp_suffix
  gains a seventh term, sixteen bytes
  drawn fresh for each candidate from
  /dev/urandom, the kernel's randomness
  interface, read through the standard
  library with no new dependency, and
  appended to the hash input after the
  name's bytes. The draw stays per
  candidate, as mkstemp's does: every
  candidate the retry loop tries
  carries its own term, so the
  candidate set is not a function of
  anything an observer can hold, on
  either side of the epoch and under a
  frozen and known clock. Sixteen
  bytes stands far beyond the suffix's
  own width, so the set's
  uncomputability rests on this term
  alone. The failure posture is
  refusal, never fallback: where the
  entropy read fails or comes up
  short, and where the crate has no
  operating system randomness source
  to read (its non-unix arm),
  temp_suffix yields no suffix and
  write_atomically refuses through
  its standing arm, the cannot-write
  line at exit 1, before any create,
  so no computable mix is ever drawn
  silently (the WW-57 lesson). The
  observable law is otherwise
  untouched: the candidate is the
  name, a dot, and exactly eight
  lowercase hexadecimal characters
  from the first four bytes of the
  hash; the create stays exclusive
  with redraw on AlreadyExists inside
  the standing 64-attempt budget; the
  mode stays 0600; the temp is
  removed on any failure; placement
  (WW-42) and construction (WW-36)
  stand. What the law now claims: the
  candidate set cannot be computed
  from the process id, nor from the
  process id and a frozen and known
  clock together. What it still does
  not claim: the suffix carries 32
  bits of the hash, so a planted name
  can still collide by chance, once
  in 2^32 per candidate on average,
  and the redraw budget, not the
  draw, is what answers a collision.
- WW-70. Amendment, the evidence at the
  seam, gathered in this round before and
  after the repair. The battery is this
  lane's own (ww9-scratch/battery),
  modeled on the Round 8 red team's: the
  clock frozen by an LD_PRELOAD shim at a
  post-epoch reading (nanoseconds term
  1790000000123456789) and at a pre-epoch
  reading one nanosecond before the
  epoch (term 1, the WW-57 error-branch
  magnitude), the write run at process
  id 1 in a fresh namespace, and the
  planted set computed by the pre-round
  draw's formula at that reading, all
  65 candidates the retry loop can
  draw. Against the control, a fresh
  build of this round's base tree, the
  frozen set refuses the write on both
  sides of the epoch: exit 1, the
  cannot-write line, nothing landed,
  the plants untouched; at 64 plants
  the write lands on both sides with
  output byte-identical to the oracle's
  (sha256 e2f3cbc9232c444c on every
  landed cell in this battery). The
  oracle lands at 65 plants on both
  sides with the same bytes. Against
  the repaired binary the same 65
  plants land the write on both sides,
  byte-identical output, the plants
  untouched, the shape the oracle
  shows; the post-epoch cell was
  re-run against the final formatted
  build with the same outcome. The
  entropy failure posture is
  demonstrated, not inferred: under a
  shim that fails every open of
  /dev/urandom with EACCES, the
  repaired binary refuses (exit 1,
  the standing cannot-write line) and
  creates nothing at all in the output
  directory, while the control lands
  under the same shim, never reading
  the device; those cells ran at the
  ambient process id, the refusal
  preceding any create and consulting
  neither the clock nor the id. Three
  unit pins land with the repair: the
  entropy term decides the suffix at
  fixed other terms, the suffix
  matches an independent derivation of
  the contracted mix, and successive
  live draws differ inside the
  contracted shape. A mutant that
  drops the entropy term from the mix
  fails exactly the first two pins
  and passes the third, which pins
  the shape and not the term; the
  mutant ran in a scratch copy with
  its own target directory, deleted
  after the run.
- WW-71. Amendment, the gate and the
  harness on this round's tree. cargo
  test passes 31 with 0 failed, the
  Round 8 baseline of 28 plus the
  three pins WW-70 names (3 library,
  28 integration); cargo clippy
  --all-targets with warnings denied
  is clean, re-run after formatting;
  cargo fmt --check is clean; the
  Python codepoint scan over all 22
  tracked files returns U+2014=0 and
  U+2013=0. The standing differential
  harness (the goals-tree copy,
  WW_PY_SRC at the renamed oracle
  snapshot) runs at 58 cases, 433
  steps, 0 failures against this
  round's binary, and at the same
  shape against the control. No
  harness cell's expectation changed:
  its planted cells plant the WW-47
  decimal candidates, which neither
  draw produces, and the suffix
  boundary, width, and mode cells
  are untouched by a term that feeds
  only the hash mix. The behavior
  that changed is exercised by the
  WW-70 battery alone, and it is
  stated there.
- WW-72. README records 9 of 100 only
  in the closure commit written after
  this round's three reviews, fixes,
  and owner verification.
- WW-73. Amendment, the CodeRabbit
  disposition on PR 12, written after
  the closure commit; the rounds line
  stands untouched at 9 of 100. The
  review's one finding (4505661668,
  Minor, on the third pin WW-70
  names) makes three points, each
  verified against the code as
  written before any repair. First,
  the pin unwrapped the live draw
  with expect, so on a non-unix
  build, whose entropy arm yields
  None on every call by WW-69's
  design, and wherever /dev/urandom
  cannot be read, the pin panicked
  where the contracted outcome is
  the refusal itself. Second, the
  pin required two successive draws
  to differ, which the contract
  nowhere promises: the suffix
  carries thirty-two bits of the
  hash, so two valid draws can
  collide, once in 2^32 per pair on
  average, and the requirement was
  a flake source in principle.
  Third, its suggestions follow
  from the two. Repair, in test code
  only, no production behavior
  changed: the pin is reworked and
  renamed
  a_live_draw_renders_eight_hex_characters_or_refuses,
  and it draws live over fixed terms
  and asserts, for each draw the
  entropy source answers, the
  contracted shape, exactly eight
  lowercase hexadecimal characters,
  and where the draw yields None it
  accepts the contracted refusal
  rather than panicking, so the pin
  asserts what the contract promises
  on every platform. The refusal
  path itself stays covered where
  it was covered already, the WW-70
  battery's entropy-failure cells
  and the red team's shim cells; no
  injection seam was added to
  production code to unit-test it.

- WW-74. The CodeRabbit disposition owed
  from Round 9 discharges here on
  evidence, read back in full in this
  round (checked 2026-11-09). The Round 9
  pull request (PR 12) carries three
  reviews, three review comments, and
  four issue comments. WW-73 disposed
  the review's one inline finding
  (4505661668, Minor), and the bot's
  acknowledgment (4505686611) accepted
  the repair and resolved the thread.
  Two items stand in the same review
  that WW-73's enumeration does not
  name, and this item is their record.
  The first is the Nitpick standing in
  review 5817299310's body (src/cli.rs
  889-898), asking for regression tests
  that inject a failed and a short
  entropy read through write_atomically
  and assert the write fails before any
  create. It is answered on the ground
  WW-73's own closing paragraph records:
  no injection seam was added to
  production code to unit-test the
  refusal, and the refusal path stands
  covered where it was covered already,
  the WW-70 battery's entropy-failure
  cells and the red team's shim cells,
  which run the shipped binary against
  reads that fail and reads that
  truncate. WW-77 re-runs that posture
  at this round's binary. The second is
  the failed Docstring Coverage check
  standing in the same review's summary
  and in the summary shell's pre-merge
  checks (6719567763): coverage 60.00
  percent against a threshold of 80.00
  percent, over five functions across
  one file, none of them named. It is
  dismissed on the ground this line
  states for the same check wherever it
  recurs: it names no function, the
  repository carries no docstring
  configuration or threshold of its
  own, and the check applies the review
  tool's default profile, so there is
  no repo rule for it to enforce. No
  reply thread exists for a review-body
  item; this item is its record. The
  remaining contents of the pull request
  are the owner's trigger, the summary
  shell in rate-limit-banner form over
  its complete walkthrough, the action
  notice 6719636992 (Already reviewed
  the last commit, carrying no finding),
  and the owner's landing comment. The
  finishing-touches and autofix
  checkboxes are standing offers, not
  findings. Nothing else stands
  undisposed. Any late findings on this
  round's pull request are owed to
  Round 11's contract by the same shape.

- WW-75. WW-5 restated, so the base
  claim stands in this contract alone.
  A confession is an onrecord claim, and
  the claim's full field set is this. The
  record is an append-only JSON lines
  file; each line is one claim object.
  The required fields are id, text,
  and asserter, each a non-empty
  string; asserted_at, an ISO
  timestamp string the caller supplies,
  the store never inventing a time;
  evidence, a list of at least one
  entry, each entry an object whose
  source and reference are non-empty
  strings; and supersedes, a claim id
  or null. The optional fields are
  subject and due; either may stand
  null where it stands, a subject
  otherwise is a string, and due
  otherwise is an ISO date string.
  Fields beyond these are part
  of the claim and are kept verbatim.
  The id is the word claim, a dash, and
  the first sixteen hexadecimal
  characters of the SHA-256 digest,
  under canonical form 1, of an object
  holding the claim without its id
  under the key claim and its position
  under the key position, the position
  the count of lines already in the
  record; a read recomputes every id
  and flags a mismatch. Two chain laws
  ride on supersedes: a supersedes
  naming no claim in the record is a
  defect, and a claim superseded by
  two claims is a defect, each named
  at read time like the rest. A line that fails this
  shape, a missing or mistyped required
  field among it, is a named defect at
  read time with its line number, and
  every read on a record holding
  defects refuses, naming them. The
  enforcement is the onrecord crate's,
  consumed at its pin (WW-50), not
  forked: this repo's storage format is
  that record format, which is the
  coupling WW-6a states, and it is a
  coupling of format, not of reference
  chasing; no reader of this contract
  needs onrecord's open beside it to
  know a claim's fields. Over that
  shape tetlock lays its own reading,
  WW-5, WW-5a, WW-5b, and WW-7a
  together: a confession carries kind
  with the value confession; its text
  is the sentence, under the sentence
  rules; its subject names the action
  it predicts; its due states when the
  outcome should be known; its evidence
  holds at least one entry whose source
  is approval, naming the WW-1 approval
  in its reference. An outcome carries
  kind with the value outcome, a grade
  from the fixed set (held, broke,
  unverifiable), a subject of the word
  confession, a colon, and the
  confession's claim id, and a text
  stating what actually happened. The
  two layers fail differently, and the
  difference is part of the law: a
  claim failing the record shape stops
  every read, while a confession
  standing in the record shape but
  breaking the confession reading is
  read and named, in the ledger's
  malformed list with its reasons,
  counted in the tally's malformed
  count, and refused by grade; a live
  outcome carrying a grade outside the
  fixed set refuses the views outright,
  naming the grade.

- WW-76. The differential proof of the
  restatement. The loader's accept and
  reject verdicts and the three views'
  documents stand compared byte for byte
  between this round's binary and the
  Python oracle over fifty-three
  hand-minted field cells in this
  round's own battery, each one change
  from a valid confession or a valid
  outcome, every id derived by the
  oracle from the defective claim
  itself, so each cell carries exactly
  the defect it names: each required
  field missing and mistyped, the
  evidence list empty, malformed, and
  carrying an extra key, the optional
  fields missing, mistyped, and
  unparsable, fields beyond the set
  kept, the sentence rules broken four
  ways, a missing id, a forged id, the
  optional fields standing null, two
  claims superseding one target, a bad
  grade on an outcome superseded by an
  outcome and on one superseded by a
  plain claim, raw
  lines that are not JSON, not an
  object, and empty, and grade refused
  a malformed confession. Forty further
  cases spliced into the standing
  differential harness carry the same
  variants through its seedmint form.
  Every cell agrees on both
  instruments: the fields WW-75
  restates are the fields the loader
  enforces, in both layers, a
  record-shape defect refusing every
  read on both sides with the same
  named defects, and a
  confession-reading defect surfacing
  in the same malformed lists, the
  same tally count, and the same grade
  refusal. The two implementations
  diverge nowhere in this battery.
  The restatement's own first wording
  under-stated the law at four points:
  the non-empty strings, the standing
  nulls, the chain laws and the
  digest's object, and the live
  outcome. The review found them, six
  further cells prove the amended
  text, and the repairs stand recorded
  on the pull request. No behavior
  changed in this round.

- WW-77. The grading and wall surfaces
  measured fresh at this round's
  binary, seventeen cells in the same
  battery, every cell agreeing with the
  oracle. The due-date boundary: a
  confession stands awaiting on the day
  before its due date, awaiting at the
  due date's first instant and at its
  last, and overdue from the following
  date, in the ledger's states and the
  tally's counts alike; the boundary is
  the date, as WW-8a states, never the
  instant inside it. The wall's week:
  an outcome graded broke, asserted at
  the week's first instant and at its
  last, stands on the wall for a date
  inside that week, and one asserted
  the instant before the week or the
  instant after it does not; a
  confession due on the week's Monday
  and one due on its Sunday stand in
  the ungraded list, and one due the
  day before the week or the day after
  it does not. The wall is a view:
  across every cell of both instruments
  in this round, and through view
  writes named by -o as well as views
  to the terminal, the record's bytes
  after the views equal its bytes
  before them. The confession flow end
  to end: confess mints a claim whose
  field set is exactly the set WW-75
  restates plus kind, and whose id
  recomputes from the claim without it.
  The draw's refusal posture, re-run
  under the WW-70 entropy shim against
  a view write through -o: the write
  refuses with the cannot-write line,
  creates nothing, and leaves the
  record untouched; the oracle under
  the same shim writes, its name draw
  never refusing, which is the
  asymmetry WW-69 contracted, the
  refusal being this repo's law laid
  over the oracle's shape, not a
  divergence. No conviction stands
  under this item either.

- WW-78. The gate and the harness on
  this round's tree. Cargo test runs
  31 tests, all passing; clippy over
  all targets with warnings denied is
  clean; cargo fmt --check is clean;
  the codepoint scan over the tracked
  tree reports no em dashes and no en
  dashes. The local gate ran under the
  box's file-count quota, which refused
  the test-profile build's new files in
  bursts (os error 122); the gate
  completed on a patient retry loop,
  the tests on the ninth attempt and
  clippy on the fourth, both against
  the tree as it stands, and the
  counts above are from those completed
  runs. The standing differential
  harness, extended in place under
  WW-76, runs 98 cases and 591 steps
  with no failures against this round's
  binary; its temp root stood outside
  the workspace for the same quota
  reason, a relocation that changes no
  case and no expectation. This round's
  own battery stands where WW-76 and
  WW-77 record it: 74 cells, no splits.

- WW-79. README records 10 of 100 in
  this round's closure commit, the
  rounds line moving only there.

- WW-80. The CodeRabbit disposition owed
  from Round 10 discharges here on
  evidence, read back in full in this
  round (checked 2026-11-11). The Round 10
  pull request (PR 13) carries thirteen
  reviews, seventeen review comments, and
  four issue comments. The one review
  carrying findings (5835014264) posted
  five Minors on the WW-75 restatement
  and the WW-77 wall sentence, and the
  owner's five replies disposed of them
  as the pull request body records: the
  five findings are fixed in 7ce9a663,
  each proved by a battery cell, and one
  fragment of the first finding was not
  adopted, its wording describing
  asserted_at as an ISO date where the
  pinned loader reads an ISO timestamp.
  The bot's follow-up acknowledgment on
  that thread (4519896404) accepts the
  correction and raises one further true
  omission in its place, the subject type
  law, fixed in 433e3931 with its own
  cell (the owner's reply 4519966303) and
  accepted in the bot's closing
  acknowledgment (4519968804), which
  resolves the thread. The remaining
  four threads end in the bot's
  acknowledgments accepting each fix
  (4519892315, 4519892749, 4519893614,
  4519894323), each marked resolved. The
  summary shell (6755515119) stands in
  its final edited form at 10,623
  characters: a rate-limit banner over a
  complete walkthrough, a Low merge-risk
  line whose coverage stands at the slice
  head 2602c718 while its commits line
  reads through the final head 433e3931,
  four pre-merge checks all passing
  (Description, Title, Linked Issues,
  Out of Scope Changes), two files
  selected (CONTRACT.md and README.md),
  an effort of 1 (Trivial) at about five
  minutes, and the shell's run
  2d8c31e7-7a6a-4301-a196-3864376a332a.
  The one remaining shell-side comment
  (6755695394) is the rate-limited action
  notice answering the owner's trigger
  (6755515427), carrying no finding; the
  owner's landing comment (6756698890)
  closes the pull request. No Round 10
  disposition is falsified by the
  follow-ups or the shell's final form:
  the bot's acknowledgments confirm each
  repair, its one new catch was repaired
  inside Round 10 and stands in WW-75's
  final text, and the Docstring Coverage
  ground WW-74 states for the recurring
  check was not tested on this pull
  request, whose review posted no such
  item. Any late findings on this
  round's pull request are owed to
  Round 12's contract by the same shape.

- WW-81. Fresh measurement on the round's
  named surfaces, in this round's own
  battery, fifty-seven cells, every cell
  decided at its exact point on both
  implementations (this round's binary
  and the final Python oracle, whose
  pairing WW-56 records; the oracle
  snapshot was re-verified this round
  byte for byte against the tree at
  e0244e88, the parent of the port commit
  2b07909d, with the staged renamed copy
  differing in exactly the four rename
  lines). The grading and banding law,
  twenty-three cells. The due-date
  boundary is the date, as WW-8a states:
  a confession due 2026-09-17 stands
  awaiting at the day before's last
  instant, at the due date's first
  instant (the exact point itself), at
  its midday, and at its last instant,
  and overdue from the following date's
  first instant, in the ledger's states
  and the tally's counts alike; three
  offset spellings of the same instants
  land on the same UTC dates (WW-9b), and
  a leap-day due date (2024-02-29) stands
  awaiting on the leap day and overdue
  on 2024-03-01. The wall's week edges
  are exact (WW-9a): a broke outcome
  asserted at the week's first instant,
  its last instant, and its last
  fractional instant stands on the wall
  for 2026-W38, and one asserted the
  instant before the week or the instant
  after it does not; confessions due on
  the week's Monday and Sunday stand in
  the ungraded list, and one due the day
  before or the day after does not; the
  wall's own date argument in its ISO
  week form and its basic calendar form
  selects the same week. One confession
  per grade fills each tally band, and a
  correction superseding the broke
  outcome moves its confession to the
  held band with the replaced outcome
  counting nowhere. The draw, eleven
  cells, re-probed with a positive
  control bracketed at both ends: a
  plain view write lands on this round's
  binary, byte-identical to the oracle's
  (the first bracket); under the WW-70
  entropy shim the write refuses with
  the cannot-write line, creates
  nothing, and leaves the record
  untouched, and under a four-byte
  entropy source bound over the device
  in a mount namespace, where the open
  succeeds and the sixteen-byte read
  comes up short, the same refusal
  stands on all four of its checks; the
  oracle under the failing shim writes,
  the WW-69 asymmetry standing as
  contracted; and the plain write lands
  again after the shim cells (the
  closing bracket). The resolver and
  walk constants, eleven cells, measured
  at the shipped binary and never
  inherited: head chains away from the
  record write through 990 and 991
  splices and refuse with the guard
  phrase at 992 and 993, so the constant
  re-derives at 991 nested splices
  resolving and the 992nd failing
  (WW-33); head chains onto the record
  refuse at both counts, at 991 because
  the resolved output is the record and
  at 992 under the walk's failure
  posture (WW-34); prefix chains at 991
  resolve at the guard and fail at the
  write on both sides with the
  cannot-write phrase, the kernel cap
  posture WW-37 records, and at 992 take
  the guard's refusal; the oracle writes
  at 991 away and crashes uncaught at
  992, the named deviation class. The
  pins, twelve cells: the flighttape pin
  (74a83298) and the onrecord pin
  (9e41a7be) stand identically in
  Cargo.toml and Cargo.lock, and each
  pin's tree was re-verified blob by
  blob against the remote, flighttape
  at fifty blobs and onrecord at twenty,
  every blob fetched and its git blob
  identity recomputed from its bytes
  with no mismatch; one mutation per pin
  family (a single hex character of one
  blob identity flipped in the manifest
  copy) is convicted by the pin check,
  and the restored manifest passes
  again. No cell diverges and no
  behavior changed in this round.

- WW-82. A realistic pass: realistic data
  at real shapes and volumes, synthetic
  in content and real in shape, run
  through the shipped binary's surfaces
  and twinned against the oracle, seven
  cells, no divergences. The record
  holds 240 confessions across the
  twenty-six ISO weeks 2026-W02 through
  2026-W27, each naming one of sixteen
  operations actions, a one-sentence
  prediction, a due date inside its
  week, and an approval reference, minted
  through the oracle's own id derivation
  under a fixed seed. Its parts: 200 of
  the confessions stand graded (130
  held, 48 broke, 22 unverifiable), 6
  stand superseded while ungraded by
  plain claims later in the record, 4
  stand malformed by the confession
  reading (no approval evidence), and
  the remaining 30 ungraded well-formed
  confessions carry due dates in the
  record's final weeks, standing
  awaiting at the pass's two earlier
  stated times (2026-03-02 and
  2026-05-11) and overdue at its last
  (2026-07-06); the record's 450 lines
  are the 240 confessions, the 200
  outcomes, the 6 superseding plain
  claims, and 4 standing notes. Thirty-
  four view steps (the ledger and the
  tally at the three stated times, the
  wall for each of the twenty-six weeks,
  and one ledger and one tally written
  through -o) agree byte for byte on
  both implementations, the trees after
  the views are identical, and the
  record's bytes stand untouched by
  them. The tally's arithmetic closes
  from the document itself: its total
  is the 236 well-formed confessions
  (240 minted less the 4 malformed, the
  malformed count standing apart as
  WW-14 states), the state counts sum
  to the total (200 graded, 30 overdue,
  6 superseded, 0 awaiting at the last
  stated time), the grade counts sum to
  the graded count (130, 48, and 22),
  and the sixteen action rows' grade
  counts sum to the document's. The
  write path runs fresh through both
  implementations: twelve confessions
  minted by the confess command and six
  grades landed by the grade command,
  with the closing tally, nineteen steps
  whose streams and records agree byte
  for byte. Every total in this item
  traces from the parts stated above.

- WW-83. The gate and the harness on
  this round's tree. Cargo test runs
  31 tests, all passing (3 library, 28
  integration), the Round 9 baseline
  held on a tree that changes no source;
  clippy over all targets with warnings
  denied is clean; cargo fmt --check is
  clean; the Python codepoint scan over
  the tracked tree reports no em dashes
  and no en dashes. The standing
  differential harness, extended in
  place under WW-76, runs 98 cases and
  591 steps with no failures against
  this round's binary; its temp root
  stands outside the workspace under
  the same quota WW-78 records, a
  relocation that changes no case and
  no expectation. This round's own
  battery stands where WW-81 records
  it: 57 cells, no splits; the
  realistic pass stands where WW-82
  records it: 7 cells, no divergences.
  The local work ran under the box's
  file-count quota and write fault at
  their hardest: the workspace refused
  every write (os error 122), so the
  lane's tree, build target, and battery
  staged outside it, and the release
  build and gate completed on a patient
  retry loop once a write window opened,
  the counts above from those completed
  runs; hosted CI stands as the gate of
  record on the pushed head.

- WW-84. README records 11 of 100 in
  this round's closure commit, the
  rounds line moving only there.

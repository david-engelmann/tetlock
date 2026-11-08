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

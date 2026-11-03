//! The Store adapter (WW-18). The record itself is
//! onrecord's: claim_defects, derive_id, and
//! load_record are the onrecord crate's functions,
//! consumed pinned. What stands here is this repo's
//! own surface over them: claims, add with the
//! extra field rule, and check, with this repo's
//! own append, mirroring the Python record module
//! line for line.

use flighttape::value::{Text, Value};
use onrecord::store::{claim_defects, derive_id, load_record, RecPath};

use crate::util::{self, Msg};

/// A named refusal, or the crash class: an input on
/// which the Python dies with an uncaught exception
/// meets a silent exit 1 here (named in the
/// dossier).
#[derive(Debug)]
pub enum StoreError {
    Named(Msg),
    Crash,
}

impl StoreError {
    pub fn named(text: &str) -> StoreError {
        StoreError::Named(Msg::str(text))
    }
}

fn conv(message: &onrecord::util::Msg) -> Msg {
    Msg(message.0.clone())
}

/// The record path's display spelling in this
/// crate's message type.
pub fn display_msg(path: &RecPath) -> Msg {
    Msg(path.display.0.clone())
}

fn conv_all(messages: &[onrecord::util::Msg]) -> Vec<Msg> {
    messages.iter().map(conv).collect()
}

/// The render thresholds for this repo's entries
/// (WW-19), probed per command at the shipped CLI.
/// The Python's claim check derives each id by
/// rendering the claim under canonical form 1 with
/// a recursive renderer, so a deeply nested claim
/// dies on the interpreter recursion limit and is
/// reported as a claim that cannot be rendered;
/// where the limit bites depends on the calling
/// surface's stack. The views through the shared
/// preamble (ledger, tally) sit one frame deeper
/// than the rest. The thresholds are in the cost
/// units of render_fails below: the shared
/// preamble's is 987, every other entry's is 988.
/// The crate's own depth budgets cannot express
/// the leaf costs the interpreter charges (a string
/// costs two frames for its dumps call, a float
/// one for its number helper, a leaf under an
/// object parent one more when it calls no helper
/// of its own), so the crate is always called with
/// a generous budget and the defect it should have
/// raised is restored by fix_up_defects, which
/// decides render failure with the probed model.
pub const THRESHOLD_NOW_VIEW: usize = 987;
pub const THRESHOLD_READ: usize = 988;

/// A budget deep enough that the crate's renderer
/// never refuses on depth; the depth law is this
/// module's render_fails, not the crate's budget.
const GENEROUS: (usize, usize) = (1_000_000, 1_000_000);

pub struct Store {
    pub path: RecPath,
}

impl Store {
    pub fn new(path: RecPath) -> Store {
        Store { path }
    }

    /// pathlib's exists under the Python's law: the
    /// ignore set answers false, any other OS error
    /// propagates as its rendered phrase.
    pub fn exists(&self) -> Result<bool, StoreError> {
        match std::fs::metadata(&self.path.os) {
            Ok(_) => Ok(true),
            Err(error) => match error.raw_os_error() {
                Some(2) | Some(20) | Some(9) | Some(40) => Ok(false),
                _ => Err(StoreError::Named(util::os_error_with_path(
                    &error,
                    &display_msg(&self.path),
                ))),
            },
        }
    }

    fn require_sound(defects: &[Msg]) -> Result<(), StoreError> {
        if defects.is_empty() {
            return Ok(());
        }
        let mut message = Msg::str("the record has defects: ");
        message.push_msg(&util::join_msgs(defects, "; "));
        Err(StoreError::Named(message))
    }

    fn load(&self) -> Result<Vec<Value>, StoreError> {
        self.load_with(THRESHOLD_READ)
    }

    fn load_with(&self, threshold: usize) -> Result<Vec<Value>, StoreError> {
        if !self.exists()? {
            return Ok(Vec::new());
        }
        let (claims, defects) = load_record(&self.path, GENEROUS);
        let defects = fix_up_defects(&claims, conv_all(&defects), threshold, &self.path);
        Self::require_sound(&defects)?;
        Ok(claims)
    }

    /// The record's claims, sound or refused (the
    /// public read surface for the views).
    pub fn claims(&self) -> Result<Vec<Value>, StoreError> {
        self.load()
    }

    /// The claims read at another entry's threshold
    /// (WW-19): the ledger and the tally reach the
    /// record one frame deeper than the other
    /// commands, through the shared preamble.
    pub fn claims_at(&self, threshold: usize) -> Result<Vec<Value>, StoreError> {
        self.load_with(threshold)
    }

    pub fn check(&self) -> Result<Vec<Msg>, StoreError> {
        if !self.exists()? {
            let mut message = Msg::str("no such record, ");
            message.push_msg(&display_msg(&self.path));
            return Err(StoreError::Named(message));
        }
        let (claims, defects) = load_record(&self.path, GENEROUS);
        Ok(fix_up_defects(
            &claims,
            conv_all(&defects),
            THRESHOLD_READ,
            &self.path,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add(
        &self,
        text: &Text,
        asserter: &Text,
        asserted_at: &Text,
        evidence: &[(Text, Text)],
        supersedes: Option<&Text>,
        subject: Option<&Text>,
        due: Option<&Text>,
        extra: &[(Text, Value)],
    ) -> Result<String, StoreError> {
        let claims = self.load()?;
        if let Some(target) = supersedes {
            let known = claims.iter().any(|claim| {
                matches!(onrecord::store::field(claim, "id"),
                    Some(Value::Str(id)) if id.units() == target.units())
            });
            if !known {
                let mut message = Msg::str("supersedes names no claim in the record, ");
                message.push_units(target.units());
                return Err(StoreError::Named(message));
            }
            let already = claims.iter().any(|claim| {
                matches!(onrecord::store::field(claim, "supersedes"),
                    Some(Value::Str(id)) if id.units() == target.units())
            });
            if already {
                let mut message = Msg::str("claim ");
                message.push_units(target.units());
                message.push_str(" has already been superseded");
                return Err(StoreError::Named(message));
            }
        }
        let mut entries: Vec<(Text, Value)> = vec![
            (Text::from_str("text"), Value::Str(text.clone())),
            (Text::from_str("asserter"), Value::Str(asserter.clone())),
            (
                Text::from_str("asserted_at"),
                Value::Str(asserted_at.clone()),
            ),
            (
                Text::from_str("evidence"),
                Value::Array(
                    evidence
                        .iter()
                        .map(|(source, reference)| {
                            Value::Object(vec![
                                (Text::from_str("source"), Value::Str(source.clone())),
                                (Text::from_str("reference"), Value::Str(reference.clone())),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                Text::from_str("supersedes"),
                match supersedes {
                    Some(id) => Value::Str(id.clone()),
                    None => Value::Null,
                },
            ),
        ];
        if let Some(subject) = subject {
            entries.push((Text::from_str("subject"), Value::Str(subject.clone())));
        }
        if let Some(due) = due {
            entries.push((Text::from_str("due"), Value::Str(due.clone())));
        }
        for (key, value) in extra {
            let collides = key.units() == Text::from_str("id").units()
                || entries.iter().any(|(held, _)| held.units() == key.units());
            if collides {
                let mut message = Msg::str("extra field ");
                message.push_units(key.units());
                message.push_str(" collides with a claim field");
                return Err(StoreError::Named(message));
            }
            entries.push((key.clone(), value.clone()));
        }
        let claim = Value::Object(entries);
        let position = claims.len();
        let claim_id = match derive_id(&claim, position, GENEROUS) {
            Ok(id) => id,
            Err(_) => {
                return Err(StoreError::named(
                    "the claim cannot be rendered under canonical form 1",
                ))
            }
        };
        let mut stored_entries =
            vec![(Text::from_str("id"), Value::Str(Text::from_str(&claim_id)))];
        if let Value::Object(entries) = &claim {
            stored_entries.extend(entries.iter().cloned());
        }
        let stored = Value::Object(stored_entries);
        let defects = claim_defects(&stored, position + 1, position, GENEROUS);
        let mut defects = conv_all(&defects);
        if render_fails(&stored, THRESHOLD_READ) {
            fix_up_one(&mut defects, position + 1);
        }
        if let Some(first) = defects.first() {
            // The Python raises the first defect's tail
            // past its "line N: " head.
            let text = first.clone();
            let tail = match util::find_subslice(&text.0, &Msg::str(": ").0) {
                Some(at) => Msg(text.0[at + 2..].to_vec()),
                None => text,
            };
            return Err(StoreError::Named(tail));
        }
        self.append(&stored)?;
        Ok(claim_id)
    }

    /// The append: complete an unterminated final
    /// line, never glue, and write prefix and line
    /// in one call. The line renders after the open,
    /// where the Python's write-time failure lands,
    /// so a line the strict law refuses still leaves
    /// the created file behind (the crash class,
    /// named in the dossier). OS failures are named
    /// the way the Python names them.
    fn append(&self, stored: &Value) -> Result<(), StoreError> {
        let wrapped = |error: &std::io::Error| -> StoreError {
            let mut message = Msg::str("cannot append to the record, ");
            message.push_msg(&display_msg(&self.path));
            message.push_str(": ");
            message.push_str(&util::strerror_text(error));
            StoreError::Named(message)
        };
        let mut prefix: &[u8] = b"";
        match std::fs::metadata(&self.path.os) {
            Ok(meta) => {
                if meta.len() > 0 {
                    match read_last_byte(&self.path.os) {
                        Ok(Some(byte)) => {
                            if byte != b'\n' {
                                prefix = b"\n";
                            }
                        }
                        Ok(None) => {}
                        Err(error) => return Err(wrapped(&error)),
                    }
                }
            }
            Err(error) => match error.raw_os_error() {
                Some(2) | Some(20) | Some(9) | Some(40) => {}
                _ => return Err(wrapped(&error)),
            },
        }
        use std::io::Write;
        let mut file = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path.os)
        {
            Ok(file) => file,
            Err(error) => return Err(wrapped(&error)),
        };
        let mut line = match crate::emit::dumps_line(stored, crate::emit::Mode::Strict) {
            Ok(line) => line,
            Err(_) => return Err(StoreError::Crash),
        };
        line.push(b'\n');
        let mut bytes = Vec::with_capacity(prefix.len() + line.len());
        bytes.extend_from_slice(prefix);
        bytes.extend_from_slice(&line);
        if let Err(error) = file.write_all(&bytes) {
            return Err(wrapped(&error));
        }
        if let Err(error) = file.flush() {
            return Err(wrapped(&error));
        }
        Ok(())
    }
}

fn read_last_byte(path: &std::path::Path) -> std::io::Result<Option<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)?;
    if file.seek(SeekFrom::End(-1)).is_err() {
        return Ok(None);
    }
    let mut buf = [0u8; 1];
    match file.read_exact(&mut buf) {
        Ok(()) => Ok(Some(buf[0])),
        Err(_) => Ok(None),
    }
}

const CANNOT_RENDER: &str =
    "the claim cannot be rendered under canonical form 1, so its id cannot be verified";
const ID_MISMATCH: &str = "id does not match its derivation";

/// Text the canonical renderer can encode: no lone
/// surrogates (the Python's UTF-16BE sort key and
/// its UTF-8 encode both refuse them).
fn encodable(text: &Text) -> bool {
    let units = text.units();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit) {
            if index + 1 >= units.len() || !(0xDC00..=0xDFFF).contains(&units[index + 1]) {
                return false;
            }
            index += 2;
        } else if (0xDC00..=0xDFFF).contains(&unit) {
            return false;
        } else {
            index += 1;
        }
    }
    true
}

/// Whether the Python's recursive canonical render
/// of this claim dies at a surface whose threshold
/// is `threshold`. The claim sits at depth 2 of its
/// derive payload ({"claim": ..., "position": ...}),
/// under the payload object. A leaf's cost is its
/// depth, plus 2 for a string (its dumps call costs
/// two frames), plus 1 for a float (the number
/// helper costs one), plus 1 when its parent is an
/// object and it calls no helper of its own. A
/// non-finite float or unencodable text fails at
/// any depth. Probed per surface, per leaf shape,
/// and per parent shape at the shipped CLI (WW-19).
fn render_fails(claim: &Value, threshold: usize) -> bool {
    node_fails(claim, 2, true, threshold)
}

fn node_fails(value: &Value, depth: usize, parent_object: bool, threshold: usize) -> bool {
    match value {
        Value::Null | Value::Bool(_) | Value::Int(_) => {
            depth + usize::from(parent_object) > threshold
        }
        Value::Float(number) => !number.is_finite() || depth + 1 > threshold,
        Value::Str(text) => !encodable(text) || depth + 2 > threshold,
        Value::Array(items) => {
            if items.is_empty() {
                depth + usize::from(parent_object) > threshold
            } else {
                items
                    .iter()
                    .any(|item| node_fails(item, depth + 1, false, threshold))
            }
        }
        Value::Object(entries) => {
            if entries.is_empty() {
                depth + usize::from(parent_object) > threshold
            } else {
                entries.iter().any(|(key, item)| {
                    !encodable(key) || node_fails(item, depth + 1, true, threshold)
                })
            }
        }
    }
}

/// A defect's line number and the offset where its
/// phrase starts, when it carries the "line N: "
/// head.
fn defect_line(units: &[u16]) -> Option<(usize, usize)> {
    let head = Msg::str("line ").0;
    if !units.starts_with(&head) {
        return None;
    }
    let mut index = head.len();
    let mut number: usize = 0;
    let mut digits = 0;
    while index < units.len() && (b'0' as u16..=b'9' as u16).contains(&units[index]) {
        number = number * 10 + (units[index] - b'0' as u16) as usize;
        index += 1;
        digits += 1;
    }
    if digits == 0 || index + 1 >= units.len() {
        return None;
    }
    if units[index] != b':' as u16 || units[index + 1] != b' ' as u16 {
        return None;
    }
    Some((number, index + 2))
}

fn tail_is(units: &[u16], start: usize, phrase: &str) -> bool {
    units[start..] == Msg::str(phrase).0[..]
}

fn tail_starts(units: &[u16], start: usize, phrase: &str) -> bool {
    units[start..].starts_with(&Msg::str(phrase).0)
}

fn tail_ends(units: &[u16], start: usize, phrase: &str) -> bool {
    units[start..].ends_with(&Msg::str(phrase).0)
}

fn tail_contains(units: &[u16], start: usize, phrase: &str) -> bool {
    util::find_subslice(&units[start..], &Msg::str(phrase).0).is_some()
}

/// The chain-level entries the loader appends after
/// every per-line entry: a duplicated id, a dangling
/// supersession, a forked supersession.
fn is_chain_entry(units: &[u16], start: usize) -> bool {
    (tail_starts(units, start, "id ") && tail_ends(units, start, " appears twice"))
        || tail_starts(units, start, "supersedes names no claim in the record, ")
        || (tail_starts(units, start, "claim ")
            && tail_contains(units, start, " is superseded by two claims, "))
}

/// The record's line count under the loader's own
/// split (lines are newline-separated; one trailing
/// empty segment is not a line).
fn line_count(path: &RecPath) -> Option<usize> {
    let raw = std::fs::read(&path.os).ok()?;
    if raw.is_empty() {
        return Some(0);
    }
    let newlines = raw.iter().filter(|byte| **byte == b'\n').count();
    if raw.last() == Some(&b'\n') {
        Some(newlines)
    } else {
        Some(newlines + 1)
    }
}

/// The Python attempts the id derivation only
/// under a gate: every required field present and
/// the id a string (record.py claim_defects). With
/// the gate closed there is no derivation and no
/// failure to restore, however deep or strange the
/// claim. The add path's stored claim is built
/// whole, so its gate is always open; the loaded
/// record's claims must be gated one by one.
fn derivation_attempted(claim: &Value) -> bool {
    let entries = match claim {
        Value::Object(entries) => entries,
        _ => return false,
    };
    let field = |name: &str| {
        let wanted = Text::from_str(name);
        entries
            .iter()
            .find(|(key, _)| key.units() == wanted.units())
            .map(|(_, value)| value)
    };
    if !matches!(field("id"), Some(Value::Str(_))) {
        return false;
    }
    ["text", "asserter", "asserted_at", "evidence", "supersedes"]
        .iter()
        .all(|name| field(name).is_some())
}

/// Restore, in one claim's own defect list (the add
/// path), the cannot-be-rendered entry the Python
/// raises where the crate's generous budget derived
/// the id happily.
fn fix_up_one(defects: &mut Vec<Msg>, line: usize) {
    let cannot = Msg::str(&format!("line {line}: {CANNOT_RENDER}"));
    if defects.iter().any(|defect| defect.0 == cannot.0) {
        return;
    }
    let mismatch = Msg::str(&format!("line {line}: {ID_MISMATCH}"));
    defects.retain(|defect| defect.0 != mismatch.0);
    defects.push(cannot);
}

/// Restore the cannot-be-rendered entries in a
/// loaded record's defect list. The crate is called
/// with a generous budget (see GENEROUS), so its
/// derive renders claims the Python's renderer
/// cannot; for each claim the probed model calls
/// unrenderable, the line's derive entry (an id
/// mismatch, or silence when the id happens to
/// match) becomes the cannot-be-rendered entry, in
/// the position the Python's own check leaves it:
/// last among the line's per-line entries, which
/// all precede the loader's chain entries.
fn fix_up_defects(
    claims: &[Value],
    mut defects: Vec<Msg>,
    threshold: usize,
    path: &RecPath,
) -> Vec<Msg> {
    if claims.is_empty() {
        return defects;
    }
    let total = match line_count(path) {
        Some(total) => total,
        None => return defects,
    };
    // The lines that produced no claim object are
    // exactly the ones carrying a parse-level entry.
    let mut excluded: Vec<usize> = Vec::new();
    for defect in &defects {
        if let Some((line, start)) = defect_line(&defect.0) {
            let units = &defect.0;
            if tail_is(units, start, "blank line")
                || tail_is(units, start, "nesting too deep to read")
                || tail_is(units, start, "not JSON")
                || tail_is(units, start, "not a claim object")
                || tail_starts(units, start, "duplicate key ")
            {
                excluded.push(line);
            }
        }
    }
    let claim_lines: Vec<usize> = (1..=total)
        .filter(|line| !excluded.contains(line))
        .collect();
    if claim_lines.len() != claims.len() {
        return defects;
    }
    let mut segment_end = defects.len();
    for (index, defect) in defects.iter().enumerate() {
        if let Some((_, start)) = defect_line(&defect.0) {
            if is_chain_entry(&defect.0, start) {
                segment_end = index;
                break;
            }
        }
    }
    for (claim, &line) in claims.iter().zip(&claim_lines) {
        if !derivation_attempted(claim) || !render_fails(claim, threshold) {
            continue;
        }
        let cannot = Msg::str(&format!("line {line}: {CANNOT_RENDER}"));
        if defects[..segment_end]
            .iter()
            .any(|defect| defect.0 == cannot.0)
        {
            continue;
        }
        let mismatch = Msg::str(&format!("line {line}: {ID_MISMATCH}"));
        if let Some(at) = defects[..segment_end]
            .iter()
            .position(|defect| defect.0 == mismatch.0)
        {
            defects.remove(at);
            segment_end -= 1;
        }
        let at = (0..segment_end)
            .find(|&index| matches!(defect_line(&defects[index].0), Some((held, _)) if held > line))
            .unwrap_or(segment_end);
        defects.insert(at, cannot);
        segment_end += 1;
    }
    defects
}

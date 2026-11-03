//! The Store adapter (WW-18). The record itself is
//! onrecord's: claim_defects, derive_id, and
//! load_record are the onrecord crate's functions,
//! consumed pinned. What stands here is this repo's
//! own surface over them: claims, add with the
//! extra field rule, and check, with this repo's
//! own append, mirroring the Python record module
//! line for line.

use flighttape::value::{Text, Value};
use onrecord::store::{
    claim_defects, derive_id, load_record, RecPath, BUDGET_ADD_CHECK, BUDGET_ADD_DERIVE,
    BUDGET_READ_NARROW,
};

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

    pub fn named_msg(message: Msg) -> StoreError {
        StoreError::Named(message)
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

/// The loader budgets for this repo's entries
/// (WW-19), probed per command at the shipped CLI
/// against records holding one claim with a deeply
/// nested extra field, in (node, object) terms from
/// the digest payload's root. The views through
/// the shared preamble (ledger, tally) sit one
/// frame deeper than the rest: theirs is
/// (988, 986). Wall, grade, the load inside add,
/// and check run at (989, 987), the same value the
/// onrecord crate calls its narrow read budget.
/// The add-site render budgets are the crate's own
/// (BUDGET_ADD_CHECK, BUDGET_ADD_DERIVE), consumed
/// as probed on onrecord; the CLI builds claims
/// from scalar arguments only, so those boundaries
/// are unreachable at depth through it.
pub const BUDGET_LOAD: (usize, usize) = BUDGET_READ_NARROW;
pub const BUDGET_LOAD_DEEP: (usize, usize) = (988, 986);

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
        self.load_with(BUDGET_LOAD)
    }

    fn load_with(&self, budget: (usize, usize)) -> Result<Vec<Value>, StoreError> {
        if !self.exists()? {
            return Ok(Vec::new());
        }
        let (claims, defects) = load_record(&self.path, budget);
        let defects = conv_all(&defects);
        Self::require_sound(&defects)?;
        Ok(claims)
    }

    /// The record's claims, sound or refused (the
    /// public read surface for the views).
    pub fn claims(&self) -> Result<Vec<Value>, StoreError> {
        self.load()
    }

    /// The claims read at another entry's budget
    /// (WW-19): the ledger and the tally reach the
    /// loader one frame deeper than the other
    /// commands.
    pub fn claims_with(&self, budget: (usize, usize)) -> Result<Vec<Value>, StoreError> {
        self.load_with(budget)
    }

    pub fn check(&self) -> Result<Vec<Msg>, StoreError> {
        if !self.exists()? {
            let mut message = Msg::str("no such record, ");
            message.push_msg(&display_msg(&self.path));
            return Err(StoreError::Named(message));
        }
        let (_, defects) = load_record(&self.path, BUDGET_LOAD);
        Ok(conv_all(&defects))
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
        let claim_id = match derive_id(&claim, position, BUDGET_ADD_DERIVE) {
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
        let defects = claim_defects(&stored, position + 1, position, BUDGET_ADD_CHECK);
        if let Some(first) = defects.first() {
            // The Python raises the first defect's tail
            // past its "line N: " head.
            let text = conv(first);
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

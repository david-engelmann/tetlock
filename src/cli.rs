//! WW-12. A thin command line: confess, grade,
//! ledger, wall, tally. Failures are named errors
//! with exit 1. The parse reproduces the Python's
//! argparse behavior for this grammar byte for
//! byte: its help texts, its error shapes, and the
//! order they fire in. Help fires at the end of the
//! token that carries it; a scan error fires when
//! its token is processed and outranks a pending
//! help; extras are only reported after the
//! required check. Argument tokens are carried as
//! text in the operating system's own decoding
//! (fsdecode), so argument bytes survive as the
//! Python's do.

use std::io::Write;

use flighttape::value::{Text, Value};

use crate::emit::{dumps_view, Mode};
use crate::helptext::*;
use crate::pyrepr::repr_text;
use crate::record::{Store, StoreError};
use crate::util::{fsdecode_units, stderr_render, Msg};
use onrecord::store::RecPath;

/// One argument token: its text, and its decoded
/// scalars for the parser's character work.
struct Token {
    text: Text,
    scalars: Vec<u32>,
}

impl Token {
    fn from_units(units: Vec<u16>) -> Token {
        let scalars = crate::util::scalars_of(&units);
        Token {
            text: Text::from_units(units),
            scalars,
        }
    }

    fn rendered(&self) -> String {
        stderr_render(self.text.units())
    }
}

/// Whether a token is shaped like an option: it
/// starts with a dash, is not a bare dash, and does
/// not look like a negative number (no option in
/// this grammar does, so argparse lets those
/// through as values).
fn option_shaped(token: &Token) -> bool {
    let scalars = &token.scalars;
    if scalars.len() < 2 || scalars[0] != '-' as u32 {
        return false;
    }
    !is_negative_number(&scalars[1..])
}

/// argparse's negative number matcher, the full
/// token pattern ^-\d+$|^-\d*\.\d+$ with \d a
/// Unicode decimal digit: a token matching it
/// whole is a value, never an option.
fn is_negative_number(rest: &[u32]) -> bool {
    let digit = |c: &u32| crate::util::decimal_digit(*c).is_some();
    if !rest.is_empty() && rest.iter().all(digit) {
        return true;
    }
    match rest.iter().position(|c| *c == '.' as u32) {
        Some(dot) => {
            let (before, after) = (&rest[..dot], &rest[dot + 1..]);
            !after.is_empty() && before.iter().all(digit) && after.iter().all(digit)
        }
        None => false,
    }
}

fn scalars_of(text: &str) -> Vec<u32> {
    text.chars().map(|ch| ch as u32).collect()
}

/// Split a long-option token at its first '=': the
/// name's scalars, and the glued value's text when
/// an '=' is present.
fn split_long(token: &Token) -> (&[u32], Option<Text>) {
    match token.scalars.iter().position(|c| *c == '=' as u32) {
        Some(at) => {
            let units = token.text.units();
            let unit_at = crate::util::unit_offset(units, at);
            (
                &token.scalars[..at],
                Some(Text::from_units(units[unit_at + 1..].to_vec())),
            )
        }
        None => (&token.scalars[..], None),
    }
}

/// The text of a token from a scalar index on, for
/// the value glued behind a short option letter or
/// an '=' in a short cluster.
fn text_from_scalar(token: &Token, scalar_index: usize) -> Text {
    let units = token.text.units();
    let unit_at = crate::util::unit_offset(units, scalar_index);
    Text::from_units(units[unit_at..].to_vec())
}

/// The outcome of matching a long-option name
/// against a level's option set.
enum LongMatch {
    Exact(&'static str),
    Unique(&'static str),
    Ambiguous(Vec<&'static str>),
    None,
}

fn match_long(name: &[u32], options: &[&'static str]) -> LongMatch {
    let mut candidates: Vec<&'static str> = Vec::new();
    for option in options {
        let full = scalars_of(option);
        if full == name {
            return LongMatch::Exact(option);
        }
        if full.len() > name.len() && full[..name.len()] == *name {
            candidates.push(option);
        }
    }
    match candidates.len() {
        0 => LongMatch::None,
        1 => LongMatch::Unique(candidates[0]),
        _ => LongMatch::Ambiguous(candidates),
    }
}

enum ParseOutcome {
    Run(&'static str, Box<SubParsed>),
    Exit(i32),
}

/// Write bytes to stdout under its lock, reporting
/// only whether the write and its flush landed.
fn write_stdout(bytes: &[u8]) -> Result<(), std::io::Error> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(bytes).and_then(|_| handle.flush())
}

/// Write bytes to stderr under its lock, reporting
/// only whether the write and its flush landed.
fn write_stderr(bytes: &[u8]) -> Result<(), std::io::Error> {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    handle.write_all(bytes).and_then(|_| handle.flush())
}

/// The "error, ..." line's destination: stderr, or
/// stdout when stderr was closed at launch (the
/// Python backs a missing standard stream with the
/// surviving one, so its reports migrate there). A
/// failed write is dropped at this seam; the caller
/// keeps the exit its own rule assigns.
fn report_error(line: &str) {
    let mut bytes = line.as_bytes().to_vec();
    bytes.push(b'\n');
    if crate::util::closed_at_launch(2) {
        let _ = write_stdout(&bytes);
    } else {
        let _ = write_stderr(&bytes);
    }
}

fn parse_error(usage: &str, prog: &str, message: &str) -> ParseOutcome {
    let text = format!("{usage}{prog}: error: {message}\n");
    if crate::util::closed_at_launch(2) {
        // With stderr closed at launch the Python's
        // usage line migrates to stdout and its
        // error line is lost in the fallback stream;
        // the exit stays 2. The module's own error
        // reports, written in one piece, migrate
        // whole (see report_error).
        let _ = write_stdout(usage.as_bytes());
        return ParseOutcome::Exit(2);
    }
    match write_stderr(text.as_bytes()) {
        Ok(()) => ParseOutcome::Exit(2),
        // A usage that cannot be delivered collapses
        // to 1 where the Python's shutdown flush
        // failure gives 120 (the flush class, named
        // in the dossier).
        Err(_) => ParseOutcome::Exit(1),
    }
}

/// Print a help text the guarded way: a broken
/// stream is a caught write error with exit 1,
/// never a panic (the shutdown flush class, named
/// in the dossier). With stdout closed at launch
/// the help migrates to stderr and still exits 0,
/// as the Python's does.
fn print_help(text: &str) -> ParseOutcome {
    if crate::util::closed_at_launch(1) {
        return match write_stderr(text.as_bytes()) {
            Ok(()) => ParseOutcome::Exit(0),
            Err(_) => ParseOutcome::Exit(1),
        };
    }
    match write_stdout(text.as_bytes()) {
        Ok(()) => ParseOutcome::Exit(0),
        Err(error) => {
            report_error(&format!("error, {}", os_error_text(&error)));
            ParseOutcome::Exit(1)
        }
    }
}

struct SubParsed {
    positionals: Vec<Text>,
    values: Vec<(&'static str, Text)>,
}

impl SubParsed {
    fn get_text(&self, canonical: &str) -> Option<Text> {
        self.values
            .iter()
            .rev()
            .find_map(|(name, value)| (*name == canonical).then(|| value.clone()))
    }

    fn get_all(&self, canonical: &str) -> Vec<Text> {
        self.values
            .iter()
            .filter(|(name, _)| *name == canonical)
            .map(|(_, value)| value.clone())
            .collect()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum OptKind {
    Help,
    Value,
    Append,
}

struct OptSpec {
    canonical: &'static str,
    label: &'static str,
    kind: OptKind,
    short: Option<char>,
}

struct SubSpec {
    usage: &'static str,
    prog: &'static str,
    help: &'static str,
    positionals: &'static [&'static str],
    options: &'static [OptSpec],
    /// The required arguments in definition order:
    /// (label, is_positional).
    required: &'static [(&'static str, bool)],
}

const HELP_OPT: OptSpec = OptSpec {
    canonical: "--help",
    label: "-h/--help",
    kind: OptKind::Help,
    short: Some('h'),
};

const fn value_opt(canonical: &'static str) -> OptSpec {
    OptSpec {
        canonical,
        label: canonical,
        kind: OptKind::Value,
        short: None,
    }
}

const OUTPUT_OPT: OptSpec = OptSpec {
    canonical: "--output",
    label: "-o/--output",
    kind: OptKind::Value,
    short: Some('o'),
};

const CONFESS_SPEC: SubSpec = SubSpec {
    usage: CONFESS_USAGE,
    prog: "tetlock confess",
    help: CONFESS_HELP,
    positionals: &["record"],
    options: &[
        HELP_OPT,
        value_opt("--action"),
        value_opt("--sentence"),
        value_opt("--due"),
        value_opt("--at"),
        value_opt("--asserter"),
        value_opt("--approval"),
    ],
    required: &[
        ("record", true),
        ("--action", false),
        ("--sentence", false),
        ("--due", false),
        ("--at", false),
        ("--asserter", false),
        ("--approval", false),
    ],
};

const GRADE_SPEC: SubSpec = SubSpec {
    usage: GRADE_USAGE,
    prog: "tetlock grade",
    help: GRADE_HELP,
    positionals: &["record"],
    options: &[
        HELP_OPT,
        value_opt("--confession"),
        value_opt("--grade"),
        value_opt("--happened"),
        value_opt("--at"),
        value_opt("--asserter"),
        OptSpec {
            canonical: "--evidence",
            label: "--evidence",
            kind: OptKind::Append,
            short: None,
        },
        value_opt("--supersedes"),
    ],
    required: &[
        ("record", true),
        ("--confession", false),
        ("--grade", false),
        ("--happened", false),
        ("--at", false),
        ("--asserter", false),
    ],
};

const LEDGER_SPEC: SubSpec = SubSpec {
    usage: LEDGER_USAGE,
    prog: "tetlock ledger",
    help: LEDGER_HELP,
    positionals: &["record"],
    options: &[HELP_OPT, value_opt("--now"), OUTPUT_OPT],
    required: &[("record", true), ("--now", false)],
};

const WALL_SPEC: SubSpec = SubSpec {
    usage: WALL_USAGE,
    prog: "tetlock wall",
    help: WALL_HELP,
    positionals: &["record"],
    options: &[HELP_OPT, value_opt("--date"), OUTPUT_OPT],
    required: &[("record", true), ("--date", false)],
};

const TALLY_SPEC: SubSpec = SubSpec {
    usage: TALLY_USAGE,
    prog: "tetlock tally",
    help: TALLY_HELP,
    positionals: &["record"],
    options: &[HELP_OPT, value_opt("--now"), OUTPUT_OPT],
    required: &[("record", true), ("--now", false)],
};

fn spec_for(command: &str) -> &'static SubSpec {
    match command {
        "confess" => &CONFESS_SPEC,
        "grade" => &GRADE_SPEC,
        "ledger" => &LEDGER_SPEC,
        "wall" => &WALL_SPEC,
        _ => &TALLY_SPEC,
    }
}

/// Take the value for an option: a glued form wins,
/// else the next token, which an option-shaped
/// token can never be. On success `index` still
/// points at the option token; the caller advances
/// past both.
fn take_value(
    spec: &SubSpec,
    tokens: &[Token],
    index: &mut usize,
    glued: Option<Text>,
    opt: &OptSpec,
) -> Result<Text, ParseOutcome> {
    if let Some(value) = glued {
        return Ok(value);
    }
    match tokens.get(*index + 1) {
        Some(next) if !option_shaped(next) => {
            *index += 1;
            Ok(next.text.clone())
        }
        _ => Err(parse_error(
            spec.usage,
            spec.prog,
            &format!("argument {}: expected one argument", opt.label),
        )),
    }
}

/// A short cluster token's outcome.
enum ClusterFlow {
    /// The token is an extra.
    Extra,
    /// The token was consumed with nothing recorded.
    Skip,
    /// Values were recorded from the cluster.
    Recorded,
    /// The parse ends here.
    Exit(ParseOutcome),
}

/// Read a short cluster. A first letter the command
/// does not know makes the whole token an extra.
/// Help fires when the token ends, unless a scan
/// error in it fired first; a value letter takes
/// the rest of the cluster as its glued value, or
/// the next token when the cluster ends there.
fn short_cluster(
    spec: &SubSpec,
    tokens: &[Token],
    index: &mut usize,
    values: &mut Vec<(&'static str, Text)>,
) -> ClusterFlow {
    let token = &tokens[*index];
    let cluster = &token.scalars[1..];
    let first = cluster[0] as u8 as char;
    let known = spec.options.iter().any(|opt| opt.short == Some(first));
    if !known {
        return ClusterFlow::Extra;
    }
    let mut help_pending = false;
    let mut recorded = false;
    let mut at = 0usize;
    while at < cluster.len() {
        let letter = cluster[at] as u8 as char;
        if letter == 'h' {
            if cluster.get(at + 1) == Some(&('=' as u32)) {
                let value = text_from_scalar(token, at + 3);
                return ClusterFlow::Exit(parse_error(
                    spec.usage,
                    spec.prog,
                    &format!(
                        "argument -h/--help: ignored explicit argument {}",
                        repr_text(&value)
                    ),
                ));
            }
            help_pending = true;
            at += 1;
            continue;
        }
        let opt = spec
            .options
            .iter()
            .find(|opt| opt.short == Some(letter) && opt.kind != OptKind::Help);
        match opt {
            Some(opt) => {
                let rest = &cluster[at + 1..];
                let glued = if rest.is_empty() {
                    None
                } else if rest[0] == '=' as u32 {
                    Some(text_from_scalar(token, at + 3))
                } else {
                    Some(text_from_scalar(token, at + 2))
                };
                match take_value(spec, tokens, index, glued, opt) {
                    Ok(value) => {
                        values.push((opt.canonical, value));
                        recorded = true;
                    }
                    Err(outcome) => return ClusterFlow::Exit(outcome),
                }
                // A value letter ends the cluster.
                at = cluster.len();
            }
            None => {
                // A letter the command does not know,
                // after known letters: argparse reads
                // the whole token as unrecognized.
                return ClusterFlow::Extra;
            }
        }
    }
    if help_pending {
        return ClusterFlow::Exit(print_help(spec.help));
    }
    if recorded {
        return ClusterFlow::Recorded;
    }
    ClusterFlow::Skip
}

fn parse_sub(command: &'static str, tokens: &[Token], top_extras: &[Text]) -> ParseOutcome {
    let spec = spec_for(command);
    let candidates: Vec<&'static str> = spec.options.iter().map(|opt| opt.canonical).collect();
    let mut positionals: Vec<Text> = Vec::new();
    let mut extras: Vec<Text> = top_extras.to_vec();
    let mut values: Vec<(&'static str, Text)> = Vec::new();
    let mut separated = false;
    let mut index = 0usize;

    fn push_positional(
        token: &Text,
        spec: &SubSpec,
        positionals: &mut Vec<Text>,
        extras: &mut Vec<Text>,
    ) {
        if positionals.len() < spec.positionals.len() {
            positionals.push(token.clone());
        } else {
            extras.push(token.clone());
        }
    }

    while index < tokens.len() {
        let token = &tokens[index];
        if separated {
            push_positional(&token.text, spec, &mut positionals, &mut extras);
            index += 1;
            continue;
        }
        if token.scalars == scalars_of("--") {
            separated = true;
            if positionals.len() == spec.positionals.len() {
                extras.push(token.text.clone());
            }
            index += 1;
            continue;
        }
        if token.scalars.len() >= 2
            && token.scalars[0] == '-' as u32
            && token.scalars[1] == '-' as u32
        {
            let (name, glued) = split_long(token);
            let chosen = match match_long(name, &candidates) {
                LongMatch::Exact(option) | LongMatch::Unique(option) => option,
                LongMatch::Ambiguous(matches) => {
                    return parse_error(
                        spec.usage,
                        spec.prog,
                        &format!(
                            "ambiguous option: {} could match {}",
                            token.rendered(),
                            matches.join(", ")
                        ),
                    );
                }
                LongMatch::None => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
            };
            let opt = spec
                .options
                .iter()
                .find(|opt| opt.canonical == chosen)
                .expect("a spec option");
            if opt.kind == OptKind::Help {
                if let Some(value) = glued {
                    return parse_error(
                        spec.usage,
                        spec.prog,
                        &format!(
                            "argument -h/--help: ignored explicit argument {}",
                            repr_text(&value)
                        ),
                    );
                }
                return print_help(spec.help);
            }
            match take_value(spec, tokens, &mut index, glued, opt) {
                Ok(value) => values.push((opt.canonical, value)),
                Err(outcome) => return outcome,
            }
            index += 1;
            continue;
        }
        if option_shaped(token) {
            match short_cluster(spec, tokens, &mut index, &mut values) {
                ClusterFlow::Extra => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
                ClusterFlow::Skip | ClusterFlow::Recorded => {
                    index += 1;
                    continue;
                }
                ClusterFlow::Exit(outcome) => return outcome,
            }
        }
        push_positional(&token.text, spec, &mut positionals, &mut extras);
        index += 1;
    }

    let mut missing: Vec<&str> = Vec::new();
    for (label, is_positional) in spec.required {
        if *is_positional {
            let at = spec
                .positionals
                .iter()
                .position(|name| name == label)
                .expect("a spec positional");
            if positionals.len() <= at {
                missing.push(label);
            }
        } else if !values.iter().any(|(name, _)| name == label) {
            missing.push(label);
        }
    }
    if !missing.is_empty() {
        return parse_error(
            spec.usage,
            spec.prog,
            &format!(
                "the following arguments are required: {}",
                missing.join(", ")
            ),
        );
    }
    if !extras.is_empty() {
        let rendered: Vec<String> = extras.iter().map(|t| stderr_render(t.units())).collect();
        return parse_error(
            TOP_USAGE,
            "tetlock",
            &format!("unrecognized arguments: {}", rendered.join(" ")),
        );
    }
    ParseOutcome::Run(
        command,
        Box::new(SubParsed {
            positionals,
            values,
        }),
    )
}

const COMMANDS: [&str; 5] = ["confess", "grade", "ledger", "wall", "tally"];

/// Parse the top level: help, the command word, and
/// the held extras an unknown token becomes.
fn parse_top(tokens: &[Token]) -> ParseOutcome {
    let mut extras: Vec<Text> = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.scalars.len() >= 2
            && token.scalars[0] == '-' as u32
            && token.scalars[1] == '-' as u32
            && token.scalars != scalars_of("--")
        {
            let (name, glued) = split_long(token);
            match match_long(name, &["--help"]) {
                LongMatch::Exact("--help") | LongMatch::Unique("--help") => {
                    if let Some(value) = glued {
                        return parse_error(
                            TOP_USAGE,
                            "tetlock",
                            &format!(
                                "argument -h/--help: ignored explicit argument {}",
                                repr_text(&value)
                            ),
                        );
                    }
                    return print_help(TOP_HELP);
                }
                _ => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
            }
        }
        if token.scalars == scalars_of("--") || !option_shaped(token) {
            if token.scalars == scalars_of("--") && index == tokens.len() - 1 {
                break;
            }
            for command in COMMANDS {
                if token.text.units() == Text::from_str(command).units() {
                    let command: &'static str = command;
                    return parse_sub(command, &tokens[index + 1..], &extras);
                }
            }
            return parse_error(
                TOP_USAGE,
                "tetlock",
                &format!(
                    "argument command: invalid choice: {} (choose from 'confess', 'grade', 'ledger', 'wall', 'tally')",
                    repr_text(&token.text)
                ),
            );
        }
        if token.scalars[1] == 'h' as u32 {
            return print_help(TOP_HELP);
        }
        extras.push(token.text.clone());
        index += 1;
    }
    parse_error(
        TOP_USAGE,
        "tetlock",
        "the following arguments are required: command",
    )
}

/// Write text to stdout the way the Python's
/// _emit_text does: a failure is the named refusal
/// "error, cannot write to stdout" and exit 1. With
/// stdout closed at launch the Python's write
/// raises out of _emit_text entirely (its stdout
/// is unusable), the crash class: the command's
/// file effects have already happened and the exit
/// is a bare 1 with nothing anywhere.
fn emit_text(bytes: &[u8]) -> i32 {
    if crate::util::closed_at_launch(1) {
        return 1;
    }
    match write_stdout(bytes) {
        Ok(()) => 0,
        Err(_) => {
            report_error("error, cannot write to stdout");
            1
        }
    }
}

/// The view write: the document rendered strict to
/// a temp file in the target's directory, then
/// replaced onto the target (WW-12). Any failure
/// is the named refusal with the target as typed.
fn emit_view(document: &Value, out: &Text) -> i32 {
    let bytes = match dumps_view(document, Mode::Strict) {
        Ok(bytes) => bytes,
        Err(_) => {
            report_error(&format!(
                "error, cannot write the view, {}",
                stderr_render(out.units())
            ));
            return 1;
        }
    };
    // The write consumes the constructed spelling
    // (WW-36); the refusal below keeps the raw one,
    // as the Python's message does.
    match write_atomically(&construct_units(out.units()), &bytes) {
        Ok(()) => 0,
        Err(()) => {
            report_error(&format!(
                "error, cannot write the view, {}",
                stderr_render(out.units())
            ));
            1
        }
    }
}

/// The temp's directory as mkstemp reads the
/// parent (WW-42): the parent absolutized
/// lexically, joined to the working directory
/// when relative and then normalized over its
/// components with no link resolution anywhere
/// in the step. An absolute parent never
/// consults the working directory; a failed
/// working-directory read on a relative parent
/// fails the step (WW-43).
fn lexical_abspath(units: &[u16]) -> Option<Vec<u16>> {
    let joined: Vec<u16> = if units.first() == Some(&SEP) {
        units.to_vec()
    } else {
        let cwd = std::env::current_dir().ok()?;
        join_frag(&raw_units_of_path(&cwd), units)
    };
    Some(normalize_lexically(&joined))
}

fn write_atomically(out: &[u16], bytes: &[u8]) -> Result<(), ()> {
    let path = crate::util::os_path_from_units(out);
    let dir = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => std::path::PathBuf::from("."),
    };
    // The temp is created under the parent as
    // mkstemp absolutizes it (WW-42), not under
    // the parent as spelled: a symlink component
    // the spelling pops with a following ".."
    // must not carry the temp to the physical
    // directory the interpreter never names. The
    // rename below still targets the constructed
    // path itself.
    let tmp_dir = match lexical_abspath(&raw_units_of_path(&dir)) {
        Some(units) => units,
        None => return Err(()),
    };
    // The candidate is the constructed name's own
    // units, a dot, and the drawn suffix (WW-48),
    // assembled as units and fsencoded whole, so
    // a name carrying a raw byte reaches the
    // create at its own length, never lengthened
    // by a lossy conversion. The name is the part
    // of the constructed path after its last
    // separator; a construction with no component
    // standing (the dot or the anchor alone) has
    // no name, as pathlib's name reads there.
    let name: &[u16] = match out.iter().rposition(|unit| *unit == SEP) {
        Some(at) => &out[at + 1..],
        None => out,
    };
    let name: &[u16] = if name == DOT { &[] } else { name };
    let mut attempt = 0u32;
    let tmp = loop {
        let counter = TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut candidate = tmp_dir.clone();
        if candidate.last() != Some(&SEP) {
            candidate.push(SEP);
        }
        candidate.extend_from_slice(name);
        candidate.extend_from_slice(DOT);
        candidate.extend_from_slice(&temp_suffix(&tmp_dir, name, counter, attempt));
        let candidate = crate::util::os_path_from_units(&candidate);
        #[cfg(unix)]
        let opener = {
            use std::os::unix::fs::OpenOptionsExt;
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true).mode(0o600);
            options
        };
        #[cfg(not(unix))]
        let opener = {
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            options
        };
        match opener.open(&candidate) {
            Ok(mut file) => match file.write_all(bytes).and_then(|_| file.flush()) {
                Ok(()) => break candidate,
                Err(_) => {
                    let _ = std::fs::remove_file(&candidate);
                    return Err(());
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                attempt += 1;
                if attempt > 64 {
                    return Err(());
                }
            }
            Err(_) => return Err(()),
        }
    };
    match std::fs::rename(&tmp, &path) {
        Ok(()) => Ok(()),
        Err(_) => {
            let _ = std::fs::remove_file(&tmp);
            Err(())
        }
    }
}

static TEMP_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The temp candidate's suffix (WW-48): exactly
/// eight lowercase hexadecimal characters from
/// the first four bytes of a SHA-256 hash over
/// the process id, the wall-clock nanoseconds,
/// the per-process counter, the attempt, the
/// temp directory's bytes, and the name's
/// bytes. The terms make successive draws
/// differ and keep the sequence from being a
/// public function of the process id, the
/// defect WW-47 measures in the shipped
/// decimal concatenation of the same three
/// numbers.
fn temp_suffix(dir: &[u16], name: &[u16], counter: u32, attempt: u32) -> Vec<u16> {
    use sha2::Digest;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let mut hasher = sha2::Sha256::new();
    hasher.update(std::process::id().to_le_bytes());
    hasher.update(nanos.to_le_bytes());
    hasher.update(counter.to_le_bytes());
    hasher.update(attempt.to_le_bytes());
    hasher.update(units_bytes(dir));
    hasher.update(units_bytes(name));
    let digest = hasher.finalize();
    let mut suffix = Vec::with_capacity(8);
    for byte in &digest[..4] {
        suffix.push(hex_unit(byte >> 4));
        suffix.push(hex_unit(byte & 0xF));
    }
    suffix
}

fn hex_unit(nibble: u8) -> u16 {
    match nibble {
        0..=9 => b'0' as u16 + nibble as u16,
        _ => b'a' as u16 + (nibble as u16 - 10),
    }
}

/// A unit string's OS bytes, for the suffix
/// hash's inputs: the fsencoding on unix, the
/// lossy rendering where the crate has no
/// fsencoding (the hash's inputs are a draw,
/// never a path, so the rendering there costs
/// nothing observable).
fn units_bytes(units: &[u16]) -> Vec<u8> {
    #[cfg(unix)]
    {
        crate::util::fsencode_bytes(units)
    }
    #[cfg(not(unix))]
    {
        String::from_utf16_lossy(units).into_bytes()
    }
}

/// The raw units of an OS path, unedited: the
/// walk and the construction read a path's own
/// bytes, never a normalized form of them.
fn raw_units_of_path(path: &std::path::Path) -> Vec<u16> {
    #[cfg(unix)]
    let bytes = {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes().to_vec()
    };
    #[cfg(not(unix))]
    let bytes = path.to_string_lossy().into_owned().into_bytes();
    fsdecode_units(&bytes)
}

/// Why a resolution failed (WW-34, WW-35). A
/// loop and the depth budget are the walk's own
/// failures; a failed current-directory read is
/// the deleted-working-directory case, and the
/// guard posts each failure kind as the oracle's
/// own control flow posts it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ResolveFail {
    Loop,
    Budget,
    Cwd,
}

/// The deepest nesting the interpreter's walk
/// reaches at this CLI: 991 nested splices
/// resolve and the 992nd fails the resolution
/// (WW-33). The constant is this crate's own,
/// measured at its guard on head chains and on
/// prefix chains; a sibling's constant is never
/// inherited.
const MAX_WALK_DEPTH: usize = 991;

const SEP: u16 = 0x2F; // '/'
const DOT: &[u16] = &[0x2E];
const DOTDOT: &[u16] = &[0x2E, 0x2E];

/// posixpath.join for one pair: an absolute
/// second spelling stands alone, an empty or
/// separator-ending first spelling takes the
/// second directly, anything else joins on a
/// separator.
fn join_frag(first: &[u16], second: &[u16]) -> Vec<u16> {
    if second.first() == Some(&SEP) {
        return second.to_vec();
    }
    if first.is_empty() || first.last() == Some(&SEP) {
        let mut out = first.to_vec();
        out.extend_from_slice(second);
        return out;
    }
    let mut out = first.to_vec();
    out.push(SEP);
    out.extend_from_slice(second);
    out
}

/// posixpath.split: the head is everything
/// before the final separator (the root stands
/// as its own head), the tail is the last
/// component.
fn split_path(units: &[u16]) -> (Vec<u16>, Vec<u16>) {
    match units.iter().rposition(|unit| *unit == SEP) {
        Some(0) => (vec![SEP], units[1..].to_vec()),
        Some(at) => (units[..at].to_vec(), units[at + 1..].to_vec()),
        None => (Vec::new(), units.to_vec()),
    }
}

/// True where lstat reads the path as a symlink.
/// An lstat that fails reads as not a link, the
/// interpreter's own answer: the name simply
/// joins the accumulated prefix unresolved.
fn spells_link(units: &[u16]) -> bool {
    let path = crate::util::os_path_from_units(units);
    match std::fs::symlink_metadata(path) {
        Ok(meta) => meta.file_type().is_symlink(),
        Err(_) => false,
    }
}

/// The splice walk transcribed from the
/// interpreter's _joinrealpath (WW-33). The
/// accumulated prefix grows component by
/// component; a symlink splices its target in
/// by recursion, an absolute target restarting
/// at the root and a relative one joining the
/// accumulated prefix; a ".." pops the
/// accumulated resolved prefix lexically,
/// without being read. The seen map carries
/// each path's finished resolution, or None
/// while that resolution is still open: a name
/// met again under its own open resolution
/// stops the walk there, the part resolved so
/// far returning with the rest of the spelling
/// standing, for the closing stat to judge
/// (WW-38). Depth is nesting (a
/// splice inside another splice's resolution),
/// never a count of splices in the spelling,
/// and a call entered past MAX_WALK_DEPTH fails
/// the whole resolution as the budget. Returns
/// the finished path and whether it finished
/// clean, or the budget failure.
fn joinrealpath(
    path: &[u16],
    rest: &[u16],
    seen: &mut std::collections::HashMap<Vec<u16>, Option<Vec<u16>>>,
    depth: usize,
) -> Result<(Vec<u16>, bool), ResolveFail> {
    if depth > MAX_WALK_DEPTH {
        return Err(ResolveFail::Budget);
    }
    let mut path: Vec<u16> = path.to_vec();
    let mut rest: &[u16] = rest;
    if rest.first() == Some(&SEP) {
        rest = &rest[1..];
        path = vec![SEP];
    }
    while !rest.is_empty() {
        let (name, tail) = match rest.iter().position(|unit| *unit == SEP) {
            Some(at) => (&rest[..at], &rest[at + 1..]),
            None => (rest, &rest[rest.len()..]),
        };
        rest = tail;
        if name.is_empty() || name == DOT {
            continue;
        }
        if name == DOTDOT {
            if path.is_empty() {
                path = DOTDOT.to_vec();
            } else {
                let (head, last) = split_path(&path);
                if last == DOTDOT {
                    path = join_frag(&join_frag(&head, DOTDOT), DOTDOT);
                } else {
                    path = head;
                }
            }
            continue;
        }
        let newpath = join_frag(&path, name);
        if !spells_link(&newpath) {
            path = newpath;
            continue;
        }
        if let Some(cached) = seen.get(&newpath) {
            match cached {
                Some(resolved) => {
                    path = resolved.clone();
                    continue;
                }
                None => return Ok((join_frag(&newpath, rest), false)),
            }
        }
        seen.insert(newpath.clone(), None);
        let target = {
            let os = crate::util::os_path_from_units(&newpath);
            match std::fs::read_link(os) {
                Ok(target) => raw_units_of_path(&target),
                Err(_) => return Err(ResolveFail::Loop),
            }
        };
        let (resolved, ok) = joinrealpath(&path, &target, seen, depth + 1)?;
        if !ok {
            return Ok((join_frag(&resolved, rest), false));
        }
        path = resolved;
        seen.insert(newpath, Some(path.clone()));
    }
    Ok((path, true))
}

/// The kernel's symlink-loop errno on this
/// platform family.
const ELOOP: i32 = 40;

/// posixpath.normpath over a finished absolute
/// path: empty and "." components drop, ".."
/// pops lexically, the root holds. The walk's
/// own result is already nearly this; the
/// partial path a stopped walk returns (WW-38)
/// is not, and the interpreter normalizes both.
fn normalize_lexically(units: &[u16]) -> Vec<u16> {
    let mut parts: Vec<&[u16]> = Vec::new();
    for part in units.split(|unit| *unit == SEP) {
        if part.is_empty() || part == DOT {
            continue;
        }
        if part == DOTDOT {
            parts.pop();
            continue;
        }
        parts.push(part);
    }
    let mut out = vec![SEP];
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(SEP);
        }
        out.extend_from_slice(part);
    }
    out
}

/// A path resolved the way pathlib's resolve
/// reads, through the splice walk (WW-33): the
/// record and the output compare as finished
/// paths or not at all. A relative spelling
/// joins the current directory first, and a
/// failed current-directory read fails the
/// resolution outright (WW-35): no fallback
/// prefix is ever substituted, since a
/// substituted prefix compares a fiction and
/// can pass a guard the real one fails. Where
/// the walk stops at a loop the resolution is
/// the normalization of the part it resolved
/// with the rest standing, and the loop is
/// convicted only by the closing stat: a stat
/// of the finished path failing with ELOOP is
/// the loop failure, and every other stat
/// outcome (a sound file, a dangling end) lets
/// the resolution stand (WW-38).
fn resolve_path(units: &[u16]) -> Result<Vec<u16>, ResolveFail> {
    let absolute: Vec<u16> = if units.first() == Some(&SEP) {
        units.to_vec()
    } else {
        let cwd = std::env::current_dir().map_err(|_| ResolveFail::Cwd)?;
        join_frag(&raw_units_of_path(&cwd), units)
    };
    let mut seen = std::collections::HashMap::new();
    let (resolved, _) = joinrealpath(&[], &absolute, &mut seen, 0)?;
    let finished = normalize_lexically(&resolved);
    let os = crate::util::os_path_from_units(&finished);
    if let Err(error) = std::fs::metadata(os) {
        if error.raw_os_error() == Some(ELOOP) {
            return Err(ResolveFail::Loop);
        }
    }
    Ok(finished)
}

/// The spelling pathlib's construction hands to
/// the write (WW-36): every empty component and
/// every "." component drops wherever it stands,
/// a ".." and every other component stand whole,
/// a leading pair of separators stands as the
/// anchor and a longer leading run collapses to
/// one separator, and a spelling with no
/// component left stands as "." or as its
/// anchor. The guard and the cannot-write
/// message keep the raw spelling; only the
/// write consumes this one.
fn construct_units(units: &[u16]) -> Vec<u16> {
    let mut lead = 0;
    while lead < units.len() && units[lead] == SEP {
        lead += 1;
    }
    let anchor: &[u16] = match lead {
        0 => &[],
        2 => &[SEP, SEP],
        _ => &[SEP],
    };
    let mut parts: Vec<&[u16]> = Vec::new();
    for part in units[lead..].split(|unit| *unit == SEP) {
        if part.is_empty() || part == DOT {
            continue;
        }
        parts.push(part);
    }
    if parts.is_empty() {
        if anchor.is_empty() {
            return DOT.to_vec();
        }
        return anchor.to_vec();
    }
    let mut out = anchor.to_vec();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(SEP);
        }
        out.extend_from_slice(part);
    }
    out
}

/// Report a store failure the way the Python's
/// main does: a named refusal after "error, ", and
/// the crash class as a bare exit 1.
fn report(result: Result<i32, StoreError>) -> i32 {
    match result {
        Ok(code) => code,
        Err(StoreError::Named(error)) => {
            report_error(&format!("error, {}", error.render_stderr()));
            1
        }
        Err(StoreError::Crash) => 1,
    }
}

fn text_arg(parsed: &SubParsed, canonical: &str) -> Text {
    parsed.get_text(canonical).expect("a required argument")
}

fn parse_evidence(items: &[Text]) -> Result<Vec<(Text, Text)>, StoreError> {
    let mut evidence = Vec::new();
    for item in items {
        let units = item.units();
        let colon = units.iter().position(|unit| *unit == b':' as u16);
        match colon {
            Some(at) if at > 0 && at + 1 < units.len() => evidence.push((
                Text::from_units(units[..at].to_vec()),
                Text::from_units(units[at + 1..].to_vec()),
            )),
            _ => {
                let mut message = Msg::str("evidence must name a source and a reference, ");
                message.push_text(item);
                return Err(StoreError::Named(message));
            }
        }
    }
    Ok(evidence)
}

fn execute(command: &str, parsed: &SubParsed) -> i32 {
    let store = Store::new(RecPath::from_units(parsed.positionals[0].units()));
    if let Some(out) = parsed.get_text("--output") {
        // WW-25a: an output whose resolution fails
        // counts as naming the record and is refused
        // here, where the Python dies in the crash
        // class before its guard. Two failures take
        // the bare crash class instead, one on each
        // side, and nothing is printed or written
        // for either. An output whose resolution
        // fails on the current-directory read
        // (WW-45): the Python's guard evaluates the
        // output's resolve first and dies there,
        // before any comparison can name the
        // record. A record whose resolution fails
        // on the current-directory read (WW-35):
        // the Python dies inside the guard
        // expression, before any load. A record
        // whose resolution fails in the walk does
        // not refuse at the guard: the load below
        // refuses it by name, the crash class's
        // named side.
        let refused = match resolve_path(out.units()) {
            Err(ResolveFail::Cwd) => return 1,
            Err(_) => true,
            Ok(resolved_out) => match resolve_path(parsed.positionals[0].units()) {
                Ok(resolved_record) => resolved_out == resolved_record,
                Err(ResolveFail::Cwd) => return 1,
                Err(_) => false,
            },
        };
        if refused {
            report_error("error, a view never overwrites the record");
            return 1;
        }
    }
    match command {
        "confess" => report((|| {
            let id = crate::wall::confess(
                &store,
                &text_arg(parsed, "--action"),
                &text_arg(parsed, "--sentence"),
                &text_arg(parsed, "--due"),
                &text_arg(parsed, "--at"),
                &text_arg(parsed, "--asserter"),
                &text_arg(parsed, "--approval"),
            )?;
            let mut bytes = id.into_bytes();
            bytes.push(b'\n');
            Ok(emit_text(&bytes))
        })()),
        "grade" => report((|| {
            let evidence = parse_evidence(&parsed.get_all("--evidence"))?;
            let id = crate::wall::grade(
                &store,
                &text_arg(parsed, "--confession"),
                &text_arg(parsed, "--grade"),
                &text_arg(parsed, "--happened"),
                &text_arg(parsed, "--at"),
                &text_arg(parsed, "--asserter"),
                &evidence,
                parsed.get_text("--supersedes").as_ref(),
            )?;
            let mut bytes = id.into_bytes();
            bytes.push(b'\n');
            Ok(emit_text(&bytes))
        })()),
        "ledger" | "wall" | "tally" => {
            let document = match command {
                "ledger" => crate::wall::ledger(&store, &text_arg(parsed, "--now")),
                "wall" => crate::wall::wall(&store, &text_arg(parsed, "--date")),
                _ => crate::wall::tally(&store, &text_arg(parsed, "--now")),
            };
            report((|| {
                let document = document?;
                Ok(match parsed.get_text("--output") {
                    Some(out) => emit_view(&document, &out),
                    None => match dumps_view(&document, Mode::Stdout) {
                        Ok(bytes) => emit_text(&bytes),
                        Err(_) => {
                            report_error("error, cannot write to stdout");
                            1
                        }
                    },
                })
            })())
        }
        _ => 1,
    }
}

/// An operating system error's CPython rendering,
/// "[Errno N] message", for the CLI's stream
/// failures; the strerror table itself lives in
/// util, shared with the record's phrases.
fn os_error_text(err: &std::io::Error) -> String {
    match err.raw_os_error() {
        Some(errno) => {
            let bare = err.to_string();
            let text = crate::util::strerror_text(err);
            if text == bare {
                bare
            } else {
                format!("[Errno {errno}] {text}")
            }
        }
        None => err.to_string(),
    }
}

pub fn run_cli() -> i32 {
    let tokens: Vec<Token> = std::env::args_os()
        .skip(1)
        .map(|arg| {
            #[cfg(unix)]
            let bytes = arg.as_encoded_bytes().to_vec();
            #[cfg(not(unix))]
            let bytes = arg.to_string_lossy().into_owned().into_bytes();
            Token::from_units(fsdecode_units(&bytes))
        })
        .collect();
    match parse_top(&tokens) {
        ParseOutcome::Run(command, parsed) => execute(command, &parsed),
        ParseOutcome::Exit(code) => code,
    }
}

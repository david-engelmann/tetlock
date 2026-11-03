//! Confessions, outcomes, and the views (WW-5 to
//! WW-16), over the record the adapter reads.
//! Claims are flighttape values; field reads go
//! through the onrecord crate's field helper so the
//! two readers agree on shape.

use flighttape::value::{Text, Value};
use onrecord::store::field;

use crate::dates;
use crate::record::{Store, StoreError};
use crate::util::{self, Msg};

pub const GRADES: [&str; 3] = ["held", "broke", "unverifiable"];
pub const STATES: [&str; 4] = ["graded", "awaiting", "overdue", "superseded"];
const TERMINATORS: [u32; 3] = ['.' as u32, '?' as u32, '!' as u32];

fn as_text(value: &Value) -> Option<&Text> {
    if let Value::Str(text) = value {
        Some(text)
    } else {
        None
    }
}

fn get<'a>(claim: &'a Value, name: &str) -> Option<&'a Value> {
    field(claim, name)
}

fn get_text<'a>(claim: &'a Value, name: &str) -> Option<&'a Text> {
    get(claim, name).and_then(as_text)
}

/// Python truthiness over a parsed value.
fn py_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Int(digits) => digits.chars().any(|c| c != '0' && c != '-'),
        Value::Float(x) => *x != 0.0,
        Value::Str(text) => !text.units().is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(entries) => !entries.is_empty(),
    }
}

fn present_truthy(claim: &Value, name: &str) -> bool {
    get(claim, name).map(py_truthy).unwrap_or(false)
}

/// Python's str() of a value, for the phrases that
/// interpolate one: a string renders raw, everything
/// else by its repr.
fn py_str(value: &Value) -> Msg {
    match value {
        Value::Str(text) => Msg(text.units().to_vec()),
        other => Msg::str(&crate::pyrepr::repr_value(other)),
    }
}

/// Code points of a text, surrogate pairs combined
/// and lone surrogates standing as their own values.
fn code_points(text: &Text) -> Vec<u32> {
    let units = text.units();
    let mut points = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..0xDC00).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..0xE000).contains(&units[index + 1])
        {
            points.push(util::decode_pair(unit, units[index + 1]));
            index += 2;
        } else {
            points.push(unit as u32);
            index += 1;
        }
    }
    points
}

/// The mechanical sentence rule (WW-5, WW-5a,
/// WW-5b).
pub fn sentence_defect(text: &Text) -> Option<&'static str> {
    let points = code_points(text);
    if points.contains(&(b'\n' as u32)) {
        return Some("the sentence is not one line");
    }
    if points.last() != Some(&(b'.' as u32)) {
        return Some("the sentence does not end in a period");
    }
    let head = &points[..points.len() - 1];
    if head.iter().any(|point| TERMINATORS.contains(point)) {
        return Some("the sentence holds an internal sentence break");
    }
    if !head.iter().any(|point| util::py_is_alnum(*point)) {
        return Some("the sentence is empty");
    }
    None
}

pub fn confessions(claims: &[Value]) -> Vec<&Value> {
    claims
        .iter()
        .filter(|claim| {
            matches!(get_text(claim, "kind"),
                Some(kind) if kind.units() == Text::from_str("confession").units())
        })
        .collect()
}

/// What WW-5 requires of a confession, checked on
/// claims the write path did not mint (WW-8b).
pub fn confession_defects(claim: &Value) -> Result<Vec<&'static str>, StoreError> {
    let mut defects = Vec::new();
    let empty = Value::Array(Vec::new());
    let evidence = match get(claim, "evidence") {
        Some(value) if py_truthy(value) => value,
        _ => &empty,
    };
    let entries: &[Value] = match evidence {
        Value::Array(items) => items,
        Value::Str(_) | Value::Object(_) => {
            // Iterating a string or a mapping yields
            // non-dict items one by one; none of them
            // is approval evidence.
            &[]
        }
        _ => return Err(StoreError::Crash),
    };
    let has_approval = entries.iter().any(|entry| {
        if !matches!(entry, Value::Object(_)) {
            return false;
        }
        let source_ok = matches!(get_text(entry, "source"),
            Some(source) if source.units() == Text::from_str("approval").units());
        let reference_ok = matches!(get_text(entry, "reference"),
            Some(reference) if !util::py_strip_units(reference.units()).is_empty());
        source_ok && reference_ok
    });
    if !has_approval {
        defects.push("no approval evidence");
    }
    let subject_ok = matches!(get_text(claim, "subject"),
        Some(subject) if !util::py_strip_units(subject.units()).is_empty());
    if !subject_ok {
        defects.push("no subject naming the action");
    }
    if !present_truthy(claim, "due") {
        defects.push("no due date");
    }
    let text = match get(claim, "text") {
        Some(value) if py_truthy(value) => match as_text(value) {
            Some(text) => Some(text.clone()),
            None => return Err(StoreError::Crash),
        },
        _ => None,
    };
    let defect = match &text {
        Some(text) => sentence_defect(text),
        None => sentence_defect(&Text::from_str("")),
    };
    if let Some(defect) = defect {
        defects.push(defect);
    }
    Ok(defects)
}

fn superseded_ids(claims: &[Value]) -> Vec<Vec<u16>> {
    claims
        .iter()
        .filter_map(|claim| get_text(claim, "supersedes"))
        .filter(|target| !target.units().is_empty())
        .map(|target| target.units().to_vec())
        .collect()
}

/// Live outcomes resolved once per view (WW-7b).
pub struct OutcomeIndex<'a> {
    live: Vec<(Vec<u16>, &'a Value)>,
}

impl<'a> OutcomeIndex<'a> {
    pub fn new(claims: &'a [Value]) -> Result<OutcomeIndex<'a>, StoreError> {
        let outcomes: Vec<&Value> = claims
            .iter()
            .filter(|claim| {
                matches!(get_text(claim, "kind"),
                    Some(kind) if kind.units() == Text::from_str("outcome").units())
            })
            .collect();
        let superseded_by_outcome: Vec<Vec<u16>> = outcomes
            .iter()
            .filter_map(|claim| get_text(claim, "supersedes"))
            .filter(|target| !target.units().is_empty())
            .map(|target| target.units().to_vec())
            .collect();
        let mut live: Vec<(Vec<u16>, &Value)> = Vec::new();
        for outcome in outcomes {
            let id = match get_text(outcome, "id") {
                Some(id) => id.units().to_vec(),
                None => continue,
            };
            if superseded_by_outcome.contains(&id) {
                continue;
            }
            let subject = match get(outcome, "subject") {
                Some(value) if py_truthy(value) => match as_text(value) {
                    Some(text) => text.clone(),
                    None => return Err(StoreError::Crash),
                },
                _ => Text::from_str(""),
            };
            let prefix = Text::from_str("confession:");
            if !subject.units().starts_with(prefix.units()) {
                continue;
            }
            let confession_units = subject.units()[prefix.units().len()..].to_vec();
            if let Some((_, held)) = live
                .iter()
                .find(|(held_id, _)| *held_id == confession_units)
            {
                let held_id = match get_text(held, "id") {
                    Some(id) => id.clone(),
                    None => return Err(StoreError::Crash),
                };
                let mut message = Msg::str("confession ");
                message.push_units(&confession_units);
                message.push_str(" holds two live outcomes, ");
                message.push_units(held_id.units());
                message.push_str(" and ");
                message.push_units(&id);
                message.push_str("; graded once is the model and the record stands outside it");
                return Err(StoreError::Named(message));
            }
            let grade = get(outcome, "grade");
            let grade_ok = matches!(grade,
                Some(Value::Str(text)) if GRADES.iter().any(|g| text.units() == Text::from_str(g).units()));
            if !grade_ok {
                let mut message = Msg::str("outcome ");
                message.push_units(&id);
                message.push_str(" carries a grade outside the fixed set, ");
                match grade {
                    Some(value) => message.push_msg(&py_str(value)),
                    None => message.push_str("None"),
                }
                return Err(StoreError::Named(message));
            }
            live.push((confession_units, outcome));
        }
        Ok(OutcomeIndex { live })
    }

    pub fn get(&self, confession_id: &[u16]) -> Option<&'a Value> {
        self.live
            .iter()
            .find(|(held, _)| held.as_slice() == confession_id)
            .map(|(_, outcome)| *outcome)
    }
}

pub fn confess(
    store: &Store,
    action: &Text,
    sentence: &Text,
    due: &Text,
    at: &Text,
    asserter: &Text,
    approval: &Text,
) -> Result<String, StoreError> {
    if let Some(defect) = sentence_defect(sentence) {
        return Err(StoreError::named(defect));
    }
    if util::py_strip_units(action.units()).is_empty() {
        return Err(StoreError::named("the action names nothing"));
    }
    if util::py_strip_units(approval.units()).is_empty() {
        return Err(StoreError::named("the approval names nothing"));
    }
    store.add(
        sentence,
        asserter,
        at,
        &[(Text::from_str("approval"), approval.clone())],
        None,
        Some(action),
        Some(due),
        &[(
            Text::from_str("kind"),
            Value::Str(Text::from_str("confession")),
        )],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn grade(
    store: &Store,
    confession_id: &Text,
    grade_value: &Text,
    happened: &Text,
    at: &Text,
    asserter: &Text,
    evidence: &[(Text, Text)],
    supersedes: Option<&Text>,
) -> Result<String, StoreError> {
    if !GRADES
        .iter()
        .any(|g| grade_value.units() == Text::from_str(g).units())
    {
        let mut message = Msg::str("grade must be one of held, broke, unverifiable, ");
        message.push_units(grade_value.units());
        return Err(StoreError::Named(message));
    }
    let claims = store.claims()?;
    let target = claims.iter().find(|claim| {
        matches!(get_text(claim, "id"),
            Some(id) if id.units() == confession_id.units())
    });
    let target = match target {
        Some(claim)
            if matches!(get_text(claim, "kind"),
                Some(kind) if kind.units() == Text::from_str("confession").units()) =>
        {
            claim
        }
        _ => {
            let mut message = Msg::str("no confession with id ");
            message.push_units(confession_id.units());
            return Err(StoreError::Named(message));
        }
    };
    let defects = confession_defects(target)?;
    if let Some(first) = defects.first() {
        let mut message = Msg::str("confession ");
        message.push_units(confession_id.units());
        message.push_str(" is not a confession under WW-5: ");
        message.push_str(first);
        return Err(StoreError::Named(message));
    }
    let index = OutcomeIndex::new(&claims)?;
    let current = index.get(confession_id.units());
    if current.is_none() && superseded_ids(&claims).contains(&confession_id.units().to_vec()) {
        let mut message = Msg::str("confession ");
        message.push_units(confession_id.units());
        message.push_str(" has been superseded and cannot be graded");
        return Err(StoreError::Named(message));
    }
    if let Some(current) = current {
        let current_id = match get_text(current, "id") {
            Some(id) => id.clone(),
            None => return Err(StoreError::Crash),
        };
        if supersedes.map(|s| s.units()) != Some(current_id.units()) {
            let mut message = Msg::str("confession ");
            message.push_units(confession_id.units());
            message.push_str(" is already graded; a correction must supersede outcome ");
            message.push_units(current_id.units());
            return Err(StoreError::Named(message));
        }
    }
    if let Some(named) = supersedes {
        let matches_current = match current {
            Some(current) => matches!(get_text(current, "id"),
                Some(id) if id.units() == named.units()),
            None => false,
        };
        if !matches_current {
            let mut message = Msg::str("supersedes names no live outcome of confession ");
            message.push_units(confession_id.units());
            message.push_str(", ");
            message.push_units(named.units());
            return Err(StoreError::Named(message));
        }
    }
    let mut subject_units = Text::from_str("confession:").units().to_vec();
    subject_units.extend_from_slice(confession_id.units());
    store.add(
        happened,
        asserter,
        at,
        evidence,
        supersedes,
        Some(&Text::from_units(subject_units)),
        None,
        &[
            (
                Text::from_str("kind"),
                Value::Str(Text::from_str("outcome")),
            ),
            (Text::from_str("grade"), Value::Str(grade_value.clone())),
        ],
    )
}

/// The UTC day of an ISO timestamp, or the refusal
/// the views name (WW-9b).
fn utc_date(timestamp: &Text) -> Option<i64> {
    let plain = timestamp.as_str()?;
    let micros = flighttape::timeparse::parse_timestamp(&plain)?;
    Some(dates::utc_day_of(micros))
}

fn confession_state(
    claims: &[Value],
    index: &OutcomeIndex,
    confession: &Value,
    now_day: i64,
) -> Result<Value, StoreError> {
    let id = match get_text(confession, "id") {
        Some(id) => id.clone(),
        None => return Err(StoreError::Crash),
    };
    let text = match get_text(confession, "text") {
        Some(text) => text.clone(),
        None => return Err(StoreError::Crash),
    };
    let action = get(confession, "subject").cloned().unwrap_or(Value::Null);
    let due = get(confession, "due").cloned().unwrap_or(Value::Null);
    let mut grade = Value::Null;
    let state: &str;
    if let Some(outcome) = index.get(id.units()) {
        state = "graded";
        grade = get(outcome, "grade").cloned().unwrap_or(Value::Null);
    } else if superseded_ids(claims).contains(&id.units().to_vec()) {
        state = "superseded";
    } else {
        let overdue = match get(confession, "due") {
            Some(value) if py_truthy(value) => match as_text(value) {
                Some(due_text) => match due_text.as_str() {
                    Some(plain) => match dates::parse_date_wide(&plain) {
                        Some(day) => day < now_day,
                        None => return Err(StoreError::Crash),
                    },
                    None => return Err(StoreError::Crash),
                },
                None => return Err(StoreError::Crash),
            },
            _ => false,
        };
        state = if overdue { "overdue" } else { "awaiting" };
    }
    Ok(Value::Object(vec![
        (Text::from_str("id"), Value::Str(id)),
        (Text::from_str("action"), action),
        (Text::from_str("sentence"), Value::Str(text)),
        (Text::from_str("due"), due),
        (Text::from_str("grade"), grade),
        (Text::from_str("state"), Value::Str(Text::from_str(state))),
    ]))
}

/// The shared preamble of the views judged at a
/// caller-supplied now: the record must exist and
/// the time must parse. The caller resolves the
/// outcomes over the returned claims.
fn now_view(store: &Store, now: &Text) -> Result<(Vec<Value>, i64), StoreError> {
    if !store.exists()? {
        let mut message = Msg::str("no such record, ");
        message.push_msg(&crate::record::display_msg(&store.path));
        return Err(StoreError::Named(message));
    }
    let claims = store.claims_with(crate::record::BUDGET_LOAD_DEEP)?;
    let now_day = match utc_date(now) {
        Some(day) => day,
        None => {
            let mut message = Msg::str("now is not an ISO timestamp, ");
            message.push_units(now.units());
            return Err(StoreError::Named(message));
        }
    };
    Ok((claims, now_day))
}

pub fn ledger(store: &Store, now: &Text) -> Result<Value, StoreError> {
    let (claims, now_day) = now_view(store, now)?;
    let index = OutcomeIndex::new(&claims)?;
    let mut entries = Vec::new();
    let mut malformed = Vec::new();
    for confession in confessions(&claims) {
        entries.push(confession_state(&claims, &index, confession, now_day)?);
        let defects = confession_defects(confession)?;
        if !defects.is_empty() {
            let id = match get_text(confession, "id") {
                Some(id) => id.clone(),
                None => return Err(StoreError::Crash),
            };
            malformed.push(Value::Object(vec![
                (Text::from_str("id"), Value::Str(id)),
                (
                    Text::from_str("reasons"),
                    Value::Array(
                        defects
                            .iter()
                            .map(|defect| Value::Str(Text::from_str(defect)))
                            .collect(),
                    ),
                ),
            ]));
        }
    }
    let mut document = vec![(Text::from_str("confessions"), Value::Array(entries))];
    if !malformed.is_empty() {
        document.push((Text::from_str("malformed"), Value::Array(malformed)));
    }
    Ok(Value::Object(document))
}

/// One tally action row under construction: the
/// action's units (its sort key), the action value,
/// its per-grade counts, and its graded total.
struct ActionTally {
    units: Vec<u16>,
    action: Value,
    counts: Vec<(Text, i64)>,
    graded: i64,
}

pub fn tally(store: &Store, now: &Text) -> Result<Value, StoreError> {
    let (claims, now_day) = now_view(store, now)?;
    let index = OutcomeIndex::new(&claims)?;
    let mut by_state: Vec<(Text, i64)> = STATES
        .iter()
        .map(|state| (Text::from_str(state), 0))
        .collect();
    let mut by_grade: Vec<(Text, i64)> = GRADES
        .iter()
        .map(|grade| (Text::from_str(grade), 0))
        .collect();
    let mut actions: Vec<ActionTally> = Vec::new();
    let mut malformed: i64 = 0;
    let mut total: i64 = 0;
    for confession in confessions(&claims) {
        if !confession_defects(confession)?.is_empty() {
            malformed += 1;
            continue;
        }
        total += 1;
        let entry = confession_state(&claims, &index, confession, now_day)?;
        let state = match get_text(&entry, "state") {
            Some(state) => state.clone(),
            None => return Err(StoreError::Crash),
        };
        for (name, count) in by_state.iter_mut() {
            if name.units() == state.units() {
                *count += 1;
            }
        }
        if state.units() == Text::from_str("graded").units() {
            let grade = match get(&entry, "grade") {
                Some(value) => value.clone(),
                None => Value::Null,
            };
            for (name, count) in by_grade.iter_mut() {
                if let Value::Str(grade_text) = &grade {
                    if name.units() == grade_text.units() {
                        *count += 1;
                    }
                }
            }
            let action_value = get(&entry, "action").cloned().unwrap_or(Value::Null);
            let action_units = match &action_value {
                Value::Str(text) => text.units().to_vec(),
                _ => Vec::new(),
            };
            let held = actions.iter_mut().find(|row| row.units == action_units);
            match held {
                Some(row) => {
                    row.graded += 1;
                    if let Value::Str(grade_text) = &grade {
                        for (name, count) in row.counts.iter_mut() {
                            if name.units() == grade_text.units() {
                                *count += 1;
                            }
                        }
                    }
                }
                None => {
                    let mut counts: Vec<(Text, i64)> =
                        GRADES.iter().map(|g| (Text::from_str(g), 0)).collect();
                    if let Value::Str(grade_text) = &grade {
                        for (name, count) in counts.iter_mut() {
                            if name.units() == grade_text.units() {
                                *count += 1;
                            }
                        }
                    }
                    actions.push(ActionTally {
                        units: action_units,
                        action: action_value,
                        counts,
                        graded: 1,
                    });
                }
            }
        }
    }
    actions.sort_by(|a, b| a.units.cmp(&b.units));
    let action_rows: Vec<Value> = actions
        .into_iter()
        .map(|row| {
            let mut out = vec![
                (Text::from_str("action"), row.action),
                (Text::from_str("graded"), Value::Int(row.graded.to_string())),
            ];
            for (name, count) in row.counts {
                out.push((name, Value::Int(count.to_string())));
            }
            Value::Object(out)
        })
        .collect();
    let graded_total = by_state
        .iter()
        .find(|(name, _)| name.units() == Text::from_str("graded").units())
        .map(|(_, count)| *count)
        .unwrap_or(0);
    Ok(Value::Object(vec![
        (Text::from_str("total"), Value::Int(total.to_string())),
        (
            Text::from_str("by_state"),
            Value::Object(
                by_state
                    .into_iter()
                    .map(|(name, count)| (name, Value::Int(count.to_string())))
                    .collect(),
            ),
        ),
        (
            Text::from_str("graded"),
            Value::Int(graded_total.to_string()),
        ),
        (
            Text::from_str("by_grade"),
            Value::Object(
                by_grade
                    .into_iter()
                    .map(|(name, count)| (name, Value::Int(count.to_string())))
                    .collect(),
            ),
        ),
        (Text::from_str("actions"), Value::Array(action_rows)),
        (
            Text::from_str("malformed"),
            Value::Int(malformed.to_string()),
        ),
    ]))
}

pub fn wall(store: &Store, on: &Text) -> Result<Value, StoreError> {
    if !store.exists()? {
        let mut message = Msg::str("no such record, ");
        message.push_msg(&crate::record::display_msg(&store.path));
        return Err(StoreError::Named(message));
    }
    let claims = store.claims()?;
    let day = match on.as_str() {
        Some(plain) => match dates::parse_date_wide(&plain) {
            Some(day) => day,
            None => {
                let mut message = Msg::str("not an ISO date, ");
                message.push_units(on.units());
                return Err(StoreError::Named(message));
            }
        },
        None => {
            let mut message = Msg::str("not an ISO date, ");
            message.push_units(on.units());
            return Err(StoreError::Named(message));
        }
    };
    let (iso_year, iso_week, _) = dates::iso_calendar(day);
    let monday = day - dates::weekday_from_days(day) as i64;
    let sunday = monday + 6;
    let index = OutcomeIndex::new(&claims)?;
    let mut well_formed = Vec::new();
    for confession in confessions(&claims) {
        if confession_defects(confession)?.is_empty() {
            well_formed.push(confession);
        }
    }
    let mut broke = Vec::new();
    for confession in &well_formed {
        let id = match get_text(confession, "id") {
            Some(id) => id.clone(),
            None => return Err(StoreError::Crash),
        };
        let outcome = match index.get(id.units()) {
            Some(outcome) => outcome,
            None => continue,
        };
        if !matches!(get_text(outcome, "grade"),
            Some(grade) if grade.units() == Text::from_str("broke").units())
        {
            continue;
        }
        let asserted = match get_text(outcome, "asserted_at") {
            Some(text) => text.clone(),
            None => return Err(StoreError::Crash),
        };
        let landed = match utc_date(&asserted) {
            Some(day) => day,
            None => return Err(StoreError::Crash),
        };
        if monday <= landed && landed <= sunday {
            broke.push(Value::Object(vec![
                (
                    Text::from_str("action"),
                    get(confession, "subject").cloned().unwrap_or(Value::Null),
                ),
                (
                    Text::from_str("sentence"),
                    get(confession, "text").cloned().unwrap_or(Value::Null),
                ),
                (
                    Text::from_str("happened"),
                    get(outcome, "text").cloned().unwrap_or(Value::Null),
                ),
                (
                    Text::from_str("outcome"),
                    get(outcome, "id").cloned().unwrap_or(Value::Null),
                ),
            ]));
        }
    }
    let superseded = superseded_ids(&claims);
    let mut ungraded = Vec::new();
    for confession in &well_formed {
        let id = match get_text(confession, "id") {
            Some(id) => id.clone(),
            None => return Err(StoreError::Crash),
        };
        if index.get(id.units()).is_some() {
            continue;
        }
        if superseded.contains(&id.units().to_vec()) {
            continue;
        }
        if let Some(due_value) = get(confession, "due") {
            if !py_truthy(due_value) {
                continue;
            }
            let due_text = match as_text(due_value) {
                Some(text) => text.clone(),
                None => return Err(StoreError::Crash),
            };
            let due_day = match due_text.as_str() {
                Some(plain) => dates::parse_date_wide(&plain),
                None => None,
            };
            if let Some(due_day) = due_day {
                if monday <= due_day && due_day <= sunday {
                    ungraded.push(Value::Object(vec![
                        (
                            Text::from_str("action"),
                            get(confession, "subject").cloned().unwrap_or(Value::Null),
                        ),
                        (
                            Text::from_str("sentence"),
                            get(confession, "text").cloned().unwrap_or(Value::Null),
                        ),
                        (Text::from_str("due"), due_value.clone()),
                    ]));
                }
            }
        }
    }
    Ok(Value::Object(vec![
        (
            Text::from_str("week"),
            Value::Str(Text::from_str(&format!("{}-W{:02}", iso_year, iso_week))),
        ),
        (Text::from_str("broke"), Value::Array(broke)),
        (Text::from_str("ungraded"), Value::Array(ungraded)),
    ]))
}

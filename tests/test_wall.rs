//! WW-5 to WW-16 ported file for file from the
//! Python suite, plus the Round 3 pins. Claims are
//! flighttape values; documents are compared
//! rendered, so key order and rendering are part of
//! every assertion.

use flighttape::value::{Text, Value};
use tetlock::emit::{dumps_view, Mode};
use tetlock::record::{Store, StoreError};
use tetlock::wall::{confess, grade, ledger, tally, wall};

const CONF_A: &str = "claim-b0e7464b9254d2cb";
const CONF_B: &str = "claim-1008d801474cbae0";
const NOW: &str = "2026-03-01T00:00:00Z";

fn t(text: &str) -> Text {
    Text::from_str(text)
}

fn store_at(dir: &std::path::Path, name: &str) -> Store {
    Store::new(onrecord::store::RecPath::new(
        dir.join(name).to_str().expect("a utf8 path"),
    ))
}

fn fresh_record(dir: &std::path::Path) -> Store {
    let target = dir.join("record.jsonl");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/record_synthetic.jsonl");
    std::fs::copy(&fixture, &target).expect("the fixture copies");
    Store::new(onrecord::store::RecPath::new(
        target.to_str().expect("a utf8 path"),
    ))
}

fn err_text<T>(result: Result<T, StoreError>) -> String {
    match result {
        Ok(_) => panic!("expected a refusal"),
        Err(StoreError::Named(message)) => message.render_stderr(),
        Err(StoreError::Crash) => panic!("expected a named refusal, got the crash class"),
    }
}

fn render(document: &Value) -> String {
    // The Python tests compare json.dumps output, which
    // carries no trailing newline; the CLI adds it.
    let bytes = dumps_view(document, Mode::Strict).expect("the document renders");
    let text = String::from_utf8(bytes).expect("rendered views are utf8");
    text.strip_suffix('\n').map(str::to_string).unwrap_or(text)
}

fn field<'a>(value: &'a Value, name: &str) -> &'a Value {
    onrecord::store::field(value, name).expect("the field is present")
}

fn as_text(value: &Value) -> String {
    match value {
        Value::Str(text) => text.as_str().expect("plain text"),
        _ => panic!("expected a string"),
    }
}

fn as_int(value: &Value) -> i64 {
    match value {
        Value::Int(digits) => digits.parse().expect("an integer"),
        _ => panic!("expected an integer"),
    }
}

fn evidence(pairs: &[(&str, &str)]) -> Vec<(Text, Text)> {
    pairs.iter().map(|(s, r)| (t(s), t(r))).collect()
}

fn run_cli(args: &[&str]) -> (i32, String, String) {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tetlock"))
        .args(args)
        .output()
        .expect("the binary runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8(output.stdout).expect("stdout utf8"),
        String::from_utf8(output.stderr).expect("stderr utf8"),
    )
}

fn confess_base<'a>(
    store: &'a Store,
    action: &'a str,
    sentence: &'a str,
    due: &'a str,
) -> Result<String, StoreError> {
    confess(
        store,
        &t(action),
        &t(sentence),
        &t(due),
        &t("2026-01-05T09:00:00Z"),
        &t("agent"),
        &t("op-1"),
    )
}

fn grade_base(store: &Store, confession_id: &str, grade_value: &str) -> Result<String, StoreError> {
    grade(
        store,
        &t(confession_id),
        &t(grade_value),
        &t("What happened, stated plainly."),
        &t("2026-02-20T09:00:00Z"),
        &t("grader"),
        &evidence(&[("trace", "run-9")]),
        None,
    )
}

#[test]
fn the_digest_is_pinned_to_the_family_golden() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tape_synthetic.json");
    let text = std::fs::read_to_string(&fixture).expect("the fixture reads");
    let tape = flighttape::json::parse(&text).expect("the tape parses");
    let hex = onrecord::digest::digest_of(&tape, (990, 988)).expect("the digest renders");
    assert_eq!(
        hex,
        "cc9c4e01fca418dcc33114caf75fae1f6bf0d401f297b5d61f6776903fa955a3"
    );
}

#[test]
fn the_onrecord_minted_fixture_loads_sound() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    assert_eq!(store.check().expect("check runs").len(), 0);
    let claims = store.claims().expect("the fixture loads");
    let ids: Vec<String> = claims
        .iter()
        .map(|claim| as_text(field(claim, "id")))
        .collect();
    assert_eq!(
        ids,
        vec![
            CONF_A.to_string(),
            CONF_B.to_string(),
            "claim-2df18080a5b05ac3".to_string(),
            "claim-1fddf8798a175d0d".to_string()
        ]
    );
}

#[test]
fn the_sentence_rule_is_mechanical() {
    let dir = tempfile_dir();
    let store = store_at(&dir, "r.jsonl");
    let base = |sentence: &str| {
        confess(
            &store,
            &t("deploy"),
            &t(sentence),
            &t("2026-02-01"),
            &t("2026-01-05T09:00:00Z"),
            &t("agent"),
            &t("op-1"),
        )
    };
    assert!(err_text(base("Two\nlines.")).contains("one line"));
    assert!(err_text(base("No terminator")).contains("end in a period"));
    assert!(err_text(base("First. Then the rest.")).contains("internal sentence break"));
    assert!(err_text(base("Ships version 1.5 today.")).contains("internal sentence break"));
    let cid = base("The deploy finishes cleanly.").expect("a sound confession");
    assert!(cid.starts_with("claim-"));
}

#[test]
fn ledger_states_from_the_fixture() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    let document = ledger(&store, &t("2026-01-15T00:00:00Z")).expect("the ledger");
    let entries = match field(&document, "confessions") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    let state_of = |action: &str| {
        let entry = entries
            .iter()
            .find(|entry| as_text(field(entry, "action")) == action)
            .expect("the action is listed");
        (
            as_text(field(entry, "state")).to_string(),
            field(entry, "grade").clone(),
        )
    };
    assert_eq!(state_of("deploy-staging").0, "graded");
    assert_eq!(as_text(&state_of("deploy-staging").1), "broke");
    assert_eq!(state_of("drop-table").0, "awaiting");
    let later = ledger(&store, &t("2026-01-25T00:00:00Z")).expect("the ledger");
    let entries = match field(&later, "confessions") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    let drop = entries
        .iter()
        .find(|entry| as_text(field(entry, "action")) == "drop-table")
        .expect("listed");
    assert_eq!(as_text(field(drop, "state")), "overdue");
}

#[test]
fn the_wall_for_the_fixtures_week() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    let document = wall(&store, &t("2026-01-07")).expect("the wall");
    assert_eq!(as_text(field(&document, "week")), "2026-W02");
    assert_eq!(
        render(field(&document, "broke")),
        "[\n  {\n    \"action\": \"deploy-staging\",\n    \"sentence\": \"The staging deploy finishes without downtime.\",\n    \"happened\": \"The deploy rolled back after nine minutes down.\",\n    \"outcome\": \"claim-2df18080a5b05ac3\"\n  }\n]"
    );
    assert_eq!(render(field(&document, "ungraded")), "[]");
    let week3 = wall(&store, &t("2026-01-20")).expect("the wall");
    assert_eq!(as_text(field(&week3, "week")), "2026-W04");
    assert_eq!(render(field(&week3, "broke")), "[]");
    assert_eq!(
        render(field(&week3, "ungraded")),
        "[\n  {\n    \"action\": \"drop-table\",\n    \"sentence\": \"The legacy table drops cleanly in one pass.\",\n    \"due\": \"2026-01-20\"\n  }\n]"
    );
}

#[test]
fn grading_once_and_corrections_supersede() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    let ev = evidence(&[("trace", "run-9")]);
    let outcome = grade(
        &store,
        &t(CONF_B),
        &t("held"),
        &t("The table dropped in one pass."),
        &t("2026-01-19T09:00:00Z"),
        &t("grader"),
        &ev,
        None,
    )
    .expect("the first grade");
    let again = grade(
        &store,
        &t(CONF_B),
        &t("broke"),
        &t("Actually it failed."),
        &t("2026-01-19T10:00:00Z"),
        &t("grader"),
        &ev,
        None,
    );
    assert!(err_text(again).contains("already graded"));
    grade(
        &store,
        &t(CONF_B),
        &t("broke"),
        &t("Actually it failed."),
        &t("2026-01-19T11:00:00Z"),
        &t("grader"),
        &ev,
        Some(&t(&outcome)),
    )
    .expect("the correction");
    let document = ledger(&store, &t("2026-01-19T12:00:00Z")).expect("the ledger");
    let entries = match field(&document, "confessions") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    let drop = entries
        .iter()
        .find(|entry| as_text(field(entry, "action")) == "drop-table")
        .expect("listed");
    assert_eq!(as_text(field(drop, "grade")), "broke");
}

#[test]
fn grading_unknown_or_superseded_confessions_is_refused() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    let ev = evidence(&[("trace", "run-9")]);
    let ghost = grade(
        &store,
        &t("claim-ghost"),
        &t("held"),
        &t("x."),
        &t("2026-01-19T09:00:00Z"),
        &t("g"),
        &ev,
        None,
    );
    assert!(err_text(ghost).contains("no confession"));
    store
        .add(
            &t("The plan changed and the migration is off."),
            &t("operator"),
            &t("2026-01-06T10:00:00Z"),
            &evidence(&[("note", "n-1")]),
            Some(&t(CONF_B)),
            None,
            None,
            &[],
        )
        .expect("the superseding claim");
    let refused = grade(
        &store,
        &t(CONF_B),
        &t("held"),
        &t("x."),
        &t("2026-01-19T09:00:00Z"),
        &t("g"),
        &ev,
        None,
    );
    assert!(err_text(refused).contains("cannot be graded"));
    let document = ledger(&store, &t("2026-01-19T12:00:00Z")).expect("the ledger");
    let entries = match field(&document, "confessions") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    let drop = entries
        .iter()
        .find(|entry| as_text(field(entry, "action")) == "drop-table")
        .expect("listed");
    assert_eq!(as_text(field(drop, "state")), "superseded");
}

#[test]
fn a_graded_confession_keeps_its_grade_when_superseded() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    store
        .add(
            &t("The deploy plan was replaced wholesale."),
            &t("operator"),
            &t("2026-01-08T10:00:00Z"),
            &evidence(&[("note", "n-2")]),
            Some(&t(CONF_A)),
            None,
            None,
            &[],
        )
        .expect("the superseding claim");
    let document = ledger(&store, &t("2026-01-19T12:00:00Z")).expect("the ledger");
    let entries = match field(&document, "confessions") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    let deploy = entries
        .iter()
        .find(|entry| as_text(field(entry, "action")) == "deploy-staging")
        .expect("listed");
    assert_eq!(as_text(field(deploy, "state")), "graded");
    assert_eq!(as_text(field(deploy, "grade")), "broke");
    let walled = wall(&store, &t("2026-01-07")).expect("the wall");
    let broke = match field(&walled, "broke") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    assert_eq!(broke.len(), 1);
    assert_eq!(as_text(field(&broke[0], "action")), "deploy-staging");
}

#[test]
fn an_outcome_superseded_by_a_plain_claim_still_governs() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    store
        .add(
            &t("A remark about the deploy outcome."),
            &t("operator"),
            &t("2026-01-08T09:00:00Z"),
            &evidence(&[("note", "n-3")]),
            Some(&t("claim-2df18080a5b05ac3")),
            None,
            None,
            &[],
        )
        .expect("the remark");
    let document = ledger(&store, &t("2026-01-15T00:00:00Z")).expect("the ledger");
    let entries = match field(&document, "confessions") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    let deploy = entries
        .iter()
        .find(|entry| as_text(field(entry, "action")) == "deploy-staging")
        .expect("listed");
    assert_eq!(as_text(field(deploy, "grade")), "broke");
}

#[test]
fn two_live_outcomes_refuse_the_view_by_name() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    store
        .add(
            &t("It held after all."),
            &t("second-grader"),
            &t("2026-01-08T09:00:00Z"),
            &evidence(&[("trace", "run-40")]),
            None,
            Some(&t(&format!("confession:{}", CONF_A))),
            None,
            &[
                (t("kind"), Value::Str(t("outcome"))),
                (t("grade"), Value::Str(t("held"))),
            ],
        )
        .expect("the second outcome");
    let refused = ledger(&store, &t("2026-01-15T00:00:00Z"));
    assert!(err_text(refused).contains("two live outcomes"));
}

#[test]
fn a_malformed_confession_is_named_and_never_walled() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    let forged = store
        .add(
            &t("The cache clears itself nightly."),
            &t("mallory"),
            &t("2026-01-05T09:00:00Z"),
            &evidence(&[("hunch", "none")]),
            None,
            Some(&t("cache-clear")),
            Some(&t("2026-01-06")),
            &[(t("kind"), Value::Str(t("confession")))],
        )
        .expect("the forged confession");
    let document = ledger(&store, &t("2026-01-15T00:00:00Z")).expect("the ledger");
    let entries = match field(&document, "confessions") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    let actions: Vec<String> = entries
        .iter()
        .map(|entry| as_text(field(entry, "action")))
        .collect();
    assert_eq!(
        actions,
        vec![
            "deploy-staging".to_string(),
            "drop-table".to_string(),
            "cache-clear".to_string()
        ]
    );
    assert_eq!(
        render(field(&document, "malformed")),
        format!(
            "[\n  {{\n    \"id\": \"{}\",\n    \"reasons\": [\n      \"no approval evidence\"\n    ]\n  }}\n]",
            forged
        )
    );
    let walled = wall(&store, &t("2026-01-06")).expect("the wall");
    assert_eq!(render(field(&walled, "ungraded")), "[]");
    let refused = grade(
        &store,
        &t(&forged),
        &t("broke"),
        &t("It did not."),
        &t("2026-01-07T09:00:00Z"),
        &t("g"),
        &evidence(&[("trace", "r-1")]),
        None,
    );
    assert!(err_text(refused).contains("not a confession"));
}

#[test]
fn week_boundaries_and_utc_reading() {
    let dir = tempfile_dir();
    let store = store_at(&dir, "r.jsonl");
    let cid = confess_base(&store, "deploy", "The Sunday deploy holds.", "2026-01-11")
        .expect("the confession");
    grade(
        &store,
        &t(&cid),
        &t("broke"),
        &t("It failed late Sunday."),
        &t("2026-01-11T23:30:00Z"),
        &t("g"),
        &evidence(&[("trace", "r-5")]),
        None,
    )
    .expect("the grade");
    let walled = wall(&store, &t("2026-01-11")).expect("the wall");
    assert_ne!(render(field(&walled, "broke")), "[]");
    let next = wall(&store, &t("2026-01-12")).expect("the wall");
    assert_eq!(render(field(&next, "broke")), "[]");
    let store2 = store_at(&dir, "r2.jsonl");
    let cid2 = confess_base(&store2, "deploy", "The Sunday deploy holds.", "2026-01-11")
        .expect("the confession");
    grade(
        &store2,
        &t(&cid2),
        &t("broke"),
        &t("It failed late Sunday."),
        &t("2026-01-12T13:30:00+14:00"),
        &t("g"),
        &evidence(&[("trace", "r-5")]),
        None,
    )
    .expect("the grade");
    let walled2 = wall(&store2, &t("2026-01-11")).expect("the wall");
    assert_ne!(render(field(&walled2, "broke")), "[]");
}

#[test]
fn held_confessions_never_reach_the_wall() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    grade(
        &store,
        &t(CONF_B),
        &t("held"),
        &t("The table dropped in one pass."),
        &t("2026-01-19T09:00:00Z"),
        &t("grader"),
        &evidence(&[("trace", "run-9")]),
        None,
    )
    .expect("the grade");
    let document = wall(&store, &t("2026-01-20")).expect("the wall");
    assert_eq!(render(field(&document, "broke")), "[]");
    assert_eq!(render(field(&document, "ungraded")), "[]");
}

#[test]
fn a_view_never_overwrites_the_record() {
    let dir = tempfile_dir();
    let target = dir.join("record.jsonl");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/record_synthetic.jsonl");
    std::fs::copy(&fixture, &target).expect("the fixture copies");
    let before = std::fs::read(&target).expect("reads");
    let path = target.to_str().expect("utf8");
    let (code, _out, err) = run_cli(&["wall", path, "--date", "2026-01-07", "-o", path]);
    assert_eq!(code, 1);
    assert!(err.contains("never overwrites the record"));
    assert_eq!(std::fs::read(&target).expect("reads"), before);
}

#[test]
fn views_are_deterministic() {
    let dir = tempfile_dir();
    let store = fresh_record(&dir);
    let first = ledger(&store, &t("2026-01-15T00:00:00Z")).expect("the ledger");
    let second = ledger(&store, &t("2026-01-15T00:00:00Z")).expect("the ledger");
    assert_eq!(render(&first), render(&second));
    let w1 = wall(&store, &t("2026-01-07")).expect("the wall");
    let w2 = wall(&store, &t("2026-01-08")).expect("the wall");
    assert_eq!(render(&w1), render(&w2));
}

#[test]
fn a_defective_record_is_refused_with_defects_named() {
    let dir = tempfile_dir();
    let target = dir.join("record.jsonl");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/record_synthetic.jsonl");
    std::fs::copy(&fixture, &target).expect("the fixture copies");
    use std::io::Write;
    let mut handle = std::fs::OpenOptions::new()
        .append(true)
        .open(&target)
        .expect("opens");
    handle
        .write_all(b"{\"id\": \"claim-forged\", \"text\": \"x\"}\n")
        .expect("appends");
    let store = Store::new(onrecord::store::RecPath::new(
        target.to_str().expect("utf8"),
    ));
    let refused = ledger(&store, &t("2026-01-15T00:00:00Z"));
    assert!(err_text(refused).contains("defects"));
}

#[test]
fn the_cli_round_trip() {
    let dir = tempfile_dir();
    let path = dir.join("r.jsonl");
    let path = path.to_str().expect("utf8");
    let (code, out, _err) = run_cli(&[
        "confess",
        path,
        "--action",
        "deploy",
        "--sentence",
        "The deploy finishes cleanly.",
        "--due",
        "2026-02-01",
        "--at",
        "2026-01-05T09:00:00Z",
        "--asserter",
        "agent",
        "--approval",
        "op-1",
    ]);
    assert_eq!(code, 0);
    let cid = out.trim().to_string();
    let (code, _out, _err) = run_cli(&[
        "grade",
        path,
        "--confession",
        &cid,
        "--grade",
        "broke",
        "--happened",
        "It rolled back.",
        "--at",
        "2026-01-30T09:00:00Z",
        "--asserter",
        "g",
        "--evidence",
        "trace:run-2",
    ]);
    assert_eq!(code, 0);
    let (code, out, _err) = run_cli(&["wall", path, "--date", "2026-01-30"]);
    assert_eq!(code, 0);
    assert!(out.contains("\"action\": \"deploy\""));
    let (code, _out, err) = run_cli(&[
        "confess",
        path,
        "--action",
        "x",
        "--sentence",
        "Two. Sentences.",
        "--due",
        "2026-02-01",
        "--at",
        "2026-01-05T09:00:00Z",
        "--asserter",
        "agent",
        "--approval",
        "op-1",
    ]);
    assert_eq!(code, 1);
    assert!(err.contains("internal sentence break"));
}

fn population(dir: &std::path::Path) -> Store {
    let store = store_at(dir, "r.jsonl");
    let held = confess_base(&store, "deploy", "The deploy holds.", "2026-02-01").expect("confess");
    grade_base(&store, &held, "held").expect("grade");
    let broke =
        confess_base(&store, "deploy", "The deploy is quick.", "2026-02-01").expect("confess");
    grade_base(&store, &broke, "broke").expect("grade");
    confess_base(&store, "deploy", "The deploy waits.", "2026-04-01").expect("confess");
    let changed = confess_base(&store, "deploy", "The plan ships.", "2026-02-01").expect("confess");
    store
        .add(
            &t("The plan changed and is off."),
            &t("operator"),
            &t("2026-01-06T10:00:00Z"),
            &evidence(&[("note", "n-1")]),
            Some(&t(&changed)),
            None,
            None,
            &[],
        )
        .expect("the change");
    let unv = confess_base(
        &store,
        "drop-table",
        "The table drops in one pass.",
        "2026-02-01",
    )
    .expect("confess");
    grade_base(&store, &unv, "unverifiable").expect("grade");
    confess_base(&store, "drop-table", "The index rebuilds.", "2026-01-10").expect("confess");
    store
        .add(
            &t("No approval stands behind this."),
            &t("agent"),
            &t("2026-01-05T09:00:00Z"),
            &evidence(&[("trace", "run-9")]),
            None,
            Some(&t("deploy")),
            Some(&t("2026-02-01")),
            &[(t("kind"), Value::Str(t("confession")))],
        )
        .expect("the malformed confession");
    store
}

#[test]
fn the_tally_counts_states_grades_and_actions() {
    let dir = tempfile_dir();
    let store = population(&dir);
    let document = tally(&store, &t(NOW)).expect("the tally");
    assert_eq!(as_int(field(&document, "total")), 6);
    assert_eq!(
        render(field(&document, "by_state")),
        "{\n  \"graded\": 3,\n  \"awaiting\": 1,\n  \"overdue\": 1,\n  \"superseded\": 1\n}"
    );
    assert_eq!(as_int(field(&document, "graded")), 3);
    assert_eq!(
        render(field(&document, "by_grade")),
        "{\n  \"held\": 1,\n  \"broke\": 1,\n  \"unverifiable\": 1\n}"
    );
    assert_eq!(
        render(field(&document, "actions")),
        "[\n  {\n    \"action\": \"deploy\",\n    \"graded\": 2,\n    \"held\": 1,\n    \"broke\": 1,\n    \"unverifiable\": 0\n  },\n  {\n    \"action\": \"drop-table\",\n    \"graded\": 1,\n    \"held\": 0,\n    \"broke\": 0,\n    \"unverifiable\": 1\n  }\n]"
    );
    assert_eq!(as_int(field(&document, "malformed")), 1);
}

#[test]
fn the_tallys_arithmetic_closes() {
    let dir = tempfile_dir();
    let store = population(&dir);
    let document = tally(&store, &t(NOW)).expect("the tally");
    let by_state = field(&document, "by_state");
    let state_sum: i64 = ["graded", "awaiting", "overdue", "superseded"]
        .iter()
        .map(|name| as_int(field(by_state, name)))
        .sum();
    assert_eq!(state_sum, as_int(field(&document, "total")));
    let by_grade = field(&document, "by_grade");
    let grade_sum: i64 = ["held", "broke", "unverifiable"]
        .iter()
        .map(|name| as_int(field(by_grade, name)))
        .sum();
    assert_eq!(grade_sum, as_int(field(&document, "graded")));
    let actions = match field(&document, "actions") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    for grade in ["held", "broke", "unverifiable"] {
        let row_sum: i64 = actions.iter().map(|row| as_int(field(row, grade))).sum();
        assert_eq!(row_sum, as_int(field(by_grade, grade)));
    }
}

#[test]
fn the_tally_of_an_empty_record_is_all_zeroes() {
    let dir = tempfile_dir();
    let path = dir.join("empty.jsonl");
    std::fs::write(&path, "").expect("the empty record");
    let store = Store::new(onrecord::store::RecPath::new(path.to_str().expect("utf8")));
    let document = tally(&store, &t(NOW)).expect("the tally");
    assert_eq!(
        render(&document),
        "{\n  \"total\": 0,\n  \"by_state\": {\n    \"graded\": 0,\n    \"awaiting\": 0,\n    \"overdue\": 0,\n    \"superseded\": 0\n  },\n  \"graded\": 0,\n  \"by_grade\": {\n    \"held\": 0,\n    \"broke\": 0,\n    \"unverifiable\": 0\n  },\n  \"actions\": [],\n  \"malformed\": 0\n}"
    );
}

#[test]
fn the_tally_command_and_its_refusals() {
    let dir = tempfile_dir();
    let store = population(&dir);
    let path = store.path.os.to_str().expect("utf8").to_string();
    let out = dir.join("tally.json");
    let out = out.to_str().expect("utf8");
    let (code, _o, _e) = run_cli(&["tally", &path, "--now", NOW, "-o", out]);
    assert_eq!(code, 0);
    let text = std::fs::read_to_string(out).expect("the tally file");
    assert!(text.contains("\"total\": 6"));
    let missing = dir.join("missing.jsonl");
    let (code, _o, err) = run_cli(&["tally", missing.to_str().expect("utf8"), "--now", NOW]);
    assert_eq!(code, 1);
    assert!(err.contains("no such record"));
    let (code, _o, err) = run_cli(&["tally", &path, "--now", "next week"]);
    assert_eq!(code, 1);
    assert!(err.contains("not an ISO timestamp"));
}

#[test]
fn action_rows_order_by_name_not_appearance() {
    let dir = tempfile_dir();
    let store = store_at(&dir, "r.jsonl");
    let zebra = confess_base(&store, "zebra", "The zebra ships.", "2026-02-01").expect("confess");
    grade_base(&store, &zebra, "broke").expect("grade");
    let apple = confess_base(&store, "apple", "The apple ships.", "2026-02-01").expect("confess");
    grade_base(&store, &apple, "held").expect("grade");
    confess_base(&store, "someday", "The someday ships.", "2026-04-01").expect("confess");
    let document = tally(&store, &t(NOW)).expect("the tally");
    let actions = match field(&document, "actions") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    let names: Vec<String> = actions
        .iter()
        .map(|row| as_text(field(row, "action")))
        .collect();
    assert_eq!(names, vec!["apple".to_string(), "zebra".to_string()]);
    assert_eq!(as_int(field(field(&document, "by_state"), "awaiting")), 1);
}

#[test]
fn now_is_read_in_utc_at_the_boundary() {
    let dir = tempfile_dir();
    let store = store_at(&dir, "r.jsonl");
    confess_base(&store, "deploy", "The deploy holds.", "2026-02-28").expect("confess");
    let document = tally(&store, &t("2026-02-28T23:30:00-02:00")).expect("the tally");
    assert_eq!(as_int(field(field(&document, "by_state"), "overdue")), 1);
    assert_eq!(as_int(field(field(&document, "by_state"), "awaiting")), 0);
}

#[test]
fn the_tally_is_deterministic_and_refuses_bad_outcomes() {
    let dir = tempfile_dir();
    let store = population(&dir);
    let first = tally(&store, &t(NOW)).expect("the tally");
    let second = tally(&store, &t(NOW)).expect("the tally");
    assert_eq!(render(&first), render(&second));
    let claims = store.claims().expect("claims");
    let target = claims
        .iter()
        .find(|claim| {
            matches!(onrecord::store::field(claim, "kind"),
                Some(Value::Str(kind)) if kind.units() == t("confession").units())
                && matches!(onrecord::store::field(claim, "subject"),
                Some(Value::Str(subject)) if subject.units() == t("deploy").units())
        })
        .expect("a deploy confession");
    let target_id = as_text(field(target, "id")).to_string();
    store
        .add(
            &t("A second live outcome lands."),
            &t("grader"),
            &t("2026-02-21T09:00:00Z"),
            &evidence(&[("trace", "run-9")]),
            None,
            Some(&t(&format!("confession:{}", target_id))),
            None,
            &[
                (t("kind"), Value::Str(t("outcome"))),
                (t("grade"), Value::Str(t("held"))),
            ],
        )
        .expect("the second outcome");
    assert!(tally(&store, &t(NOW)).is_err());
}

#[test]
fn a_graded_confession_stays_graded_when_superseded() {
    let dir = tempfile_dir();
    let store = store_at(&dir, "r.jsonl");
    let cid = confess_base(&store, "deploy", "The deploy holds.", "2026-02-01").expect("confess");
    grade_base(&store, &cid, "broke").expect("grade");
    store
        .add(
            &t("The plan was replaced wholesale."),
            &t("operator"),
            &t("2026-02-22T10:00:00Z"),
            &evidence(&[("note", "n-2")]),
            Some(&t(&cid)),
            None,
            None,
            &[],
        )
        .expect("the replacement");
    let document = tally(&store, &t(NOW)).expect("the tally");
    assert_eq!(as_int(field(field(&document, "by_state"), "graded")), 1);
    assert_eq!(as_int(field(field(&document, "by_state"), "superseded")), 0);
    assert_eq!(as_int(field(field(&document, "by_grade"), "broke")), 1);
}

#[test]
fn a_sentence_breaking_confession_is_malformed() {
    let dir = tempfile_dir();
    let store = store_at(&dir, "r.jsonl");
    let forged = store
        .add(
            &t("Two. Sentences."),
            &t("agent"),
            &t("2026-01-05T09:00:00Z"),
            &evidence(&[("approval", "op-1")]),
            None,
            Some(&t("deploy")),
            Some(&t("2026-02-01")),
            &[(t("kind"), Value::Str(t("confession")))],
        )
        .expect("the forged confession");
    let blank = store
        .add(
            &t("A fine sentence."),
            &t("agent"),
            &t("2026-01-05T09:00:00Z"),
            &evidence(&[("approval", "op-1")]),
            None,
            Some(&t(" ")),
            Some(&t("2026-02-01")),
            &[(t("kind"), Value::Str(t("confession")))],
        )
        .expect("the blank subject confession");
    let document = tally(&store, &t(NOW)).expect("the tally");
    assert_eq!(as_int(field(&document, "total")), 0);
    assert_eq!(as_int(field(&document, "malformed")), 2);
    let refused = grade_base(&store, &forged, "held");
    assert!(refused.is_err());
    let led = ledger(&store, &t(NOW)).expect("the ledger");
    let malformed = match field(&led, "malformed") {
        Value::Array(items) => items,
        _ => panic!("an array"),
    };
    let reasons_of = |id: &str| -> Vec<String> {
        let entry = malformed
            .iter()
            .find(|entry| as_text(field(entry, "id")) == id)
            .expect("the malformed entry");
        match field(entry, "reasons") {
            Value::Array(items) => items.iter().map(|r| as_text(r).to_string()).collect(),
            _ => panic!("an array"),
        }
    };
    assert!(reasons_of(&forged)
        .iter()
        .any(|r| r.contains("internal sentence break")));
    assert!(reasons_of(&blank).iter().any(|r| r.contains("no subject")));
}

#[test]
fn blank_names_are_refused_at_write_and_read() {
    let dir = tempfile_dir();
    let store = store_at(&dir, "r.jsonl");
    let base = |action: &str, approval: &str| {
        confess(
            &store,
            &t(action),
            &t("The deploy holds."),
            &t("2026-02-01"),
            &t("2026-01-05T09:00:00Z"),
            &t("agent"),
            &t(approval),
        )
    };
    assert!(err_text(base(" ", "op-1")).contains("action names"));
    assert!(err_text(base("deploy", "  ")).contains("approval names"));
    assert!(!dir.join("r.jsonl").exists());
    store
        .add(
            &t("A fine sentence."),
            &t("agent"),
            &t("2026-01-05T09:00:00Z"),
            &evidence(&[("approval", " ")]),
            None,
            Some(&t("deploy")),
            Some(&t("2026-02-01")),
            &[(t("kind"), Value::Str(t("confession")))],
        )
        .expect("the blank approval confession");
    let document = tally(&store, &t(NOW)).expect("the tally");
    assert_eq!(as_int(field(&document, "total")), 0);
    assert_eq!(as_int(field(&document, "malformed")), 1);
}

#[test]
fn closed_at_launch_reads_the_runtime_substitution() {
    // The runtime repairs a closed standard stream
    // with a read-write /dev/null; a /dev/null the
    // caller opened for writing is a real stream.
    // The detector must tell them apart, because a
    // command's whole result rides on it.
    use std::os::unix::io::AsRawFd;
    let both = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/null")
        .expect("dev null opens");
    assert!(tetlock::util::closed_at_launch(both.as_raw_fd() as u32));
    let write_only = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/null")
        .expect("dev null opens");
    assert!(!tetlock::util::closed_at_launch(
        write_only.as_raw_fd() as u32
    ));
}

fn tempfile_dir() -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/test-tmp")
        .join(format!(
            "ww-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
    std::fs::create_dir_all(&dir).expect("the temp dir");
    dir
}

static TEMP_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

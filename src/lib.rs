//! wallofwrong: confessions graded against what
//! actually happened. Confessions are onrecord
//! claims with due dates; the wall is a view over
//! the record, never a second store (WW-1 to
//! WW-16).
//!
//! Round 3 is the Rust port. The record layer is
//! consumed from the onrecord crate at a pinned
//! revision (WW-18): the claim shape, the defect
//! discipline, the id derivation, and the loader
//! are the crate's functions. This crate's Store
//! is an adapter over them carrying this repo's
//! own surface (claims, add with the extra field
//! rule, check) and its own append.

pub mod cli;
pub mod dates;
pub mod emit;
pub mod helptext;
pub mod pyrepr;
pub mod record;
pub mod repr_table;
pub mod util;
pub mod wall;

/// The command line entry, in the cli module.
pub fn run_cli() -> i32 {
    // A generous stack, on the flighttape crate's own
    // precedent: parsed values nest as deep as their
    // files, and cloning or dropping a deep value
    // recurses with its depth.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(crate::cli::run_cli)
        .expect("the CLI thread starts");
    child.join().unwrap_or(1)
}

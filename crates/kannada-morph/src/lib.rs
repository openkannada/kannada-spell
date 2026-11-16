//! Kannada inflection, morphological analysis and a Hunspell spelling dictionary.
//!
//! The paradigms in [`noun`] and [`verb`] describe how Kannada words inflect.
//! [`rules`] reads affix rules off them, which drive both the [`analyzer`] and
//! the [`hunspell`] export, so the 2 always accept the same words.

pub mod analyzer;
pub mod class;
pub mod classify;
pub mod form;
pub mod hunspell;
pub mod lexicon;
pub mod noun;
pub mod rules;
pub mod sandhi;
pub mod script;
pub mod verb;

pub use analyzer::{Analysis, Analyzer};
pub use class::Class;
pub use lexicon::{Entry, Lexicon};

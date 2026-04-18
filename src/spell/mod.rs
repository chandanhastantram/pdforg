pub mod aff;
pub mod checker;

pub use checker::{SpellChecker, SpellResult};

pub fn default_checker() -> SpellChecker {
    SpellChecker::new_minimal_english()
}

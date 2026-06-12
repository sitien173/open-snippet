//! Trigger matching against typed input.

pub mod automaton;
mod boundary;
mod buffer;

pub use automaton::{MatchHit, Matcher};
pub use boundary::{classify_boundary_char, BoundaryState};
pub use buffer::{MatchBuffer, Reset};

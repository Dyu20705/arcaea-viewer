//! Shared domain primitives and stable contracts for Arcaea-Viewer.
//!
//! This crate owns parser-independent chart concepts that later parser, timing,
//! analytics, and WASM crates can share without depending on browser, file I/O,
//! rendering, or source-text details.

mod chart;
mod coordinate;
mod error;
mod event;
mod lane;
mod tempo;
mod time;

pub use chart::Chart;
pub use coordinate::{ArcX, ArcY};
pub use error::{CoordinateError, IntervalError, LaneError, TempoError};
pub use event::{
    ArcColor, ArcCurve, ArcNote, ArcPath, ChartEvent, HoldNote, NoteId, TapNote, TimingEvent,
};
pub use lane::Lane;
pub use tempo::Tempo;
pub use time::ChartTime;

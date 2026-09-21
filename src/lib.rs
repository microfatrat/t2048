//! A 2048 clone for the terminal, built with ratatui.
//!
//! The crate is split so the rules can be tested without a terminal:
//!
//! - [`game`] holds the rules and knows nothing about rendering.
//! - [`app`] holds the application state and turns key presses into moves.
//! - [`ui`] draws an [`app::App`] with ratatui.
//! - [`theme`] holds the colour palette.
//! - [`storage`] remembers the best score between runs.
//!
//! The `t2048` binary in `src/main.rs` is a thin wrapper that wires these
//! together with a terminal event loop.

pub mod app;
pub mod game;
pub mod storage;
pub mod theme;
pub mod ui;

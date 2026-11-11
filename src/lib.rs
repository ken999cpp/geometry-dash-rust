//! Deterministic game logic, independent of the desktop renderer.
pub mod constants;
pub mod editor;
pub mod geometry;
pub mod level;
pub mod simulation;
pub mod storage;

#[cfg(feature = "desktop")]
pub mod desktop;

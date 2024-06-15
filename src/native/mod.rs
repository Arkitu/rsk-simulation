//! Contains all the native specific parts plus the default parts

#[cfg(feature = "native_gui")]
pub mod gui;

#[cfg(feature = "native_control")]
pub mod control;

#[cfg(feature = "native_gc")]
pub mod gc;
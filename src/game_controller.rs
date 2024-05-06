use std::{cell::RefCell, rc::Rc};

use rapier2d_f64::prelude::*;
use crate::game_state::{GameState, Robot};

#[cfg(feature = "standard_gc")]
mod standard;

#[cfg(feature = "http_gc")]
mod http;

#[cfg(all(feature = "standard_gc", feature = "http_gc"))]
compile_error!("Multiple game controller features enabled. You can only enable one game controller feature.");

#[cfg(feature = "standard_gc")]
pub use standard::GC;

#[cfg(feature = "http_gc")]
pub use http::GC;

#[cfg(not(any(feature = "standard_gc", feature = "http_gc")))]
compile_error!(
    "No game controller feature enabled. You need to enable at least one game controller feature."
);

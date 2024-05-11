#[cfg(feature = "standard_gc")]
mod standard;

#[cfg(feature = "http_client_gc")]
mod http_client;

#[cfg(all(feature = "standard_gc", feature = "http_client_gc"))]
compile_error!(
    "Multiple game controller features enabled. You can only enable one game controller feature."
);

#[cfg(feature = "standard_gc")]
pub use standard::GC;

#[cfg(feature = "http_client_gc")]
pub use http_client::GC;

#[cfg(not(any(feature = "standard_gc", feature = "http_client_gc")))]
compile_error!(
    "No game controller feature enabled. You need to enable at least one game controller feature."
);

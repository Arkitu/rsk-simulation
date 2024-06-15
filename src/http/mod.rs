#[cfg(any(feature = "http_client", feature = "http_server"))]
pub mod default;
#[cfg(any(feature = "alternative_http_client", feature = "alternative_http_server"))]
pub mod alternative;
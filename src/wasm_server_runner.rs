/// Copy of wasm-server-runner (https://github.com/jakobhellermann/wasm-server-runner/)
mod wasm_bindgen {
    use super::server::Options;
    use super::Result;
    use std::collections::HashMap;
    use std::path::Path;
    use tracing::debug;

    pub struct WasmBindgenOutput {
        pub js: String,
        pub wasm: Vec<u8>,
        pub snippets: HashMap<String, Vec<String>>,
        pub local_modules: HashMap<String, String>,
    }
    pub fn generate(options: &Options, wasm_file: &Path) -> Result<WasmBindgenOutput> {
        debug!("running wasm-bindgen...");
        let start = std::time::Instant::now();
        let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
        bindgen
            .input_path(wasm_file)
            .typescript(false)
            .reference_types(true);

        if options.no_module {
            bindgen.no_modules(true)?;
        } else {
            bindgen.web(true)?;
        }

        let mut output = bindgen.generate_output()?;
        debug!("finished wasm-bindgen (took {:?})", start.elapsed());

        let js = output.js().to_owned();
        let snippets = output.snippets().clone();
        let local_modules = output.local_modules().clone();

        debug!("emitting wasm...");
        let start = std::time::Instant::now();
        let wasm = output.wasm_mut().emit_wasm();
        debug!("emitting wasm took {:?}", start.elapsed());

        Ok(WasmBindgenOutput {
            js,
            wasm,
            snippets,
            local_modules,
        })
    }
}
mod server {
    mod certificate {
        use std::io::ErrorKind;
        use std::path::Path;

        use super::super::Result;
        use directories::ProjectDirs;

        pub struct Certificate {
            pub certificate: Vec<u8>,
            pub private_key: Vec<u8>,
        }

        pub fn certificate() -> Result<Certificate> {
            let directories = match ProjectDirs::from("", "", "wasm-server-runner") {
                Some(directories) => directories,
                None => {
                    tracing::warn!("failed to determine application directory");
                    return generate();
                }
            };

            let path = directories.data_local_dir();

            let certificate = match read(&path.join("certificate.der")) {
                Ok(Some(certificate)) => certificate,
                Ok(None) => return generate_in(path),
                Err(()) => return generate(),
            };

            let private_key = match read(&path.join("private_key.der")) {
                Ok(Some(private_key)) => private_key,
                Ok(None) => return generate_in(path),
                Err(()) => return generate(),
            };

            tracing::info!("re-using certificate from \"{}\"", path.display());

            Ok(Certificate {
                certificate,
                private_key,
            })
        }

        fn read(path: &Path) -> Result<Option<Vec<u8>>, ()> {
            match std::fs::read(path) {
                Ok(file) => Ok(Some(file)),
                Err(error) => {
                    if error.kind() == ErrorKind::NotFound {
                        Ok(None)
                    } else {
                        tracing::error!("error reading file from \"{}\": {error}", path.display());
                        Err(())
                    }
                }
            }
        }

        fn write(path: &Path, data: &[u8]) -> Result<(), ()> {
            match std::fs::write(path, data) {
                Ok(()) => Ok(()),
                Err(error) => {
                    tracing::error!("error saving file to \"{}\": {error}", path.display());
                    Err(())
                }
            }
        }

        fn generate() -> Result<Certificate> {
            tracing::warn!("generated temporary certificate");

            generate_internal()
        }

        fn generate_in(path: &Path) -> Result<Certificate> {
            let certificate = generate_internal()?;

            if let Err(error) = std::fs::create_dir_all(path) {
                tracing::error!("error creating directory \"{}\": {error}", path.display());
                tracing::warn!("generated temporary certificate");
                return Ok(certificate);
            }

            if let Err(()) = write(&path.join("certificate.der"), &certificate.certificate)
                .and_then(|_| write(&path.join("private_key.der"), &certificate.private_key))
            {
                tracing::warn!("generated temporary certificate");
                return Ok(certificate);
            }

            tracing::info!("generated new certificate in \"{}\"", path.display());
            Ok(certificate)
        }

        fn generate_internal() -> Result<Certificate> {
            let certificate = rcgen::generate_simple_self_signed([String::from("localhost")])?;

            Ok(Certificate {
                certificate: certificate.serialize_der()?,
                private_key: certificate.serialize_private_key_der(),
            })
        }
    }

    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::path::PathBuf;

    use axum::error_handling::HandleError;
    use axum::extract::ws::{self, WebSocket};
    use axum::extract::{Path, WebSocketUpgrade};
    use axum::http::{HeaderValue, StatusCode};
    use axum::response::{Html, IntoResponse, Response};
    use axum::routing::{get, get_service};
    use axum::Router;
    use axum_server::tls_rustls::RustlsConfig;
    use axum_server_dual_protocol::ServerExt;
    use http::HeaderName;
    use tower::ServiceBuilder;
    use tower_http::compression::CompressionLayer;
    use tower_http::services::ServeDir;
    use tower_http::set_header::SetResponseHeaderLayer;

    use super::wasm_bindgen::WasmBindgenOutput;
    use super::Result;

    fn generate_version() -> String {
        std::iter::repeat_with(fastrand::alphanumeric)
            .take(12)
            .collect()
    }

    pub struct Options {
        pub title: String,
        pub address: String,
        pub directory: PathBuf,
        pub custom_index_html: Option<PathBuf>,
        pub https: bool,
        pub no_module: bool,
    }

    pub async fn run_server(options: Options, output: WasmBindgenOutput) -> Result<()> {
        let WasmBindgenOutput {
            js,
            wasm,
            snippets,
            local_modules,
        } = output;

        let middleware_stack = ServiceBuilder::new()
            .layer(CompressionLayer::new())
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("cross-origin-opener-policy"),
                HeaderValue::from_static("same-origin"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("cross-origin-embedder-policy"),
                HeaderValue::from_static("require-corp"),
            ))
            .into_inner();

        let version = generate_version();

        let html_source = options
            .custom_index_html
            .map(|index_html_path| {
                let path = match index_html_path.is_absolute() {
                    true => index_html_path,
                    false => options.directory.join(index_html_path),
                };
                std::fs::read_to_string(path).map(Cow::Owned)
            })
            .unwrap_or_else(|| Ok(Cow::Borrowed(include_str!("../www/index.html"))))?;
        let mut html = html_source.replace("{{ TITLE }}", &options.title);

        if options.no_module {
            html = html
                .replace("{{ NO_MODULE }}", "<script src=\"./api/wasm.js\"></script>")
                .replace("// {{ MODULE }}", "");
        } else {
            html = html
                .replace(
                    "// {{ MODULE }}",
                    "import wasm_bindgen from './api/wasm.js';",
                )
                .replace("{{ NO_MODULE }}", "");
        };

        let serve_dir = HandleError::new(
            get_service(ServeDir::new(options.directory)),
            internal_server_error,
        );

        let app = Router::new()
            .route("/", get(move || async { Html(html) }))
            .route(
                "/api/wasm.js",
                get(|| async { WithContentType("application/javascript", js) }),
            )
            .route(
                "/api/wasm.wasm",
                get(|| async { WithContentType("application/wasm", wasm) }),
            )
            .route("/api/version", get(move || async { version }))
            .route(
                "/ws",
                get(|ws: WebSocketUpgrade| async { ws.on_upgrade(handle_ws) }),
            )
            .route(
                "/api/snippets/*rest",
                get(|Path(path): Path<String>| async move {
                    match get_snippet_source(&path, &local_modules, &snippets) {
                        Ok(source) => Ok(WithContentType("application/javascript", source)),
                        Err(e) => {
                            tracing::error!("failed to serve snippet `{path}`: {e}");
                            Err(e)
                        }
                    }
                }),
            )
            .fallback_service(serve_dir)
            .layer(middleware_stack);

        let mut address_string = options.address;
        if !address_string.contains(':') {
            address_string += &(":".to_owned()
                + &pick_port::pick_free_port(1334, 10)
                    .unwrap_or(1334)
                    .to_string());
        }
        let addr: SocketAddr = address_string.parse().expect("Couldn't parse address");

        if options.https {
            let certificate = certificate::certificate()?;
            let config =
                RustlsConfig::from_der(vec![certificate.certificate], certificate.private_key)
                    .await?;

            tracing::info!(target: "wasm_server_runner", "starting webserver at https://{}", addr);
            axum_server_dual_protocol::bind_dual_protocol(addr, config)
                .set_upgrade(true)
                .serve(app.into_make_service())
                .await?;
        } else {
            tracing::info!(target: "wasm_server_runner", "starting webserver at http://{}", addr);
            axum_server::bind(addr)
                .serve(app.into_make_service())
                .await?;
        }

        Ok(())
    }

    fn get_snippet_source(
        path: &str,
        local_modules: &HashMap<String, String>,
        snippets: &HashMap<String, Vec<String>>,
    ) -> Result<String, &'static str> {
        if let Some(module) = local_modules.get(path) {
            return Ok(module.clone());
        };

        let (snippet, inline_snippet_name) = path.split_once('/').ok_or("invalid snippet path")?;
        let index = inline_snippet_name
            .strip_prefix("inline")
            .and_then(|path| path.strip_suffix(".js"))
            .ok_or("invalid snippet name in path")?;
        let index: usize = index.parse().map_err(|_| "invalid index")?;
        let snippet = snippets
            .get(snippet)
            .ok_or("invalid snippet name")?
            .get(index)
            .ok_or("snippet index out of bounds")?;
        Ok(snippet.clone())
    }

    async fn handle_ws(mut socket: WebSocket) {
        while let Some(msg) = socket.recv().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => return tracing::warn!("got error {e}, closing websocket connection"),
            };

            let msg = match msg {
                ws::Message::Text(msg) => msg,
                ws::Message::Close(_) => return,
                _ => unreachable!("got non-text message from websocket"),
            };

            let (mut level, mut text) = msg.split_once(',').unwrap();

            if let Some(rest) = text.strip_prefix("TRACE ") {
                level = "debug";
                text = rest;
            } else if let Some(rest) = text.strip_prefix("DEBUG ") {
                level = "debug";
                text = rest;
            } else if let Some(rest) = text.strip_prefix("INFO ") {
                level = "info";
                text = rest;
            } else if let Some(rest) = text.strip_prefix("WARN ") {
                level = "warn";
                text = rest;
            } else if let Some(rest) = text.strip_prefix("ERROR ") {
                level = "error";
                text = rest;
            }

            match level {
                "log" => {} //tracing::info!(target: "app", "{text}"),

                "trace" => tracing::trace!(target: "app", "{text}"),
                "debug" => {} //tracing::debug!(target: "app", "{text}"),
                "info" => tracing::info!(target: "app", "{text}"),
                "warn" => tracing::warn!(target: "app", "{text}"),
                "error" => tracing::error!(target: "app", "{text}"),
                _ => unimplemented!("unexpected log level {level}: {text}"),
            }
        }
    }

    struct WithContentType<T>(&'static str, T);
    impl<T: IntoResponse> IntoResponse for WithContentType<T> {
        fn into_response(self) -> Response {
            let mut response = self.1.into_response();
            response
                .headers_mut()
                .insert("Content-Type", HeaderValue::from_static(self.0));
            response
        }
    }

    async fn internal_server_error(error: impl std::fmt::Display) -> impl IntoResponse {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", error),
        )
    }

    mod pick_port {
        use std::net::{
            Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, TcpListener, ToSocketAddrs,
        };

        fn test_bind_tcp<A: ToSocketAddrs>(addr: A) -> Option<u16> {
            Some(TcpListener::bind(addr).ok()?.local_addr().ok()?.port())
        }
        fn is_free_tcp(port: u16) -> bool {
            let ipv4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
            let ipv6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);

            test_bind_tcp(ipv6).is_some() && test_bind_tcp(ipv4).is_some()
        }

        fn ask_free_tcp_port() -> Option<u16> {
            let ipv4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
            let ipv6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0);
            test_bind_tcp(ipv6).or_else(|| test_bind_tcp(ipv4))
        }

        pub fn pick_free_port(starting_at: u16, try_consecutive: u16) -> Option<u16> {
            (starting_at..=starting_at + try_consecutive)
                .find(|&port| is_free_tcp(port))
                .or_else(ask_free_tcp_port)
        }
    }
}

use std::path::PathBuf;

use anyhow::{anyhow, ensure};
use tracing::info;
use tracing_subscriber::EnvFilter;

use server::Options;

pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

fn bool_option(name: &str, default: bool) -> Result<bool, anyhow::Error> {
    match std::env::var(name) {
        Ok(value) if ["true", "1", "yes"].contains(&value.as_str()) => Ok(true),
        Ok(value) if ["false", "0", "no"].contains(&value.as_str()) => Ok(false),
        Ok(value) => Err(anyhow!(
            "unexpected option {name}={value}, expected true,1 or false,0"
        )),
        Err(_) => Ok(default),
    }
}
fn option(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or(default.to_owned())
}

pub fn main(wasm_file: String) -> Result<(), anyhow::Error> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,app=debug,tower_http=debug,walrus=error"));
    tracing_subscriber::fmt::fmt()
        .without_time()
        .with_env_filter(filter)
        .init();

    let title = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "".to_string());
    let address = option("WASM_SERVER_RUNNER_ADDRESS", "127.0.0.1");
    let directory = option("WASM_SERVER_RUNNER_DIRECTORY", ".");
    let custom_index_html = std::env::var("WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML")
        .ok()
        .map(PathBuf::from);
    let https = bool_option("WASM_SERVER_RUNNER_HTTPS", false)?;
    let no_module = bool_option("WASM_SERVER_RUNNER_NO_MODULE", false)?;

    let options = Options {
        title,
        address,
        directory: PathBuf::from(directory),
        custom_index_html,
        https,
        no_module,
    };

    let wasm_file = PathBuf::from(wasm_file);

    let is_wasm_file = wasm_file.extension().map_or(false, |e| e == "wasm");
    ensure!(is_wasm_file, "expected to be run with a wasm target");

    let output = wasm_bindgen::generate(&options, &wasm_file)?;

    info!(
        "uncompressed wasm output is {} in size",
        pretty_size(output.wasm.len())
    );

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(server::run_server(options, output))?;

    Ok(())
}

fn pretty_size(size_in_bytes: usize) -> String {
    let size_in_kb = size_in_bytes as f32 / 1024.0;
    if size_in_kb < 1024.0 {
        return format!("{:.2}kb", size_in_kb);
    }

    let size_in_mb = size_in_kb / 1024.0;
    format!("{:.2}mb", size_in_mb)
}

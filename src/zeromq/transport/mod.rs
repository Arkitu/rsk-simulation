mod tcp;

use super::codec::FramedIo;
use super::endpoint::Endpoint;
use super::task_handle::TaskHandle;
use super::ZmqResult;

/// Connectes to the given endpoint
///
/// # Panics
/// Panics if the requested endpoint uses a transport type that isn't enabled
pub(crate) async fn connect(endpoint: &Endpoint) -> ZmqResult<(FramedIo, Endpoint)> {
    match endpoint {
        Endpoint::Tcp(_host, _port) => {
            tcp::connect(_host, *_port).await
        }
        Endpoint::Ipc(_path) => {
            panic!("IPC transport is not enabled")
        }
    }
}

pub struct AcceptStopHandle(pub(crate) TaskHandle<()>);

/// Spawns an async task that listens for connections at the provided endpoint.
///
/// `cback` will be invoked when a connection is accepted. If the result was
/// `Ok`, it will receive a tuple containing the framed raw socket, along with
/// the endpoint of the remote connection accepted.
///
/// Returns a ZmqResult, which when Ok is a tuple of the resolved bound
/// endpoint, as well as a channel to stop the async accept task
///
/// # Panics
/// Panics if the requested endpoint uses a transport type that isn't enabled
pub(crate) async fn begin_accept<T>(
    endpoint: Endpoint,
    cback: impl Fn(ZmqResult<(FramedIo, Endpoint)>) -> T + Send + 'static,
) -> ZmqResult<(Endpoint, AcceptStopHandle)>
where
    T: std::future::Future<Output = ()> + Send + 'static,
{
    let _cback = cback;
    match endpoint {
        Endpoint::Tcp(_host, _port) => tcp::begin_accept(_host, _port, _cback).await,
        Endpoint::Ipc(_path) => {
            panic!("IPC transport is not enabled")
        }
    }
}

#[allow(unused)]
fn make_framed<T>(stream: T) -> FramedIo
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Sync + 'static,
{
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
    let (read, write) = tokio::io::split(stream);
    FramedIo::new(Box::new(read.compat()), Box::new(write.compat_write()))
}
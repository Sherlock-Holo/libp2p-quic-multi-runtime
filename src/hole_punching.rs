use crate::{Error, provider::Provider};
use futures_util::future::Either;
use rand::RngExt;
use rand::distr::StandardUniform;
use std::pin::pin;
use std::{
    convert::Infallible,
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

pub(crate) async fn hole_puncher<P: Provider>(
    socket: UdpSocket,
    remote_addr: SocketAddr,
    timeout_duration: Duration,
) -> Error {
    let punch_holes_future = pin!(punch_holes::<P>(socket, remote_addr));
    match futures_util::future::select(P::sleep(timeout_duration), punch_holes_future).await {
        Either::Left(_) => Error::HandshakeTimedOut,
        Either::Right((Err(hole_punch_err), _)) => hole_punch_err,
        Either::Right((Ok(never), _)) => match never {},
    }
}

async fn punch_holes<P: Provider>(
    socket: UdpSocket,
    remote_addr: SocketAddr,
) -> Result<Infallible, Error> {
    loop {
        let contents: Vec<u8> = rand::rng().sample_iter(StandardUniform).take(64).collect();

        tracing::trace!("Sending random UDP packet to {remote_addr}");

        P::send_to(&socket, &contents, remote_addr).await?;

        let sleep_duration = Duration::from_millis(rand::rng().random_range(10..=200));
        P::sleep(sleep_duration).await;
    }
}

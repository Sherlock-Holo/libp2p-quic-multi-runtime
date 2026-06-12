#![cfg(any(feature = "tokio", feature = "smol"))]

use std::{future::Future, time::Duration};

use futures::{StreamExt, channel::oneshot};
use libp2p_core::{
    Endpoint, Transport,
    transport::{DialOpts, ListenerId, PortUse},
};
use libp2p_quic_multi_runtime as quic;

#[cfg(feature = "tokio")]
#[tokio::test]
async fn close_implies_flush() {
    let (alice, bob) = connected_peers::<quic::tokio::Provider>().await;

    libp2p_muxer_test_harness::close_implies_flush(alice, bob).await;
}

#[cfg(feature = "smol")]
#[test]
fn close_implies_flush_smol() {
    smol::block_on(async {
        let (alice, bob) = connected_peers::<quic::smol::Provider>().await;

        libp2p_muxer_test_harness::close_implies_flush(alice, bob).await;
    });
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn read_after_close() {
    let (alice, bob) = connected_peers::<quic::tokio::Provider>().await;

    libp2p_muxer_test_harness::read_after_close(alice, bob).await;
}

#[cfg(feature = "smol")]
#[test]
fn read_after_close_smol() {
    smol::block_on(async {
        let (alice, bob) = connected_peers::<quic::smol::Provider>().await;

        libp2p_muxer_test_harness::read_after_close(alice, bob).await;
    });
}

async fn connected_peers<P>() -> (quic::Connection, quic::Connection)
where
    P: quic::Provider + Spawn,
{
    let mut dialer = new_transport::<P>().boxed();
    let mut listener = new_transport::<P>().boxed();

    listener
        .listen_on(
            ListenerId::next(),
            "/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap(),
        )
        .unwrap();
    let listen_address = listener.next().await.unwrap().into_new_address().unwrap();

    let (dialer_conn_sender, dialer_conn_receiver) = oneshot::channel();
    let (listener_conn_sender, listener_conn_receiver) = oneshot::channel();

    P::spawn(async move {
        let (upgrade, _) = listener.next().await.unwrap().into_incoming().unwrap();

        P::spawn(async move {
            let (_, connection) = upgrade.await.unwrap();

            let _ = listener_conn_sender.send(connection);
        });

        loop {
            listener.next().await;
        }
    });
    let dial_fut = dialer
        .dial(
            listen_address,
            DialOpts {
                role: Endpoint::Dialer,
                port_use: PortUse::Reuse,
            },
        )
        .unwrap();
    P::spawn(async move {
        let connection = dial_fut.await.unwrap().1;

        let _ = dialer_conn_sender.send(connection);
    });

    P::spawn(async move {
        loop {
            dialer.next().await;
        }
    });

    futures::future::try_join(dialer_conn_receiver, listener_conn_receiver)
        .await
        .unwrap()
}

fn new_transport<P: quic::Provider>() -> quic::GenTransport<P> {
    let keypair = libp2p_identity::Keypair::generate_ed25519();
    let mut config = quic::Config::new(&keypair);
    config.handshake_timeout = Duration::from_secs(1);

    quic::GenTransport::<P>::new(config)
}

trait Spawn {
    fn spawn(future: impl Future<Output = ()> + Send + 'static);
}

#[cfg(feature = "tokio")]
impl Spawn for quic::tokio::Provider {
    fn spawn(future: impl Future<Output = ()> + Send + 'static) {
        tokio::spawn(future);
    }
}

#[cfg(feature = "smol")]
impl Spawn for quic::smol::Provider {
    fn spawn(future: impl Future<Output = ()> + Send + 'static) {
        smol::spawn(future).detach();
    }
}

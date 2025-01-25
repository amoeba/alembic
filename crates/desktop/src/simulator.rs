use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use libalembic::rpc::WorldClient;
use rand::Rng;
use tarpc::{client, context, tokio_serde::formats::Json};

#[allow(unused)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut rng = rand::thread_rng();

    let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
    let transport = tarpc::serde_transport::tcp::connect(&addr, Json::default);
    let wc: WorldClient =
        WorldClient::new(client::Config::default(), transport.await.expect("oops")).spawn();

    println!("Waiting for Ctrl-C...");
    let delay_ms = 1000;
    while running.load(Ordering::SeqCst) {
        // Send log
        wc.append_log(context::current(), "hello from simulator".to_string())
            .await
            .expect("Failed to send log");

        // Send packet 1024 bytes long
        let mut random_vec: Vec<u8> = vec![0; 1024]; // Create a vector of 100 zeroes
        rng.fill(&mut random_vec[..]);
        wc.handle_sendto(context::current(), random_vec)
            .await
            .unwrap();

        wc.handle_chat(context::current(), "hello from simulator".to_string())
            .await
            .unwrap();

        thread::sleep(Duration::from_millis(delay_ms));
    }
    println!("Got it! Exiting...");

    Ok(())
}

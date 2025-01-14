use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use libalembic::rpc::WorldClient;
use tarpc::{client, context, tokio_serde::formats::Json};

#[tokio::main]
async fn main() {
    let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
    let transport = tarpc::serde_transport::tcp::connect(&addr, Json::default);
    let client: WorldClient =
        WorldClient::new(client::Config::default(), transport.await.expect("oops")).spawn();

    client
        .append_log(context::current(), "hello from simulator".to_string())
        .await
        .unwrap();
}

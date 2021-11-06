use ipfs_api_backend_hyper as ipfs_api;
use ipfs_api::{IpfsApi, IpfsClient};
use hyper::client::HttpConnector;

#[tokio::main]
async fn main() {

    eprintln!("connecting to localhost:5001...");

    let client = IpfsClient::<HttpConnector>::default();

    match client.version().await {
        Ok(version) => eprintln!("version: {:?}", version.version),
        Err(e) => eprintln!("error getting version: {}", e),
    }
}

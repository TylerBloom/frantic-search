use std::env::args;

use frantic_client::FranticClient;
use frantic_core::cr::Cr;

#[tokio::main]
async fn main() {
    let mut args = args();
    args.next();
    let words: Vec<_> = args.collect();

    let client = FranticClient::connect();
    let cr = client.fetch_latest().await.unwrap();
    let cr = Cr::parse(&cr.text);

    println!("{}", cr.search(&words));
}

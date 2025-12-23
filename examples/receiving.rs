use anyhow::{Result, anyhow};
use tunnel::Tunnel;

#[tokio::main]
async fn main() -> Result<()> {
    let tunnel = Tunnel::new(|sender, data: Vec<u8>| {
        println!("Received data!");
        println!("> Sender: {}", sender);
        println!("> Data: {}", String::from_utf8_lossy(&data));
    })
    .await?;

    println!("Started tunnel with address {}", tunnel.receiver_address());
    println!("Press Ctrl+C to exit");

    match tokio::signal::ctrl_c().await {
        Ok(()) => Ok(()),
        Err(e) => Err(anyhow!(e)),
    }
}

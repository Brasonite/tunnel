use std::str::FromStr;

use anyhow::Result;
use iroh::PublicKey;
use tunnel::Tunnel;

#[tokio::main]
async fn main() -> Result<()> {
    let tunnel = Tunnel::new(|_, _| {}).await?;

    println!("Started tunnel with address {}", tunnel.receiver_address());
    println!("Enter the target address below:");

    let mut address_str = String::new();
    std::io::stdin().read_line(&mut address_str)?;

    let address = PublicKey::from_str(address_str.trim())?;

    for i in 1..=10 {
        tunnel
            .send(address, format!("This is iteration {}.", i))
            .await?;
    }

    tunnel.destroy().await;

    Ok(())
}

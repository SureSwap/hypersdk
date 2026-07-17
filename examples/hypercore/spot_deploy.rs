//! Shows how to deploy a spot token on Hyperliquid (testnet).
//!
//! The full spot deployment lifecycle consists of five sequential steps:
//!
//! 1. `spot_deploy_register_token` — Reserve the ticker and pay the auction gas
//! 2. `spot_deploy_user_genesis`   — Assign the initial supply to user addresses
//! 3. `spot_deploy_genesis`        — Finalise the supply cap and HLP configuration
//! 4. `spot_deploy_register_spot`  — Create the BASE/USDC spot market
//! 5. `spot_deploy_register_hyperliquidity` — Seed the opening order book
//!
//! Run against testnet using a funded test keystore:
//!
//! ```sh
//! cargo run --example spot_deploy -- --keystore ~/.hypersdk/keys/test.json --password pw
//! ```

use clap::Parser;
use hypersdk::hypercore::{self as hypercore, NonceHandler};
use rust_decimal::dec;

use crate::credentials::Credentials;

mod credentials;

#[derive(Parser, Debug, derive_more::Deref)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[deref]
    #[command(flatten)]
    common: Credentials,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = simple_logger::init_with_level(log::Level::Info);

    let args = Cli::parse();
    let signer = args.get()?;

    // Use testnet for deployment examples
    let client = hypercore::testnet();
    let nonce = NonceHandler::default();

    println!("Deployer address: {:#x}", signer.address());

    // Step 1: Register the token
    println!("Step 1: Registering token EXAMPLE...");
    client
        .spot_deploy_register_token(
            &signer,
            "EXAMPLE".to_string(),
            6,                     // sz_decimals
            8,                     // wei_decimals
            None,                  // max_gas
            "Example Token".to_string(),
            nonce.next(),
            None,
            None,
        )
        .await?;

    // In production, query the token index via client.spot_deploy_state(signer.address()).await
    let token_index: u32 = 42;
    println!("Token registered (index: {token_index})");

    // Step 2: Set user genesis allocation
    println!("Step 2: Setting user genesis...");
    client
        .spot_deploy_user_genesis(
            &signer,
            token_index,
            vec![(signer.address(), "1000000000".to_string())],
            vec![],
            nonce.next(),
            None,
            None,
        )
        .await?;

    // Step 3: Finalise genesis
    println!("Step 3: Finalising genesis...");
    client
        .spot_deploy_genesis(
            &signer,
            token_index,
            "1000000".to_string(),
            false,
            nonce.next(),
            None,
            None,
        )
        .await?;

    // Step 4: Register spot pair (base=EXAMPLE, quote=USDC index 0)
    println!("Step 4: Registering spot pair EXAMPLE/USDC...");
    client
        .spot_deploy_register_spot(&signer, token_index, 0, nonce.next(), None, None)
        .await?;

    let spot_index: u32 = 0; // Replace with actual spot pair index

    // Step 5: Seed hyperliquidity
    println!("Step 5: Seeding hyperliquidity...");
    client
        .spot_deploy_register_hyperliquidity(
            &signer,
            spot_index,
            dec!(1.0),
            dec!(100.0),
            5,
            Some(2),
            nonce.next(),
            None,
            None,
        )
        .await?;

    println!("Spot deployment complete!");
    Ok(())
}

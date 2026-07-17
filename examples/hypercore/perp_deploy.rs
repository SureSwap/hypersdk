//! Shows how to deploy a perpetual market on a HIP-3 DEX and update its oracle prices.
//!
//! The perp deployment workflow:
//!
//! 1. `perp_deploy_register_asset` — Register a new perpetual coin on your builder DEX
//! 2. `perp_deploy_set_oracle`     — Push oracle / mark / external reference prices
//!
//! Run against testnet:
//!
//! ```sh
//! cargo run --example perp_deploy -- --keystore ~/.hypersdk/keys/test.json --password pw
//! ```

use clap::Parser;
use hypersdk::hypercore::{self as hypercore, NonceHandler};
use hypersdk::hypercore::types::{PerpAssetRequest, PerpDexSchema};

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

    let client = hypercore::testnet();
    let nonce = NonceHandler::default();

    let dex = "my-builder-dex".to_string(); // your HIP-3 DEX name

    // Step 1: Register a perpetual asset
    println!("Registering perp asset MYTOKEN on {dex}...");
    client
        .perp_deploy_register_asset(
            &signer,
            dex.clone(),
            None, // max_gas
            PerpAssetRequest {
                coin: "MYTOKEN".to_string(),
                sz_decimals: 4,
                oracle_px: "1.0".to_string(),
                margin_table_id: 0,
                only_isolated: false,
            },
            Some(PerpDexSchema {
                full_name: "My Token Perpetual".to_string(),
                collateral_token: 0, // USDC
                oracle_updater: None, // defaults to deployer address
            }),
            nonce.next(),
            None,
            None,
        )
        .await?;

    println!("Perp asset registered.");

    // Step 2: Push initial oracle prices
    // oracle_pxs / mark_pxs / external_perp_pxs are sorted by coin name automatically.
    println!("Setting oracle prices...");
    client
        .perp_deploy_set_oracle(
            &signer,
            dex,
            vec![("MYTOKEN".to_string(), "1.05".to_string())],
            vec![vec![("MYTOKEN".to_string(), "1.05".to_string())]],
            vec![("MYTOKEN".to_string(), "1.05".to_string())],
            nonce.next(),
            None,
            None,
        )
        .await?;

    println!("Oracle prices set. Perp deployment complete!");
    Ok(())
}

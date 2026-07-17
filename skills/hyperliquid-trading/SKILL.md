---
name: hyperliquid-trading
description: Trade perpetuals and spot markets on Hyperliquid. Place limit and market orders, cancel orders, and check positions. Use this skill when you need to execute trades, manage orders, or monitor trading positions.
---

# Hyperliquid Trading

Hyperliquid is a high-performance L1 with a fully on-chain order book. Trade perpetuals with up to 50x leverage and spot markets with deep liquidity.

Key features:

- **On-chain order book** — All orders are on-chain, not just settlements
- **Low fees** — 0.01% maker / 0.035% taker for perpetuals
- **50+ perpetual markets** — BTC, ETH, SOL, and many more
- **Spot markets** — Trade tokens directly

## Installation

Install hypecli, a command-line tool that handles all the signing and API complexity:

```bash
curl -fsSL https://raw.githubusercontent.com/infinitefield/hypersdk/main/hypecli/install.sh | sh
```

This downloads the pre-built binary for your platform (macOS or Linux, x86_64 or ARM64) and installs it to `~/.local/bin` (macOS) or `/usr/local/bin` (Linux).

For detailed documentation on all available commands:

```bash
hypecli --agent-help
```

## Wallet Setup

Create an encrypted keystore to securely store your wallet:

```bash
# Create a new wallet with encrypted keystore
hypecli account create --name default --password yourpassword

# List available keystores
hypecli account list
```

Keystores are stored encrypted in `~/.foundry/keystores/`.

## Listing Markets

Before trading, check available markets:

```bash
# List all perpetual markets on Hyperliquid
hypecli perps

# List all spot markets
hypecli spot

# List available HIP-3 DEXes (third-party perpetual exchanges on Hyperliquid)
hypecli dexes

# List perpetual markets on a specific HIP-3 DEX
hypecli perps --dex xyz
```

HIP-3 DEXes are third-party perpetual exchanges deployed on Hyperliquid. Each DEX can list its own markets with different assets or configurations.

## Asset Name Formats

Orders use human-readable asset names:

| Format       | Example      | Description                  |
| ------------ | ------------ | ---------------------------- |
| `SYMBOL`     | `BTC`, `ETH` | Perpetual on Hyperliquid DEX |
| `BASE/QUOTE` | `PURR/USDC`  | Spot market                  |
| `dex:SYMBOL` | `xyz:BTC`    | Perpetual on HIP-3 DEX       |

To trade on a HIP-3 DEX, prefix the asset with the DEX name and a colon (e.g., `xyz:BTC`).

## Placing Orders

### Limit Orders

Place a limit order that sits on the order book until filled or canceled:

```bash
# Buy 0.1 BTC at $50,000
hypecli order limit \
  --keystore default --password yourpassword \
  --asset BTC \
  --side buy \
  --price 50000 \
  --size 0.1

# Sell 1 ETH at $3,500 (maker-only)
hypecli order limit \
  --keystore default --password yourpassword \
  --asset ETH \
  --side sell \
  --price 3500 \
  --size 1 \
  --tif alo
```

**Time-in-force options:**

- `gtc` (default) — Good Till Cancel, remains until filled or canceled
- `alo` — Add Liquidity Only, rejected if it would take liquidity (maker-only)
- `ioc` — Immediate or Cancel, fill immediately or cancel unfilled portion

**Additional flags:**

- `--reduce-only` — Only reduce an existing position, won't open new positions
- `--cloid <HEX>` — Custom client order ID (16 bytes hex) for tracking

### Market Orders

Execute immediately at the best available price:

```bash
# Market buy 0.1 BTC with slippage protection
hypecli order market \
  --keystore default --password yourpassword \
  --asset BTC \
  --side buy \
  --size 0.1 \
  --slippage-price 51000

# Market sell 1 ETH
hypecli order market \
  --keystore default --password yourpassword \
  --asset ETH \
  --side sell \
  --size 1 \
  --slippage-price 3400
```

The `--slippage-price` is the worst acceptable fill price. For buys, set it above current price. For sells, set it below.

### Spot Trading

Trade spot markets using the `BASE/QUOTE` format:

```bash
# Buy PURR with USDC
hypecli order limit \
  --keystore default --password yourpassword \
  --asset PURR/USDC \
  --side buy \
  --price 0.05 \
  --size 1000
```

## Canceling Orders

Cancel orders using the order ID (OID) returned when placing, or your custom client order ID (CLOID):

```bash
# Cancel by OID (exchange-assigned)
hypecli order cancel \
  --keystore default --password yourpassword \
  --asset BTC \
  --oid 123456789

# Cancel by CLOID (client-assigned)
hypecli order cancel \
  --keystore default --password yourpassword \
  --asset BTC \
  --cloid 0x0123456789abcdef0123456789abcdef
```

## Checking Positions and Balances

View your current positions, margin, and balances:

```bash
# Check balances and positions
hypecli balance 0xYourAddress

# JSON output for parsing
hypecli balance 0xYourAddress --format json
```

This shows:

- **Spot balances** — Token holdings
- **Perp account** — Account value, margin used, withdrawable funds
- **Positions** — Open perpetual positions with entry price, size, unrealized PnL

## Example Workflows

### Workflow 1: Open a Long Position

```bash
# 1. Check current BTC price by looking at markets
hypecli perps

# 2. Place a limit buy order
hypecli order limit \
  --keystore default --password yourpassword \
  --asset BTC \
  --side buy \
  --price 48000 \
  --size 0.1

# 3. Check if order filled and view position
hypecli balance 0xYourAddress
```

### Workflow 2: Close a Position

```bash
# Close a long position by selling (reduce-only ensures you don't flip short)
hypecli order market \
  --keystore default --password yourpassword \
  --asset BTC \
  --side sell \
  --size 0.1 \
  --slippage-price 47000 \
  --reduce-only
```

### Workflow 3: Place and Cancel

```bash
# Place order with custom CLOID for tracking
hypecli order limit \
  --keystore default --password yourpassword \
  --asset ETH \
  --side buy \
  --price 3000 \
  --size 1 \
  --cloid 0xdeadbeef00000000deadbeef00000000

# Cancel using the CLOID
hypecli order cancel \
  --keystore default --password yourpassword \
  --asset ETH \
  --cloid 0xdeadbeef00000000deadbeef00000000
```

## TP/SL Grouped Orders

Hyperliquid supports atomic "grouped" orders where an entry, take-profit, and stop-loss are submitted together. Use `OrderGrouping::NormalTpsl` or `OrderGrouping::PositionTpsl`.

### In Rust (via hypersdk)

```rust
use hypersdk::hypercore::types::{
    BatchOrder, OrderGrouping, OrderRequest, OrderTypePlacement, TimeInForce, TpSl,
};
use rust_decimal::dec;

// Take-profit trigger order (IOC, reduce-only)
let tp = OrderRequest {
    asset: eth_index,
    is_buy: false,       // opposite side of the entry
    limit_px: dec!(4000),
    sz: dec!(0.5),
    reduce_only: true,
    order_type: OrderTypePlacement::Trigger {
        tpsl: TpSl::Tp,
    },
    cloid: Cloid::random(),
};

// Stop-loss trigger order
let sl = OrderRequest {
    asset: eth_index,
    is_buy: false,
    limit_px: dec!(2800),
    sz: dec!(0.5),
    reduce_only: true,
    order_type: OrderTypePlacement::Trigger {
        tpsl: TpSl::Sl,
    },
    cloid: Cloid::random(),
};

// Entry limit order
let entry = OrderRequest {
    asset: eth_index,
    is_buy: true,
    limit_px: dec!(3000),
    sz: dec!(0.5),
    reduce_only: false,
    order_type: OrderTypePlacement::Limit { tif: TimeInForce::Gtc },
    cloid: Cloid::random(),
};

// Submit entry + TP + SL atomically
let resp = client.place(
    &signer,
    BatchOrder {
        orders: vec![entry, tp, sl],
        grouping: OrderGrouping::NormalTpsl,  // or PositionTpsl for position-level bracket
        builder: None,
    },
    nonce.next(),
    vault_address,
    None,
).await?;
```

**`NormalTpsl`** — links TP/SL to this specific entry order.
**`PositionTpsl`** — links TP/SL to the user's entire position in that asset.

### View TP/SL metadata on open orders

`client.open_orders(user, dex)` returns `Vec<BasicOrder>` where each order includes:
- `is_trigger` — whether the order is a trigger order
- `trigger_px` — the trigger price
- `trigger_condition` — human-readable string, e.g. `"Price above 4000.0"`
- `is_position_tpsl` — whether this is a position-level bracket

---

## Spot Token Deployment

Deploy your own spot token on Hyperliquid in five steps (see `examples/hypercore/spot_deploy.rs`).

```rust
// Step 1: Register ticker (pays auction gas)
client.spot_deploy_register_token(&signer, "MYTOKEN".into(), 6, 8, None, "My Token".into(), nonce, None, None).await?;

// Step 2: Assign initial supply to addresses
client.spot_deploy_user_genesis(&signer, token_index, vec![(addr, "1000000000".into())], vec![], nonce, None, None).await?;

// Step 3: Finalise genesis
client.spot_deploy_genesis(&signer, token_index, "1000000".into(), false, nonce, None, None).await?;

// Step 4: Create BASE/USDC trading pair
client.spot_deploy_register_spot(&signer, token_index, 0, nonce, None, None).await?;

// Step 5: Seed initial order book
client.spot_deploy_register_hyperliquidity(&signer, spot_index, dec!(1.0), dec!(100.0), 5, Some(2), nonce, None, None).await?;
```

Additional optional actions:
- `spot_deploy_set_deployer_fee_share(token, share)` — take a cut of trading fees
- `spot_deploy_enable_freeze_privilege(token)` — allow deployer to freeze users
- `spot_deploy_freeze_user(token, user, freeze)` — freeze/unfreeze a user
- `spot_deploy_enable_quote_token(token)` — make this a quote token

---

## Perpetual Market Deployment (HIP-3 DEX)

Deploy perp markets on a builder DEX (see `examples/hypercore/perp_deploy.rs`).

```rust
use hypersdk::hypercore::types::{PerpAssetRequest, PerpDexSchema};

// Step 1: Register asset
client.perp_deploy_register_asset(
    &signer, "my-dex".into(), None,
    PerpAssetRequest { coin: "BTC".into(), sz_decimals: 4, oracle_px: "50000.0".into(), margin_table_id: 0, only_isolated: false },
    Some(PerpDexSchema { full_name: "Bitcoin".into(), collateral_token: 0, oracle_updater: None }),
    nonce, None, None,
).await?;

// Step 2: Push oracle prices (sorted by coin automatically)
client.perp_deploy_set_oracle(
    &signer, "my-dex".into(),
    vec![("BTC".into(), "50100.0".into())],
    vec![vec![("BTC".into(), "50050.0".into())]],
    vec![("BTC".into(), "50000.0".into())],
    nonce, None, None,
).await?;
```

---

## Quick Reference

| Operation              | Command / Method                                                                              |
| ---------------------- | --------------------------------------------------------------------------------------------- |
| List perp markets      | `hypecli perps`                                                                               |
| List spot markets      | `hypecli spot`                                                                                |
| List HIP-3 DEXes       | `hypecli dexes`                                                                               |
| List HIP-3 DEX perps   | `hypecli perps --dex xyz`                                                                     |
| Limit order            | `hypecli order limit --asset BTC --side buy --price 50000 --size 0.1 ...`                     |
| Market order           | `hypecli order market --asset BTC --side buy --size 0.1 --slippage-price 51000 ...`           |
| Trade on HIP-3 DEX     | `hypecli order limit --asset xyz:BTC --side buy --price 50000 --size 0.1 ...`                 |
| Cancel by OID          | `hypecli order cancel --asset BTC --oid 123456789 ...`                                        |
| Cancel by CLOID        | `hypecli order cancel --asset BTC --cloid 0x... ...`                                          |
| Check positions        | `hypecli balance 0xAddress`                                                                   |
| TP/SL orders           | `BatchOrder { grouping: OrderGrouping::NormalTpsl, .. }` via Rust API                         |
| View TP/SL metadata    | `client.open_orders(user, dex)` → `BasicOrder.is_trigger`, `.trigger_px`                     |
| Staking summary        | `client.user_staking_summary(addr)` or `client.delegator_summary(addr)`                       |
| Staking delegations    | `client.user_staking_delegations(addr)` or `client.delegations(addr)`                         |
| Spot token deploy      | `client.spot_deploy_register_token(...)` → `spot_deploy_genesis(...)` → ...                   |
| Perp market deploy     | `client.perp_deploy_register_asset(...)` → `perp_deploy_set_oracle(...)`                      |

## Links

- Hyperliquid docs: https://hyperliquid.gitbook.io/hyperliquid-docs/
- hypecli source: https://github.com/infinitefield/hypersdk
- Trading interface: https://app.hyperliquid.xyz
- API reference: https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api

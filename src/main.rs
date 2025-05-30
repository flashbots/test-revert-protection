use std::time::Duration;

use alloy::{
    eips::{BlockNumberOrTag, eip2718::Encodable2718},
    hex,
    network::{EthereumWallet, TransactionBuilder},
    primitives::{B256, Bytes, address},
    providers::{PendingTransactionBuilder, Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use clap::Parser;
use eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Private key for signing transactions
    /// By default it uses a testnet private key used in Anvil/Reth.
    #[arg(
        long,
        default_value = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    )]
    private_key: String,

    /// RPC URL for the Ethereum node. Supports special names like 'local', 'sepolia', 'mainnet'
    #[arg(long, value_parser = parse_rpc_url, default_value = "http://localhost:8545")]
    rpc_url: String,

    /// Whether the transaction should revert
    #[arg(long)]
    reverts: bool,

    /// Whether to send the transaction as a bundle
    #[arg(long)]
    bundle: bool,
}

fn parse_rpc_url(input: &str) -> Result<String, String> {
    match input {
        "uni-experimental" => {
            Ok("http://experi-Proxy-cDzRHZCY6czr-74615020.us-east-2.elb.amazonaws.com".to_string())
        }
        "uni-sepolia" => Ok("https://sepolia.unichain.org".to_string()),
        url => Ok(url.to_string()),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bundle {
    #[serde(rename = "txs")]
    pub transactions: Vec<Bytes>,

    #[serde(rename = "maxBlockNumber")]
    pub block_number_max: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BundleResult {
    #[serde(rename = "bundleHash")]
    pub bundle_hash: B256,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let pk_signer: PrivateKeySigner = args.private_key.parse()?;
    let pk_addr = pk_signer.address();
    println!("Address of the signer: {pk_addr}");

    // Create a provider with the wallet.
    let rpc_url = args.rpc_url.parse()?;
    let provider = ProviderBuilder::new()
        .wallet(pk_signer.clone())
        .connect_http(rpc_url);
    let wallet = EthereumWallet::from(pk_signer);

    let nonce = provider.get_transaction_count(pk_addr).await?;
    println!("Nonce: {nonce}");

    let chain_id = provider.get_chain_id().await?;

    let base_fee = provider
        .get_block_by_number(BlockNumberOrTag::Latest)
        .await?
        .map(|block| block.header.base_fee_per_gas.expect("base fee"))
        .unwrap() as u128;

    // Add these lines:
    let priority_fee = (base_fee / 10).max(2_000_000_000); // 10% of base fee or 2 gwei minimum
    let max_fee = base_fee + priority_fee + (base_fee / 4); // base + tip + 25% buffer

    println!(
        "Sending transaction that reverts: {:?}, with bundle {:?}",
        args.reverts, args.bundle
    );

    let balance = provider.get_balance(pk_addr).await?;
    if balance.is_zero() {
        eyre::bail!("Insufficient balance for the transaction. Please fund the account.");
    }

    let mut tx = TransactionRequest::default()
        .with_gas_limit(300000)
        .with_chain_id(chain_id)
        .with_nonce(nonce)
        .with_max_priority_fee_per_gas(priority_fee)
        .with_max_fee_per_gas(max_fee);

    if args.reverts {
        tx.set_deploy_code(Bytes::from(hex!("60006000fd")));
    } else {
        tx.set_to(address!("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
    };

    let tx_envelope = tx.build(&wallet).await?;
    let tx_encoded = tx_envelope.encoded_2718();

    // Send the transaction and wait for inclusion.
    let pending_tx = if args.bundle {
        let bundle = Bundle {
            transactions: vec![tx_encoded.into()],
            block_number_max: None,
        };

        let result: BundleResult = provider
            .client()
            .request("eth_sendBundle", (bundle,))
            .await?;

        PendingTransactionBuilder::new(provider.root().clone(), result.bundle_hash)
    } else {
        let pending = provider.send_raw_transaction(&tx_encoded).await?;
        pending
    };

    let pending_tx = pending_tx.with_timeout(Some(Duration::from_secs(20)));

    println!("Sent transaction: {}", pending_tx.tx_hash());
    println!("Waiting for transaction to be mined...");

    match pending_tx.watch().await {
        Ok(tx_hash) => {
            println!("Transaction mined: {}", tx_hash);
        }
        Err(e) => {
            println!("Error watching transaction: {}", e);
        }
    }

    Ok(())
}

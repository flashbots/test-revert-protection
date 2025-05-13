use alloy::{
    hex,
    network::TransactionBuilder,
    primitives::{Bytes, address},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use clap::Parser;
use eyre::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Private key for signing transactions
    #[arg(long)]
    private_key: String,

    /// RPC URL for the Ethereum node
    #[arg(long)]
    rpc_url: String,
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
        .wallet(pk_signer)
        .connect_http(rpc_url);

    // Build a transaction to send 100 wei from Alice to Vitalik.
    // The `from` field is automatically filled to the first signer's address (Alice).
    let tx = TransactionRequest::default()
        .with_gas_limit(300000)
        .with_to(address!("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .with_input(Bytes::from(hex!("60006000fd"))); // PUSH1 0x00 PUSH1 0x00 REVERT)

    // Send the transaction and wait for inclusion.
    let tx_hash = provider.send_transaction(tx).await?.watch().await?;

    println!("Sent transaction: {tx_hash}");

    Ok(())
}

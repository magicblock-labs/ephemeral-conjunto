// Run via: cargo run --example guiding_transactions

use conjunto_providers::rpc_provider_config::RpcProviderConfig;
use conjunto_test_tools::accounts::delegated_account_ids;
use conjunto_transwise::transwise::Transwise;
use solana_sdk::{
    hash::Hash,
    pubkey,
    signature::Keypair,
    signer::Signer,
    system_instruction, system_transaction,
    transaction::{SanitizedTransaction, Transaction},
};

#[tokio::main]
async fn main() {
    // NOTE: this example depends on the below used account to be present on devnet

    // This is considered a _new_ account since it doesn't exist on devnet yet.
    // Thus it doesn't factor into the guiding decision as we can create a new
    // account anywhere, on chain or in the ephemeral validator.
    let from_kp = Keypair::new();

    let (delegated_id, _) = delegated_account_ids();
    let data_id = pubkey!("soLXiij6o94fntzfvn2meNybhNfPBviTVuyXLVEtDJ3");

    let transwise = Transwise::new(RpcProviderConfig::devnet());

    // 1. Transferring to a delegated account
    {
        let tx = system_transaction::transfer(
            &from_kp,
            &delegated_id,
            42,
            Hash::default(),
        );
        let sanitized_tx = SanitizedTransaction::from_transaction_for_tests(tx);
        let endpoint = transwise
            .guide_sanitized_transaction(&sanitized_tx)
            .await
            .unwrap();
        println!("{:#?}", endpoint);
        assert!(endpoint.is_ephemeral());
    }

    // 2. Transferring to a data account
    {
        let tx = system_transaction::transfer(
            &from_kp,
            &data_id,
            42,
            Hash::default(),
        );
        let sanitized_tx = SanitizedTransaction::from_transaction_for_tests(tx);
        let endpoint = transwise
            .guide_sanitized_transaction(&sanitized_tx)
            .await
            .unwrap();
        println!("{:#?}", endpoint);
        assert!(endpoint.is_chain());
    }

    // 3. Transferring both to a delegated and a data account
    {
        let ix_delegated =
            system_instruction::transfer(&from_kp.pubkey(), &delegated_id, 42);
        let ix_data =
            system_instruction::transfer(&from_kp.pubkey(), &data_id, 42);
        let tx = Transaction::new_signed_with_payer(
            &[ix_delegated, ix_data],
            Some(&from_kp.pubkey()),
            &[&from_kp],
            Hash::default(),
        );
        let sanitized_tx = SanitizedTransaction::from_transaction_for_tests(tx);
        let endpoint = transwise
            .guide_sanitized_transaction(&sanitized_tx)
            .await
            .unwrap();
        println!("{:#?}", endpoint);
        assert!(endpoint.is_unroutable());
    }
}

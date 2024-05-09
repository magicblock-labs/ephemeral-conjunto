// Run via: cargo run --example guiding_transactions

use std::str::FromStr;

use conjunto_lockbox::accounts::RpcAccountProviderConfig;
use conjunto_test_tools::accounts::delegated_account_ids;
use conjunto_transwise::Transwise;
use solana_sdk::{
    hash::Hash,
    pubkey::Pubkey,
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

    let (delegated_id, _, _) = delegated_account_ids();
    let unlocked_id =
        Pubkey::from_str("soLXiij6o94fntzfvn2meNybhNfPBviTVuyXLVEtDJ3")
            .unwrap();

    let transwise = Transwise::new(RpcAccountProviderConfig::default());

    // 1. Transferring to a delegated account
    {
        let tx = system_transaction::transfer(
            &from_kp,
            &delegated_id,
            42,
            Hash::default(),
        );
        let sanitized_tx = SanitizedTransaction::from_transaction_for_tests(tx);
        let endpoint =
            transwise.guide_transaction(&sanitized_tx).await.unwrap();
        println!("{:#?}", endpoint);
        assert!(endpoint.is_ephemeral());
    }

    // 2. Transferring to an undelegated account
    {
        let tx = system_transaction::transfer(
            &from_kp,
            &unlocked_id,
            42,
            Hash::default(),
        );
        let sanitized_tx = SanitizedTransaction::from_transaction_for_tests(tx);
        let endpoint =
            transwise.guide_transaction(&sanitized_tx).await.unwrap();
        println!("{:#?}", endpoint);
        assert!(endpoint.is_chain());
    }

    // 3. Transferring both to a delegated and an undelegated account
    {
        let ix_delegated =
            system_instruction::transfer(&from_kp.pubkey(), &delegated_id, 42);
        let ix_undelegated =
            system_instruction::transfer(&from_kp.pubkey(), &unlocked_id, 42);
        let tx = Transaction::new_signed_with_payer(
            &[ix_delegated, ix_undelegated],
            Some(&from_kp.pubkey()),
            &[&from_kp],
            Hash::default(),
        );
        let sanitized_tx = SanitizedTransaction::from_transaction_for_tests(tx);
        let endpoint =
            transwise.guide_transaction(&sanitized_tx).await.unwrap();
        println!("{:#?}", endpoint);
        assert!(endpoint.is_unroutable());
    }
}

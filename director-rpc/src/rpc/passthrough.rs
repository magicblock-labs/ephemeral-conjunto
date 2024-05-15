use jsonrpsee::{
    core::{client::ClientT, ClientError, RegisterMethodError},
    types::{ErrorObjectOwned, Params},
    RpcModule,
};
use log::*;
use serde::de::DeserializeOwned;
use solana_account_decoder::{parse_token::UiTokenAmount, UiAccount};
use solana_rpc_client_api::response::{
    OptionalContext, Response as RpcResponse, RpcAccountBalance,
    RpcBlockCommitment, RpcBlockProduction, RpcBlockhash,
    RpcConfirmedTransactionStatusWithSignature, RpcContactInfo, RpcIdentity,
    RpcInflationGovernor, RpcInflationRate, RpcInflationReward,
    RpcKeyedAccount, RpcLeaderSchedule, RpcPerfSample, RpcPrioritizationFee,
    RpcSimulateTransactionResult, RpcSnapshotSlotInfo, RpcStakeActivation,
    RpcSupply, RpcTokenAccountBalance, RpcVersionInfo, RpcVoteAccountStatus,
};
use solana_sdk::{
    clock::{Slot, UnixTimestamp},
    epoch_info::EpochInfo,
    epoch_schedule::EpochSchedule,
};
use solana_transaction_status::{TransactionStatus, UiConfirmedBlock};

use super::DirectorRpc;
use crate::{
    rpc::params::RawParams,
    utils::{server_error, ServerErrorCode},
};

// -----------------
// Solana Types
// -----------------
// Copied here instead of depending on large crates
const MAX_LOCKOUT_HISTORY: usize = 31;
type BlockCommitmentArray = [u64; MAX_LOCKOUT_HISTORY + 1];

// -----------------
// register_passthrough_methods
// -----------------
async fn passthrough_impl<'a, R: DeserializeOwned>(
    method: &'a str,
    params: Params<'static>,
    rpc: &DirectorRpc,
) -> Result<R, ErrorObjectOwned> {
    let params = RawParams(params);
    match rpc
        .rpc_chain_client
        .request::<R, RawParams>(method, params)
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => match err {
            // Pass RPC JSON errors through directly
            ClientError::Call(err) => Err(err),
            _ => Err(server_error(
                format!("Failed to forward to on-chain RPC: {err:?}"),
                ServerErrorCode::RpcClientError,
            )),
        },
    }
}

pub fn register_passthrough_methods(
    module: &mut RpcModule<DirectorRpc>,
) -> Result<(), RegisterMethodError> {
    macro_rules! passthrough {
        ($method:literal, $return_type:ty) => {
            module.register_async_method(
                $method,
                |params, rpc| async move {
                    debug!("{}", $method);
                    trace!("{:#?}", params);
                    passthrough_impl::<$return_type>($method, params, &rpc)
                        .await
                },
            )?;
        };
    }

    // The below macro calls provide the method name as well as the return type as copied
    // from solana/rpc/src/rpc.rs.
    // We use the ClientT::request method to forward the request to the chain RPC.
    // This is the easiest way to do this, but that method ends up parsing the result
    // already which is why we need to provide the return type.
    // See: https://docs.rs/jsonrpsee-core/0.22.5/jsonrpsee_core/client/trait.ClientT.html#tymethod.request
    // In the future we may optimize this by implementing our own way of forwarding the request
    // and somehow passing the result back raw.

    // Some of those need to be guided and we will use one of the following strategies:
    // - TryEphem: try to get a result from ephemeral and if that fails or returns None
    //             try the chain
    // - Ephem: get the result from ephemeral
    // - Chain: get the result from chain (which is basically the passthrough option selected for
    //          most)
    // - Both:  for requests that return an array of results first fill from ephem and try the
    //          remaining ones from chain

    // TODO: guide TryEphem
    passthrough!("getAccountInfo", RpcResponse<Option<UiAccount>>);
    // TODO: guide TryEphem
    passthrough!("getBalance", RpcResponse<u64>);
    passthrough!("getBlock", Option<UiConfirmedBlock>);
    passthrough!(
        "getBlockCommitment",
        RpcBlockCommitment<BlockCommitmentArray>
    );
    passthrough!("getBlockHeight", u64);
    passthrough!("getBlockProduction", RpcResponse<RpcBlockProduction>);
    passthrough!("getBlockTime", Option<UnixTimestamp>);
    passthrough!("getBlocks", Vec<Slot>);
    passthrough!("getBlocksWithLimit", Vec<Slot>);
    passthrough!("getClusterNodes", Vec<RpcContactInfo>);
    passthrough!("getEpochInfo", EpochInfo);
    passthrough!("getEpochSchedule", EpochSchedule);
    passthrough!("getFeeForMessage", RpcResponse<Option<u64>>);
    passthrough!("getFirstAvailableBlock", Slot);
    passthrough!("getGenesisHash", String);
    passthrough!("getHealth", String);
    passthrough!("getHighestSnapshotSlot", RpcSnapshotSlotInfo);
    passthrough!("getIdentity", RpcIdentity);
    passthrough!("getInflationGovernor", RpcInflationGovernor);
    passthrough!("getInflationRate", RpcInflationRate);
    passthrough!("getInflationReward", Vec<Option<RpcInflationReward>>);
    passthrough!("getLargestAccounts", RpcResponse<Vec<RpcAccountBalance>>);
    // TODO: guide Ephem (ephemeral validator should match the on chain blockhash)
    passthrough!("getLatestBlockhash", RpcResponse<RpcBlockhash>);
    passthrough!("getLeaderSchedule", Option<RpcLeaderSchedule>);
    passthrough!("getMaxRetransmitSlot", Slot);
    passthrough!("getMaxShredInsertSlot", Slot);
    passthrough!("getMinimumBalanceForRentExemption", u64);
    // TODO: guide TryEphem (if any account is missing get ALL from chain)
    passthrough!("getMultipleAccounts", RpcResponse<Vec<Option<UiAccount>>>);
    // TODO: guide TryEphem (go to chain if program is not found on ephem)
    passthrough!("getProgramAccounts", OptionalContext<Vec<RpcKeyedAccount>>);
    // TODO: guide Ephem
    passthrough!("getRecentPerformanceSamples", Vec<RpcPerfSample>);
    passthrough!("getRecentPrioritizationFees", Vec<RpcPrioritizationFee>);
    // TODO: guide TryEphem
    passthrough!(
        "getSignatureStatuses",
        RpcResponse<Vec<Option<TransactionStatus>>>
    );
    // TODO: guide Both
    passthrough!(
        "getSignaturesForAddress",
        Vec<RpcConfirmedTransactionStatusWithSignature>
    );
    // TODO: guide Ephem
    passthrough!("getSlot", Slot);
    passthrough!("getSlotLeader", String);
    passthrough!("getSlotLeaders", Vec<String>);
    passthrough!("getStakeActivation", RpcStakeActivation);
    passthrough!("getStakeMinimumDelegation", RpcResponse<u64>);
    passthrough!("getSupply", RpcResponse<RpcSupply>);
    passthrough!("getTokenAccountBalance", RpcResponse<UiTokenAmount>);
    passthrough!(
        "getTokenAccountsByDelegate",
        RpcResponse<Vec<RpcKeyedAccount>>
    );
    passthrough!("getTokenAccountsByOwner", RpcResponse<Vec<RpcKeyedAccount>>);
    passthrough!(
        "getTokenLargestAccounts",
        RpcResponse<Vec<RpcTokenAccountBalance>>
    );
    passthrough!("getTokenSupply", RpcResponse<UiTokenAmount>);
    // TODO: guide TryEphem
    // Also causes problems since some trait of the response is missing, we'll deal with that
    // once we move it to the guide module
    // passthrough!( "getTransaction", Option<EncodedConfirmedTransactionWithStatusMeta>);
    // TODO: guide Ephem
    passthrough!("getTransactionCount", u64);
    // TODO: guide Ephem
    passthrough!("getVersion", RpcVersionInfo);
    passthrough!("getVoteAccounts", RpcVoteAccountStatus);
    // TODO: guide Ephem (blockhash should match chain)
    passthrough!("isBlockhashValid", RpcResponse<bool>);
    passthrough!("minimumLedgerSlot", Slot);
    // This always goes to chain for now since we don't allow creating new accounts in the ephemeral
    // validator, only copying locked ones.
    passthrough!("requestAirdrop", String);
    // TODO: guide Ephem
    passthrough!(
        "simulateTransaction",
        RpcResponse<RpcSimulateTransactionResult>
    );

    Ok(())
}

use conjunto_core::GuideStrategy;
use log::*;
use solana_rpc_client_api::config::RpcTransactionLogsFilter;

use crate::messages::ParsedClientMessage;

pub fn guide_strategy_from_pubsub_msg(msg: &str) -> GuideStrategy {
    let parsed = match ParsedClientMessage::try_from(msg) {
        Ok(method) => method,
        Err(err) => {
            // If we cannot identify the method then we default to just
            // forward to chain
            warn!("Failed to parse message: {} ({:?})", msg, err);
            return GuideStrategy::Chain;
        }
    };
    use ParsedClientMessage::*;
    match parsed {
        // We don't know who the Ping/Pong is responding to so we forward to both
        Ping | Pong => GuideStrategy::Both,

        // Unsubscribe methods have to go to both chain and ephemeral
        // since we don't track subscription ids
        // This results in invalid unsub requests, but for now this is fine
        // The only way to improve this would be to parse all messages coming
        // from the backend in order to pull out subscription ids and store
        // which backend they belong to
        AccountUnsubscribe
        | BlockUnsubscribe
        | LogsUnsubscribe
        | ProgramUnsubscribe
        | RootUnsubscribe
        | SignatureUnsubscribe
        | SlotUnsubscribe
        | SlotsUpdatesUnsubscribe
        | VoteUnsubscribe => GuideStrategy::Both,

        // Subscribe methods that always go to chain since they
        // are either not at all supported by the ephem validator
        // and/or still in beta
        BlockSubscribe
        | RootSubscribe
        | SlotsUpdatesSubscribe
        | VoteSubscribe => GuideStrategy::Chain,

        // We expect the client to want to see faster moving slots of our
        // ephemeral validator
        SlotSubscribe => GuideStrategy::Ephemeral,

        // Subscribe methods that are handled differently depending
        // on params that are part of the message
        AccountSubscribe { address } => {
            GuideStrategy::TryEphemeralForAccount(address.to_string(), true)
        }
        ProgramSubscribe { program_id } => {
            GuideStrategy::TryEphemeralForProgram(program_id.to_string(), true)
        }
        SignatureSubscribe { signature } => {
            GuideStrategy::TryEphemeralForSignature(signature.to_string(), true)
        }
        LogsSubscribe { filter } => {
            use RpcTransactionLogsFilter::*;
            match filter {
                All => GuideStrategy::Ephemeral,
                AllWithVotes => GuideStrategy::Chain,
                Mentions(sigs) => {
                    // NOTE: only one mentioned sig is supported
                    GuideStrategy::TryEphemeralForSignature(
                        sigs[0].to_string(),
                        true,
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn guide_and_assert(msg: Value, expected: &GuideStrategy) {
        let actual = guide_strategy_from_pubsub_msg(msg.to_string().as_str());
        assert_eq!(&actual, expected);
    }

    #[test]
    fn test_guide_account_subscribe() {
        guide_and_assert(
            serde_json::json! {{
                "method": "accountSubscribe",
                "params": ["SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ"]
            }},
            &GuideStrategy::TryEphemeralForAccount(
                "SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ".to_string(),
                true,
            ),
        );
    }
    #[test]
    fn test_guide_program_subscribe() {
        guide_and_assert(
            serde_json::json! {{
                "method": "programSubscribe",
                "params": ["11111111111111111111111111111111", {}]
            }},
            &GuideStrategy::TryEphemeralForProgram(
                "11111111111111111111111111111111".to_string(),
                true,
            ),
        );
    }
    #[test]
    fn test_guide_signature_subscribe() {
        guide_and_assert(
            serde_json::json! {{
                "method": "signatureSubscribe",
                "params": [
                    "2EBVM6cB8vAAD93Ktr6Vd8p67XPbQzCJX47MpReuiCXJAtcjaxpvWpcg9Ege1Nr5Tk3a2GFrByT7WPBjdsTycY9b", {
                    "commitment": "finalized",
                    "enableReceivedNotification": false
                }]
            }},
            &GuideStrategy::TryEphemeralForSignature(
                "2EBVM6cB8vAAD93Ktr6Vd8p67XPbQzCJX47MpReuiCXJAtcjaxpvWpcg9Ege1Nr5Tk3a2GFrByT7WPBjdsTycY9b".to_string(),
                true,
            ),
        );
    }
    #[test]
    fn test_guide_log_subscribe() {
        guide_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [
                    {
                        "mentions": ["2EBVM6cB8vAAD93Ktr6Vd8p67XPbQzCJX47MpReuiCXJAtcjaxpvWpcg9Ege1Nr5Tk3a2GFrByT7WPBjdsTycY9b"]
                    }
                ]
            }},
            &GuideStrategy::TryEphemeralForSignature(
                "2EBVM6cB8vAAD93Ktr6Vd8p67XPbQzCJX47MpReuiCXJAtcjaxpvWpcg9Ege1Nr5Tk3a2GFrByT7WPBjdsTycY9b".to_string(),
                true,
            ),
        );
        guide_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ "all"]
            }},
            &GuideStrategy::Ephemeral,
        );
        guide_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ "allWithVotes"]
            }},
            &GuideStrategy::Chain,
        )
    }
    #[test]
    fn test_guide_slot_subscribe() {
        guide_and_assert(
            serde_json::json! {{
                "method": "slotSubscribe"
            }},
            &GuideStrategy::Ephemeral,
        );
    }
    #[test]
    fn test_guide_root_subscribe() {
        guide_and_assert(
            serde_json::json! {{
                "method": "rootSubscribe"
            }},
            &GuideStrategy::Chain,
        );
    }
    #[test]
    fn test_guide_account_unsubscribe() {
        guide_and_assert(
            serde_json::json! {{
                "method": "accountUnsubscribe",
                "params": [0]
            }},
            &GuideStrategy::Both,
        );
    }
    #[test]
    fn test_guide_account_unknown() {
        guide_and_assert(
            serde_json::json! {{
                "method": "someNewUnsubscribe"
            }},
            &GuideStrategy::Chain,
        );
    }
}

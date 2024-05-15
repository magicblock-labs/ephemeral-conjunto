use serde::{Deserialize, Deserializer};
use solana_rpc_client_api::config::RpcTransactionLogsFilter;

use crate::errors::DirectorPubsubError;

// -----------------
// ClientSubMethod
// -----------------
/// Message which only pulls out the method when deserialized
#[derive(Deserialize)]
pub struct ClientSubMethodMessage {
    pub method: ClientSubMethod,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClientSubMethod {
    Ping,
    Pong,
    AccountSubscribe,
    AccountUnsubscribe,
    BlockSubscribe,
    BlockUnsubscribe,
    LogsSubscribe,
    LogsUnsubscribe,
    ProgramSubscribe,
    ProgramUnsubscribe,
    RootSubscribe,
    RootUnsubscribe,
    SignatureSubscribe,
    SignatureUnsubscribe,
    SlotSubscribe,
    SlotUnsubscribe,
    SlotsUpdatesSubscribe,
    SlotsUpdatesUnsubscribe,
    VoteSubscribe,
    VoteUnsubscribe,
}

impl TryFrom<&str> for ClientSubMethod {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let msg = serde_json::from_str::<ClientSubMethodMessage>(value)?;
        Ok(msg.method)
    }
}
// -----------------
// ClientSubWithParams
// -----------------
#[derive(Debug, Deserialize)]
pub struct GetAddressParam(
    pub String,
    // I could not get serde skip to work properly without failing the parse
    #[serde(deserialize_with = "de_ignore", default)] pub (),
);

#[derive(Debug, Deserialize)]
pub struct GetLogFilterParam(
    pub RpcTransactionLogsFilter,
    #[serde(deserialize_with = "de_ignore", default)] pub (),
);

fn de_ignore<'de, D>(_deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    Ok(())
}

/// Params which only pull out the information we need to guide the
/// subscription request when deserialized.
/// For all other methods we don't need the params in order to make that
/// decision
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Deserialize)]
pub enum ClientSubWithParams {
    /// Handled by the ephem validator except for two cases:
    /// - AllWithVotes
    /// - Mentions of an account that the ephem validator does not have
    // NOTE: this needs to be first so `params: [ all ]` does not match the
    //       less specific `SingleAddressParamSubscribe` below
    #[serde(untagged)]
    LogsSubscribe { params: GetLogFilterParam },
    /// AccountSubscribe:
    /// - handled by the ephem cluster if it has that account, otherwise by chain
    /// ProgramSubscribe:
    /// - handled by the ephem validator if it has that program, otherwise by chain
    /// SignatureSubscribe
    /// - Handled by the ephem validator if it has that signature, otherwise by chain
    #[serde(untagged)]
    SingleAddressParamSubscribe { params: GetAddressParam },
}

#[derive(Deserialize)]
pub struct ClientSubParamsMessage {
    pub params: ClientSubWithParams,
}

impl TryFrom<&str> for ClientSubWithParams {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let params = serde_json::from_str::<ClientSubWithParams>(value)?;
        Ok(params)
    }
}

// -----------------
// ParsedClientSub
// -----------------
#[derive(Debug, PartialEq, Eq)]
pub enum ParsedClientMessage {
    // The web3js client sends `{ 'method': 'ping' }` messages instead of sending
    // a proper websocket Message::Ping
    Ping,
    Pong,

    AccountSubscribe { address: String },
    AccountUnsubscribe,

    BlockSubscribe,
    BlockUnsubscribe,

    LogsSubscribe { filter: RpcTransactionLogsFilter },
    LogsUnsubscribe,

    ProgramSubscribe { program_id: String },
    ProgramUnsubscribe,

    RootSubscribe,
    RootUnsubscribe,

    SignatureSubscribe { signature: String },
    SignatureUnsubscribe,

    SlotSubscribe,
    SlotUnsubscribe,
    SlotsUpdatesSubscribe,
    SlotsUpdatesUnsubscribe,
    VoteSubscribe,
    VoteUnsubscribe,
}

impl TryFrom<&str> for ParsedClientMessage {
    type Error = DirectorPubsubError;

    fn try_from(msg: &str) -> Result<Self, Self::Error> {
        let method = ClientSubMethod::try_from(msg)?;
        use ClientSubMethod::*;
        match method {
            Ping => Ok(Self::Ping),
            Pong => Ok(Self::Pong),
            AccountSubscribe => {
                let params = ClientSubWithParams::try_from(msg)?;
                match params {
                    ClientSubWithParams::SingleAddressParamSubscribe {
                        params,
                    } => Ok(Self::AccountSubscribe {
                        address: params.0,
                    }),
                    _ => Err(DirectorPubsubError::ParseClientSubscription(
                        "Expected AccountSubscribe params holding single address"
                            .to_string(),
                    )),
                }
            }
            AccountUnsubscribe => Ok(Self::AccountUnsubscribe),
            BlockSubscribe => Ok(Self::BlockSubscribe),
            BlockUnsubscribe => Ok(Self::BlockUnsubscribe),
            LogsSubscribe => {
                let params = ClientSubWithParams::try_from(msg)?;
                match params {
                    ClientSubWithParams::LogsSubscribe { params } => {
                        Ok(Self::LogsSubscribe { filter: params.0 })
                    }
                    _ => Err(DirectorPubsubError::ParseClientSubscription(
                        "Expected LogsSubscribe params".to_string(),
                    )),
                }
            }
            LogsUnsubscribe => Ok(Self::LogsUnsubscribe),
            ProgramSubscribe => {
                let params = ClientSubWithParams::try_from(msg)?;
                match params {
                    ClientSubWithParams::SingleAddressParamSubscribe {
                        params,
                    } => Ok(Self::ProgramSubscribe {
                        program_id: params.0,
                    }),
                    _ => Err(DirectorPubsubError::ParseClientSubscription(
                        "Expected ProgramSubscribe params holding single address"
                            .to_string(),
                    )),
                }
            }
            ProgramUnsubscribe => Ok(Self::ProgramUnsubscribe),
            RootSubscribe => Ok(Self::RootSubscribe),
            RootUnsubscribe => Ok(Self::RootUnsubscribe),
            SignatureSubscribe => {
                let params = ClientSubWithParams::try_from(msg)?;
                match params {
                    ClientSubWithParams::SingleAddressParamSubscribe {
                        params,
                    } => Ok(Self::SignatureSubscribe {
                        signature: params.0,
                    }),
                    _ => Err(DirectorPubsubError::ParseClientSubscription(
                        "Expected SignatureSubscribe params holding single address"
                            .to_string(),
                    )),
                }
            }
            SignatureUnsubscribe => Ok(Self::SignatureUnsubscribe),
            SlotSubscribe => Ok(Self::SlotSubscribe),
            SlotUnsubscribe => Ok(Self::SlotUnsubscribe),
            SlotsUpdatesSubscribe => Ok(Self::SlotsUpdatesSubscribe),
            SlotsUpdatesUnsubscribe => Ok(Self::SlotsUpdatesUnsubscribe),
            VoteSubscribe => Ok(Self::VoteSubscribe),
            VoteUnsubscribe => Ok(Self::VoteUnsubscribe),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn parse_and_assert(msg: Value, expected: &ParsedClientMessage) {
        let actual =
            ParsedClientMessage::try_from(msg.to_string().as_str()).unwrap();
        assert_eq!(&actual, expected);
    }

    #[test]
    fn test_parse_account_subscribe() {
        let expected = ParsedClientMessage::AccountSubscribe {
            address: "SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ".to_string(),
        };
        parse_and_assert(
            // No Options
            serde_json::json! {{
                "method": "accountSubscribe",
                "params": ["SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ"]
            }},
            &expected,
        );
        parse_and_assert(
            // Non Empty Options
            serde_json::json! {{
                "method": "accountSubscribe",
                "params": ["SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ", {
                    "encoding": "base58",
                    "commitment": "confirmed"
                }]
            }},
            &expected,
        );
    }

    #[test]
    fn test_parse_program_subscribe() {
        let expected = ParsedClientMessage::ProgramSubscribe {
            program_id: "11111111111111111111111111111111".to_string(),
        };
        parse_and_assert(
            // No Options
            serde_json::json! {{
                "method": "programSubscribe",
                "params": ["11111111111111111111111111111111"]
            }},
            &expected,
        );
        parse_and_assert(
            // Empty Options
            serde_json::json! {{
                "method": "programSubscribe",
                "params": ["11111111111111111111111111111111", {}]
            }},
            &expected,
        );
        parse_and_assert(
            // Non Empty Options
            serde_json::json! {{
                "method": "programSubscribe",
                "params": ["11111111111111111111111111111111", {
                    "encoding": "base58",
                    "filters": [
                        {
                            "dataSize": 0
                        }
                    ]
                }]
            }},
            &expected,
        );
    }

    #[test]
    fn test_parse_signature_subscribe() {
        let expected = ParsedClientMessage::SignatureSubscribe {
            signature: "2EBVM6cB8vAAD93Ktr6Vd8p67XPbQzCJX47MpReuiCXJAtcjaxpvWpcg9Ege1Nr5Tk3a2GFrByT7WPBjdsTycY9b".to_string(),
        };
        parse_and_assert(
            // No Options
            serde_json::json! {{
                "method": "signatureSubscribe",
                "params": ["2EBVM6cB8vAAD93Ktr6Vd8p67XPbQzCJX47MpReuiCXJAtcjaxpvWpcg9Ege1Nr5Tk3a2GFrByT7WPBjdsTycY9b"]
            }},
            &expected,
        );
        parse_and_assert(
            // Empty Options
            serde_json::json! {{
                "method": "signatureSubscribe",
                "params": [
                    "2EBVM6cB8vAAD93Ktr6Vd8p67XPbQzCJX47MpReuiCXJAtcjaxpvWpcg9Ege1Nr5Tk3a2GFrByT7WPBjdsTycY9b",
                    {}
                ]
            }},
            &expected,
        );
        parse_and_assert(
            // Non Empty Options
            serde_json::json! {{
                "method": "signatureSubscribe",
                "params": [
                    "2EBVM6cB8vAAD93Ktr6Vd8p67XPbQzCJX47MpReuiCXJAtcjaxpvWpcg9Ege1Nr5Tk3a2GFrByT7WPBjdsTycY9b", {
                    "commitment": "finalized",
                    "enableReceivedNotification": false
                }]
            }},
            &expected,
        );
    }

    #[test]
    fn test_parse_log_subscribe() {
        // Without Options
        parse_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ "all"]
            }},
            &ParsedClientMessage::LogsSubscribe {
                filter: RpcTransactionLogsFilter::All,
            },
        );
        parse_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ "allWithVotes"]
            }},
            &ParsedClientMessage::LogsSubscribe {
                filter: RpcTransactionLogsFilter::AllWithVotes,
            },
        );
        parse_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ { "mentions": [ "SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ" ] }]
            }},
            &ParsedClientMessage::LogsSubscribe {
                filter: RpcTransactionLogsFilter::Mentions(vec![
                    "SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ".to_string(),
                ]),
            },
        );

        // With Empty Options
        parse_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ "all", {}]
            }},
            &ParsedClientMessage::LogsSubscribe {
                filter: RpcTransactionLogsFilter::All,
            },
        );
        parse_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ "allWithVotes", {}]
            }},
            &ParsedClientMessage::LogsSubscribe {
                filter: RpcTransactionLogsFilter::AllWithVotes,
            },
        );
        parse_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ { "mentions": [ "SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ" ] }, {}]
            }},
            &ParsedClientMessage::LogsSubscribe {
                filter: RpcTransactionLogsFilter::Mentions(vec![
                    "SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ".to_string(),
                ]),
            },
        );

        // With Non Empty Options
        parse_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ "all", { "commitment": "finalized" }]
            }},
            &ParsedClientMessage::LogsSubscribe {
                filter: RpcTransactionLogsFilter::All,
            },
        );
        parse_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ "allWithVotes", { "commitment": "finalized" }]
            }},
            &ParsedClientMessage::LogsSubscribe {
                filter: RpcTransactionLogsFilter::AllWithVotes,
            },
        );
        parse_and_assert(
            serde_json::json! {{
                "method": "logsSubscribe",
                "params": [ { "mentions": [ "SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ" ] }, { "commitment": "finalized" }]
            }},
            &ParsedClientMessage::LogsSubscribe {
                filter: RpcTransactionLogsFilter::Mentions(vec![
                    "SoLXmnP9JvL6vJ7TN1VqtTxqsc2izmPfF9CsMDEuRzJ".to_string(),
                ]),
            },
        );
    }

    #[test]
    fn test_non_parametrized() {
        parse_and_assert(
            serde_json::json! {{
                "method": "accountUnsubscribe",
                "params": [0]
            }},
            &ParsedClientMessage::AccountUnsubscribe,
        );
        parse_and_assert(
            serde_json::json! {{
                "method": "slotSubscribe"
            }},
            &ParsedClientMessage::SlotSubscribe,
        );
    }
}

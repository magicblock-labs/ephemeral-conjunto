// NOTE: from sleipnir rpc via solana rpc

use core::any::type_name;

use base64::{prelude::BASE64_STANDARD, Engine};
use bincode::Options;
use jsonrpsee::core::RpcResult;
use solana_sdk::packet::PACKET_DATA_SIZE;
use solana_transaction_status::TransactionBinaryEncoding;

use crate::utils::invalid_params;

const MAX_BASE58_SIZE: usize = 1683; // Depends on PACKET_DATA_SIZE
const MAX_BASE64_SIZE: usize = 1644; // Depends on PACKET_DATA_SIZE
pub(crate) fn decode_and_deserialize<T>(
    encoded: &str,
    encoding: TransactionBinaryEncoding,
) -> RpcResult<(Vec<u8>, T)>
where
    T: serde::de::DeserializeOwned,
{
    let wire_output = match encoding {
        TransactionBinaryEncoding::Base58 => {
            if encoded.len() > MAX_BASE58_SIZE {
                return Err(invalid_params(format!(
                    "base58 encoded {} too large: {} bytes (max: encoded/raw {}/{})",
                    type_name::<T>(),
                    encoded.len(),
                    MAX_BASE58_SIZE,
                    PACKET_DATA_SIZE,
                )));
            }
            bs58::decode(encoded).into_vec().map_err(|e| {
                invalid_params(format!("invalid base58 encoding: {e:?}"))
            })?
        }
        TransactionBinaryEncoding::Base64 => {
            if encoded.len() > MAX_BASE64_SIZE {
                return Err(invalid_params(format!(
                    "base64 encoded {} too large: {} bytes (max: encoded/raw {}/{})",
                    type_name::<T>(),
                    encoded.len(),
                    MAX_BASE64_SIZE,
                    PACKET_DATA_SIZE,
                )));
            }
            BASE64_STANDARD.decode(encoded).map_err(|e| {
                invalid_params(format!("invalid base64 encoding: {e:?}"))
            })?
        }
    };
    if wire_output.len() > PACKET_DATA_SIZE {
        return Err(invalid_params(format!(
            "decoded {} too large: {} bytes (max: {} bytes)",
            type_name::<T>(),
            wire_output.len(),
            PACKET_DATA_SIZE
        )));
    }
    bincode::options()
        .with_limit(PACKET_DATA_SIZE as u64)
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .deserialize_from(&wire_output[..])
        .map_err(|err| {
            invalid_params(format!(
                "failed to deserialize {}: {}",
                type_name::<T>(),
                &err.to_string()
            ))
        })
        .map(|output| (wire_output, output))
}

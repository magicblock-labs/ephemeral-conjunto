use conjunto_core::{
    AccountProvider, GuideStrategy, RequestEndpoint, SignatureStatusProvider,
};
use log::*;

pub struct GuideStrategyResolver<T: AccountProvider, U: SignatureStatusProvider>
{
    pub ephemeral_account_provider: T,
    pub ephemeral_signature_status_provider: U,
}

impl<T: AccountProvider, U: SignatureStatusProvider>
    GuideStrategyResolver<T, U>
{
    pub fn new(
        ephemeral_account_provider: T,
        ephemeral_signature_status_provider: U,
    ) -> Self {
        Self {
            ephemeral_account_provider,
            ephemeral_signature_status_provider,
        }
    }

    pub async fn resolve(&self, strategy: &GuideStrategy) -> RequestEndpoint {
        use GuideStrategy::*;

        match strategy {
            Chain => RequestEndpoint::Chain,
            Ephemeral => RequestEndpoint::Ephemeral,
            Both => RequestEndpoint::Both,
            TryEphemeralForAccount(address, is_subscription) => {
                self.guide_by_address(address, false, *is_subscription)
                    .await
            }
            TryEphemeralForProgram(program_id, is_subscription) => {
                self.guide_by_address(program_id, true, *is_subscription)
                    .await
            }
            TryEphemeralForSignature(signature, is_subscription) => {
                self.guide_by_signature(signature.as_str(), *is_subscription)
                    .await
            }
        }
    }

    async fn guide_by_signature(
        &self,
        signature: &str,
        is_subscription: bool,
    ) -> RequestEndpoint {
        let signature = match signature.parse() {
            Ok(signature) => signature,
            Err(_) => return RequestEndpoint::Chain,
        };
        match self
            .ephemeral_signature_status_provider
            .get_signature_status(&signature)
            .await
        {
            Ok(Some(_)) => RequestEndpoint::Ephemeral,
            // Wait for any of the backends to see an update to the signature
            Ok(None) if is_subscription => RequestEndpoint::Both,
            Ok(None) => RequestEndpoint::Chain,
            Err(err) => {
                warn!("Error while fetching signature status: {:?}", err);
                // In case of an error the signature does not exist or the RPC client
                // ran into an issue. In both cases we default to chain
                RequestEndpoint::Chain
            }
        }
    }

    async fn guide_by_address(
        &self,
        address: &str,
        is_program: bool,
        is_subscription: bool,
    ) -> RequestEndpoint {
        // If we find an invalid pubkey provided as an address then we forward
        // that to chain which will provide an error to the user
        let pubkey = match address.parse() {
            Ok(pubkey) => pubkey,
            Err(_) => return RequestEndpoint::Chain,
        };
        let account =
            match self.ephemeral_account_provider.get_account(&pubkey).await {
                Ok(Some(acc)) => acc,
                // If the ephemeral validator does not have he account then we go to chain for
                // single requests and to both for subscriptions (since the account may be created
                // after the subscription)
                Ok(None) if is_subscription => return RequestEndpoint::Both,
                Ok(None) => return RequestEndpoint::Chain,
                Err(err) => {
                    warn!("Error while fetching account: {:?}", err);
                    // In case of an error the account does not exist or the RPC client
                    // ran into an issue. In both cases we default to chain
                    return RequestEndpoint::Chain;
                }
            };
        if is_program && !account.executable {
            RequestEndpoint::Chain
        } else {
            RequestEndpoint::Ephemeral
        }
    }
}

// -----------------
// GuideStrategy
// -----------------
#[derive(Debug, PartialEq, Eq)]
pub enum GuideStrategy {
    /// Forward to chain
    Chain,
    /// Forward to ephemeral
    Ephemeral,
    /// Forward to both chain and ephemeral
    Both,
    /// Forward to ephemeral if that validator has the account of given address,
    /// otherwise forward to chain
    /// - *param.0*: address
    /// - *param.1*: is_subscription
    TryEphemeralForAccount(String, bool),
    /// Forward to ephemeral if that validator has the program of given address,
    /// otherwise forward to chain
    /// - *param.0*: program_id
    /// - *param.1*: is_subscription
    TryEphemeralForProgram(String, bool),
    /// Forward to ephemeral if that validator has the transaction signature,
    /// otherwise forward to both for subscriptions since the transaction may come
    /// in after the request.
    /// For single requests forward to ephemeral if the signature is found, otherwise
    /// to chain
    /// - *param.0*: signature
    /// - *param.1*: is_subscription
    TryEphemeralForSignature(String, bool),
}

// -----------------
// RequestEndpoint
// -----------------
#[derive(Debug, PartialEq, Eq)]
pub enum RequestEndpoint {
    /// Forward to chain only
    Chain,
    /// Forward to ephemeral only
    Ephemeral,
    /// Forward to both chain and ephemeral
    Both,
}

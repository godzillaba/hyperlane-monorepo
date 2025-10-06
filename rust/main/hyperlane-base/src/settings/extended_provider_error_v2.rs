use thiserror::Error;
use ethers_providers::ProviderError;
use hyperlane_core::H256;

#[derive(Debug, Error)]
pub enum ExtendedProviderError {
    // Re-export all ProviderError variants
    #[error(transparent)]
    JsonRpcClientError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("ens name not found: {0}")]
    EnsError(String),

    #[error("reverse ens name not pointing to itself: {0}")]
    EnsNotOwned(String),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    HexError(#[from] hex::FromHexError),

    #[error(transparent)]
    HTTPError(#[from] reqwest::Error),

    #[error("custom error: {0}")]
    CustomError(String),

    #[error("unsupported RPC")]
    UnsupportedRPC,

    #[error("unsupported node client")]
    UnsupportedNodeClient,

    #[error("Attempted to sign a transaction with no available signer. Hint: did you mean to use a SignerMiddleware?")]
    SignerUnavailable,

    // Additional Hyperlane-specific errors
    #[error("Node signer not configured for chain")]
    NodeSignerNotConfigured,

    #[error("Hyperlane mailbox not found at address: {address}")]
    MailboxNotFound { address: H256 },

    #[error("Invalid chain configuration: {reason}")]
    InvalidChainConfig { reason: String },

    #[error("Checkpoint validation failed: {reason}")]
    CheckpointValidationFailed { reason: String },

    #[error("Message dispatch failed: {reason}")]
    MessageDispatchFailed { reason: String },

    #[error("Agent signer error: {0}")]
    AgentSignerError(String),
}

// Conversion from ProviderError
impl From<ProviderError> for ExtendedProviderError {
    fn from(err: ProviderError) -> Self {
        match err {
            ProviderError::JsonRpcClientError(e) => ExtendedProviderError::JsonRpcClientError(e),
            ProviderError::EnsError(s) => ExtendedProviderError::EnsError(s),
            ProviderError::EnsNotOwned(s) => ExtendedProviderError::EnsNotOwned(s),
            ProviderError::SerdeJson(e) => ExtendedProviderError::SerdeJson(e),
            ProviderError::HexError(e) => ExtendedProviderError::HexError(e),
            ProviderError::HTTPError(e) => ExtendedProviderError::HTTPError(e),
            ProviderError::CustomError(s) => ExtendedProviderError::CustomError(s),
            ProviderError::UnsupportedRPC => ExtendedProviderError::UnsupportedRPC,
            ProviderError::UnsupportedNodeClient => ExtendedProviderError::UnsupportedNodeClient,
            ProviderError::SignerUnavailable => ExtendedProviderError::SignerUnavailable,
        }
    }
}
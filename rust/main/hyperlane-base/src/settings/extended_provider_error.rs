use thiserror::Error;
use ethers_providers::ProviderError;

#[derive(Debug, Error)]
pub enum HyperlaneProviderError {
    #[error(transparent)]
    EthersProvider(#[from] ProviderError),
    
    #[error("Hyperlane signer error: {0}")]
    SignerError(String),
    
    #[error("Chain configuration error: {message}")]
    ChainConfigError { message: String },
    
    #[error("Node signer delegation failed")]
    NodeSignerDelegationFailed,
    
    #[error("RPC endpoint unavailable: {endpoint}")]
    RpcUnavailable { endpoint: String },
    
    #[error("Mailbox contract error: {0}")]
    MailboxError(String),
}

impl From<String> for HyperlaneProviderError {
    fn from(msg: String) -> Self {
        Self::SignerError(msg)
    }
}

impl From<&str> for HyperlaneProviderError {
    fn from(msg: &str) -> Self {
        Self::SignerError(msg.to_string())
    }
}
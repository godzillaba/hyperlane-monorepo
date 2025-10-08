use core::panic;

use async_trait::async_trait;
use ethers::prelude::{Address, Signature};
use ethers::providers::{Http, Middleware, Provider, ProviderError};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::transaction::eip712::Eip712;
use ethers::types::Bytes;
use ethers::utils::rlp::Rlp;
use ethers_signers::{AwsSigner, AwsSignerError, LocalWallet, Signer, WalletError};

use hex::FromHex;
use hyperlane_core::{
    HyperlaneSigner, HyperlaneSignerError, Signature as HyperlaneSignature, H160, H256,
};

mod singleton;
pub use singleton::*;

#[derive(Debug, Clone)]
/// A signer that uses an unlocked Ethereum node via HTTP JSON-RPC to sign messages and transactions.
pub struct UnlockedNodeSigner {
    /// An HTTP-based Ethereum JSON-RPC provider used to interact with the blockchain.
    /// Must have address unlocked for signing operations.
    pub provider: Provider<Http>,
    /// The chain ID associated with the Ethereum network.
    /// Must match the chain ID of the node.
    pub chain_id: u64,
    /// The Ethereum address of the signer.
    /// Must be unlocked on the node.
    pub address: Address,
}

impl UnlockedNodeSigner {
    /// Creates a new UnlockedNodeSigner with the given provider and address.
    pub async fn new(provider: Provider<Http>, address: Address) -> Result<Self, ProviderError> {
        let chain_id = provider.get_chainid().await?.as_u64();
        Ok(Self {
            provider,
            chain_id,
            address,
        })
    }
}

#[async_trait]
impl Signer for UnlockedNodeSigner {
    /// Signs the hash of the provided message after prefixing it
    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        let bytes: ethers::types::Bytes = Bytes::from(Vec::from(message.as_ref()));
        Ok(self.provider.sign(bytes, &self.address).await?)
    }

    /// Signs the transaction
    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        let mut tx_obj = message.clone();
        tx_obj.set_from(self.address);
        let tx_str = ethers::utils::serialize(&tx_obj);
        let signed_tx: String = self
            .provider
            .request("eth_signTransaction", [tx_str])
            .await?;
        let signed_tx_bytes = Vec::from_hex(signed_tx.trim_start_matches("0x"))?;
        let rlp = Rlp::new(&signed_tx_bytes);
        let (_tx, sig) = TypedTransaction::decode_signed(&rlp)?;
        Ok(sig)
    }

    /// Encodes and signs the typed data according EIP-712.
    /// Payload must implement Eip712 trait.
    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        self.sign_message(payload.encode_eip712().unwrap()).await
    }

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address {
        self.address
    }

    /// Returns the signer's chain id
    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Sets the signer's chain id
    #[must_use]
    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        let id = chain_id.into();
        if &id == &self.chain_id {
            self
        } else {
            panic!("wrong chain_id"); // todo: this is probably not ideal behavior
        }
    }

    type Error = UnlockedNodeSignerError;
}

/// Error types for UnlockedNodeSigner
#[derive(Debug, thiserror::Error)]
pub enum UnlockedNodeSignerError {
    /// Provider Error
    #[error("{0}")]
    ProviderError(#[from] ProviderError),
    /// Signature parsing error
    #[error("{0}")]
    SignatureError(#[from] ethers::core::types::SignatureError),
    /// Hex decoding error
    #[error("{0}")]
    HexError(#[from] hex::FromHexError),
    /// Transaction request error
    #[error("{0}")]
    TransactionRequestError(#[from] ethers::types::transaction::eip2718::TypedTransactionError),
}

/// Ethereum-supported signer types
#[derive(Debug, Clone)]
pub enum Signers {
    /// A wallet instantiated with a locally stored private key
    Local(LocalWallet),
    /// A signer using a key stored in aws kms
    Aws(AwsSigner),
    /// Node-based signer that delegates to RPC provider
    UnlockedNode(UnlockedNodeSigner),
}

impl From<LocalWallet> for Signers {
    fn from(s: LocalWallet) -> Self {
        Signers::Local(s)
    }
}

impl From<AwsSigner> for Signers {
    fn from(s: AwsSigner) -> Self {
        Signers::Aws(s)
    }
}

impl From<UnlockedNodeSigner> for Signers {
    fn from(s: UnlockedNodeSigner) -> Self {
        Signers::UnlockedNode(s)
    }
}

#[async_trait]
impl Signer for Signers {
    type Error = SignersError;

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_message(message).await?),
            Signers::Aws(signer) => Ok(signer.sign_message(message).await?),
            Signers::UnlockedNode(signer) => Ok(signer.sign_message(message).await?),
        }
    }

    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_transaction(message).await?),
            Signers::Aws(signer) => Ok(signer.sign_transaction(message).await?),
            Signers::UnlockedNode(signer) => Ok(signer.sign_transaction(message).await?),
        }
    }

    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_typed_data(payload).await?),
            Signers::Aws(signer) => Ok(signer.sign_typed_data(payload).await?),
            Signers::UnlockedNode(signer) => Ok(signer.sign_typed_data(payload).await?),
        }
    }

    fn address(&self) -> Address {
        match self {
            Signers::Local(signer) => signer.address(),
            Signers::Aws(signer) => signer.address(),
            Signers::UnlockedNode(signer) => signer.address(),
        }
    }

    fn chain_id(&self) -> u64 {
        match self {
            Signers::Local(signer) => signer.chain_id(),
            Signers::Aws(signer) => signer.chain_id(),
            Signers::UnlockedNode(signer) => signer.chain_id(),
        }
    }

    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        match self {
            Signers::Local(signer) => signer.with_chain_id(chain_id).into(),
            Signers::Aws(signer) => signer.with_chain_id(chain_id).into(),
            Signers::UnlockedNode(signer) => signer.with_chain_id(chain_id).into(),
        }
    }
}

#[async_trait]
impl HyperlaneSigner for Signers {
    fn eth_address(&self) -> H160 {
        Signer::address(self).into()
    }

    async fn sign_hash(&self, hash: &H256) -> Result<HyperlaneSignature, HyperlaneSignerError> {
        let mut signature = Signer::sign_message(self, hash)
            .await
            .map_err(|err| HyperlaneSignerError::from(Box::new(err) as Box<_>))?;
        signature.v = 28 - (signature.v % 2);
        Ok(signature.into())
    }
}

/// Error types for Signers
#[derive(Debug, thiserror::Error)]
pub enum SignersError {
    /// AWS Signer Error
    #[error("{0}")]
    AwsSignerError(#[from] AwsSignerError),
    /// Wallet Signer Error
    #[error("{0}")]
    WalletError(#[from] WalletError),
    /// UnlockedNode Signer Error
    #[error("{0}")]
    UnlockedNodeSignerError(#[from] UnlockedNodeSignerError),
}

impl From<std::convert::Infallible> for SignersError {
    fn from(_error: std::convert::Infallible) -> Self {
        panic!("infallible")
    }
}

#[cfg(test)]
mod test {
    use hyperlane_core::{
        Checkpoint, CheckpointWithMessageId, HyperlaneSigner, HyperlaneSignerExt, H256,
    };

    use super::Signers;

    #[test]
    fn it_sign() {
        let t = async {
            let signer: Signers =
                "1111111111111111111111111111111111111111111111111111111111111111"
                    .parse::<ethers::signers::LocalWallet>()
                    .unwrap()
                    .into();
            let message = CheckpointWithMessageId {
                checkpoint: Checkpoint {
                    merkle_tree_hook_address: H256::repeat_byte(2),
                    mailbox_domain: 5,
                    root: H256::repeat_byte(1),
                    index: 123,
                },
                message_id: H256::repeat_byte(3),
            };

            let signed = signer.sign(message).await.expect("!sign");
            assert!(signed.signature.v == 27 || signed.signature.v == 28);
            signed.verify(signer.eth_address()).expect("!verify");
        };
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(t)
    }
}

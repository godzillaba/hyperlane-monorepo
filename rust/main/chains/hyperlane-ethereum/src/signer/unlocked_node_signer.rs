use async_trait::async_trait;
use ethers::{
    providers::{Http, Middleware, Provider, ProviderError},
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address, Bytes, Signature,
    },
    utils::rlp::Rlp,
};
use ethers_signers::Signer;
use hex::FromHex;

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

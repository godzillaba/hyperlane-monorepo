use core::panic;

use async_trait::async_trait;
use ethers::prelude::{Address, Signature};
use ethers::providers::{Http, Middleware, Provider, ProviderError};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::transaction::eip712::Eip712;
use ethers::types::{Bytes, Eip1559TransactionRequest, Transaction, TransactionRequest};
use ethers::utils::rlp::{Decodable, Rlp};
use ethers_signers::{AwsSigner, AwsSignerError, LocalWallet, Signer, WalletError};

use hex::FromHex;
use hyperlane_core::{
    HyperlaneSigner, HyperlaneSignerError, Signature as HyperlaneSignature, H160, H256,
};

mod singleton;
pub use singleton::*;

#[derive(Debug, Clone)]
pub struct UnlockedNodeSigner {
    pub provider: Provider<Http>,
    pub chain_id: u64,
    pub address: Address,
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
        println!("In sign_transaction");
        let mut tx_obj = message.clone();
        tx_obj.set_from(self.address);
        let tx_str = ethers::utils::serialize(&tx_obj);
        println!("TX: {:?}", tx_str);
        let signed_tx: String = self.provider.request("eth_signTransaction", [tx_str]).await?;
        println!("Signed TX: {:?}", signed_tx);
        
        
        
        let signed_tx_bytes = Vec::from_hex(signed_tx.trim_start_matches("0x"))?;
        let rlp = Rlp::new(&signed_tx_bytes);
        println!("RLP init");
        let (_tx, sig) = TypedTransaction::decode_signed(&rlp)?;

        Ok(sig)


        // let rlp = Rlp::new(&signed_tx_bytes);
        // let decoded_tx = TypedTransaction::decode(&rlp)?;
        // decoded_tx.sig


        // let y = TypedTransaction::decode(&Rlp::new(Vec::from_hex(signed_tx)));
        // let sig = signed_tx.parse::<Transaction>()?;
        

        // println!("Response: {:?}", res);
        // Ok(res.parse::<Signature>()?)
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

    // async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
    //     &self,
    //     message: S,
    // ) -> Result<Signature, Self::Error> {
    //     // let message = message.as_ref();
    //     // let message_hash = hash_message(message);
    //     // trace!("{:?}", message_hash);
    //     // trace!("{:?}", message);
    //     // self.provider.request("eth_sign", [self.address, message_hash]).await.map_err(ProviderError::from)
    //     // self.provider.request("eth_sign", [self.address, "oqwiefjoiqwefj"]).await.map_err(ProviderError::from)
    // }

    // #[doc = " Signs the transaction"]
    // #[must_use]
    // #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    // fn sign_transaction<'life0,'life1,'async_trait>(&'life0 self,message: &'life1 TypedTransaction) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<Signature,Self::Error> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait {
    //     self.provider.sign_transaction(message, self.address)
    // }

    // #[doc = " Encodes and signs the typed data according EIP-712."]
    // #[doc = " Payload must implement Eip712 trait."]
    // #[must_use]
    // #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    // fn sign_typed_data<'life0,'life1,'async_trait,T, >(&'life0 self,payload: &'life1 T,) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<Signature,Self::Error> > + ::core::marker::Send+'async_trait> >where T:'async_trait+Eip712+Send+Sync,'life0:'async_trait,'life1:'async_trait,Self:'async_trait {
    //     let raw = payload.encode_eip712().unwrap(); // todo: what to do instead of unwrap?
    //     self.sign_message(raw)
    // }

    // #[doc = " Returns the signer\'s Ethereum Address"]
    // fn address(&self) -> Address {
    //     self.address
    // }

    // #[doc = " Returns the signer\'s chain id"]
    // fn chain_id(&self) -> u64 {
    //     self.chain_id
    // }

    // #[doc = " Sets the signer\'s chain id"]
    // #[must_use]
    // fn with_chain_id<T:Into<u64> >(self,chain_id:T) -> Self {
    //     let id = chain_id.into();
    //     if &id == &self.chain_id {
    //         self
    //     } else {
    //         panic!("wrong chain_id"); // todo: this is probably not ideal behavior
    //     }
    // }
    
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
    /// RLP Decoding error
    #[error("{0}")]
    RlpDecodingError(#[from] ethers::utils::rlp::DecoderError),
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

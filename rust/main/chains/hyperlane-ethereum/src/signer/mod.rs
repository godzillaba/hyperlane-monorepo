use async_trait::async_trait;
use ethers::prelude::{Address, Signature};
use ethers::providers::{Http, Provider, ProviderError};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::transaction::eip712::Eip712;
use ethers_signers::{AwsSigner, AwsSignerError, LocalWallet, Signer, WalletError};

use hyperlane_core::{
    HyperlaneSigner, HyperlaneSignerError, Signature as HyperlaneSignature, H160, H256,
};

mod singleton;
pub use singleton::*;

#[derive(Debug, Clone)]
pub struct NodeSigner {
    provider: Provider<Http>,
    address: Address,
}

impl Signer for NodeSigner {
    #[doc = " Signs the hash of the provided message after prefixing it"]
    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn sign_message<'life0,'async_trait,S, >(&'life0 self,message:S,) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<Signature,Self::Error> > + ::core::marker::Send+'async_trait> >where S:'async_trait+Send+Sync+AsRef<[u8]> ,'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[doc = " Signs the transaction"]
    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn sign_transaction<'life0,'life1,'async_trait>(&'life0 self,message: &'life1 TypedTransaction) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<Signature,Self::Error> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait {
        todo!()
    }

    #[doc = " Encodes and signs the typed data according EIP-712."]
    #[doc = " Payload must implement Eip712 trait."]
    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn sign_typed_data<'life0,'life1,'async_trait,T, >(&'life0 self,payload: &'life1 T,) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<Signature,Self::Error> > + ::core::marker::Send+'async_trait> >where T:'async_trait+Eip712+Send+Sync,'life0:'async_trait,'life1:'async_trait,Self:'async_trait {
        todo!()
    }

    #[doc = " Returns the signer\'s Ethereum Address"]
    fn address(&self) -> Address {
        todo!()
    }

    #[doc = " Returns the signer\'s chain id"]
    fn chain_id(&self) -> u64 {
        todo!()
    }

    #[doc = " Sets the signer\'s chain id"]
    #[must_use]
    fn with_chain_id<T:Into<u64> >(self,chain_id:T) -> Self {
        todo!()
    }
    
    type Error = ProviderError;
}

/// Ethereum-supported signer types
#[derive(Debug, Clone)]
pub enum Signers {
    /// A wallet instantiated with a locally stored private key
    Local(LocalWallet),
    /// A signer using a key stored in aws kms
    Aws(AwsSigner),
    /// Node-based signer that delegates to RPC provider
    Node(NodeSigner),
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

impl From<NodeSigner> for Signers {
    fn from(s: NodeSigner) -> Self {
        Signers::Node(s)
    }
}

// impl From<ProviderError> for SignersError {
//     fn from(err: ProviderError) -> Self {
        
//     }
// }

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
            Signers::Node(signer) => Ok(signer.sign_message(message).await?),
        }
    }

    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_transaction(message).await?),
            Signers::Aws(signer) => Ok(signer.sign_transaction(message).await?),
            Signers::Node(signer) => Ok(signer.sign_transaction(message).await?),
        }
    }

    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_typed_data(payload).await?),
            Signers::Aws(signer) => Ok(signer.sign_typed_data(payload).await?),
            Signers::Node(signer) => Ok(signer.sign_typed_data(payload).await?),
        }
    }

    fn address(&self) -> Address {
        match self {
            Signers::Local(signer) => signer.address(),
            Signers::Aws(signer) => signer.address(),
            Signers::Node(signer) => signer.address(),
        }
    }

    fn chain_id(&self) -> u64 {
        match self {
            Signers::Local(signer) => signer.chain_id(),
            Signers::Aws(signer) => signer.chain_id(),
            Signers::Node(signer) => signer.chain_id(),
        }
    }

    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        match self {
            Signers::Local(signer) => signer.with_chain_id(chain_id).into(),
            Signers::Aws(signer) => signer.with_chain_id(chain_id).into(),
            Signers::Node(signer) => signer.with_chain_id(chain_id).into(),
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
    /// Provider Error
    #[error("{0}")]
    ProviderError(#[from] ProviderError),
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

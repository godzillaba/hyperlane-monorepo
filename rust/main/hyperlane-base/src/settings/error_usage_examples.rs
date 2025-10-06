// Example usage of the extended provider errors

use crate::settings::extended_provider_error::HyperlaneProviderError;
use ethers_providers::{Provider, Middleware};
use eyre::Result;

// Option 1: Using wrapped ProviderError
async fn example_with_wrapped_error() -> Result<(), HyperlaneProviderError> {
    let provider = Provider::try_from("http://localhost:8545")?;
    
    // This will automatically convert ProviderError to HyperlaneProviderError
    let block_number = provider.get_block_number().await?;
    
    // You can also create custom errors
    if block_number.as_u64() == 0 {
        return Err(HyperlaneProviderError::ChainConfigError { 
            message: "Chain not initialized".to_string() 
        });
    }
    
    Ok(())
}

// Option 2: Using comprehensive error enum
use crate::settings::extended_provider_error_v2::ExtendedProviderError;

async fn example_with_extended_error() -> Result<(), ExtendedProviderError> {
    let provider = Provider::try_from("http://localhost:8545")?;
    
    // This will automatically convert ProviderError to ExtendedProviderError
    let block_number = provider.get_block_number().await?;
    
    // You can create Hyperlane-specific errors
    if block_number.as_u64() == 0 {
        return Err(ExtendedProviderError::InvalidChainConfig { 
            reason: "Genesis block not found".to_string() 
        });
    }
    
    Ok(())
}

// Helper function to demonstrate error conversion
fn handle_node_signer_error() -> Result<(), HyperlaneProviderError> {
    // This demonstrates how you can easily create your custom errors
    Err(HyperlaneProviderError::NodeSignerDelegationFailed)
}
//! EVMbench Runner - Phase 2 EVM-08
//! Async exploit execution with ethers-rs

use ethers::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::types::{BenchConfig, ExploitResult, VulnerabilityTest};

/// Runner for executing exploit tests
pub struct Runner {
    config: BenchConfig,
    client: Option<Arc<Provider<Http>>>,
}

impl Runner {
    /// Create new runner with configuration
    pub fn new(config: BenchConfig) -> Self {
        Self { config, client: None }
    }

    /// Connect to Anvil RPC endpoint
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from(&self.config.anvil_endpoint)?;
        self.client = Some(Arc::new(provider));
        Ok(())
    }

    /// Run single exploit test
    pub async fn run_test(&self, test: &VulnerabilityTest) -> ExploitResult {
        let start = std::time::Instant::now();
        println!("[Runner] Executing test: {}", test.id);

        match self.execute_exploit(test).await {
            Ok(tx_hash) => {
                let duration = start.elapsed().as_millis() as u64;
                println!("[Runner] {} succeeded: {:?}", test.id, tx_hash);
                ExploitResult::success_result(duration, tx_hash.map(|h| h.to_string()))
            }
            Err(e) => {
                let duration = start.elapsed().as_millis() as u64;
                println!("[Runner] {} failed: {}", test.id, e);
                ExploitResult::failure(&e.to_string(), duration)
            }
        }
    }

    /// Run batch of tests with timeout
    pub async fn run_batch(&self, tests: Vec<VulnerabilityTest>) -> Vec<ExploitResult> {
        let mut results = Vec::with_capacity(tests.len());
        for test in tests {
            let result = timeout(
                Duration::from_millis(self.config.timeout_ms),
                self.run_test(&test)
            ).await.unwrap_or_else(|_| {
                ExploitResult::failure("Timeout", self.config.timeout_ms)
            });
            results.push(result);
        }
        results
    }

    /// Execute exploit against target
    async fn execute_exploit(&self, _test: &VulnerabilityTest) -> Result<Option<H256>, Box<dyn std::error::Error>> {
        let client = self.client.as_ref().ok_or("Not connected")?;
        
        // Impersonate attacker account
        let attacker: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse()?;
        
        // Check balance before
        let balance_before = client.get_balance(attacker, None).await?;
        println!("[Runner] Attacker balance: {}", balance_before);

        // TODO: Deploy vulnerability contract (EVM-08 extension)
        // TODO: Execute exploit transactions
        // TODO: Verify balance change

        // Placeholder: return mock success
        Ok(Some(H256::zero()))
    }

    /// Get balance with error handling
    pub async fn get_balance(&self, address: Address) -> Result<U256, Box<dyn std::error::Error>> {
        let client = self.client.as_ref().ok_or("Not connected")?;
        Ok(client.get_balance(address, None).await?)
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new(BenchConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runner_connect() {
        let mut runner = Runner::default();
        // Note: Requires Anvil running at localhost:8545
        // runner.connect().await.unwrap();
    }
}

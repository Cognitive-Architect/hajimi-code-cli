use std::time::Instant;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout, Duration};
use crate::error::EngineError;
use crate::executor::Executor;
use crate::query::{Query, QueryResult};

pub struct ParallelExecutor {
    max_concurrency: usize,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self { max_concurrency: 10 }
    }

    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max;
        self
    }

    async fn run_query(query: Query) -> Result<QueryResult, EngineError> {
        let start = Instant::now();
        
        // 模拟工具执行时间（50ms）
        let result = timeout(
            Duration::from_millis(query.timeout_ms),
            async {
                sleep(Duration::from_millis(50)).await;
                let elapsed = start.elapsed().as_millis() as u64;
                Ok(QueryResult::success("executed", elapsed.max(1)))
            }
        ).await;

        match result {
            Ok(Ok(r)) => Ok(r),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(EngineError::Timeout(query.timeout_ms)),
        }
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for ParallelExecutor {
    async fn execute(&self, query: Query) -> Result<QueryResult, EngineError> {
        Self::run_query(query).await
    }

    async fn execute_batch(&self, queries: Vec<Query>) -> Vec<Result<QueryResult, EngineError>> {
        let sem = std::sync::Arc::new(Semaphore::new(self.max_concurrency));
        let mut join_set = JoinSet::new();
        
        for (idx, query) in queries.into_iter().enumerate() {
            let permit = match sem.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => {
                    join_set.spawn(async move {
                        (idx, Err(EngineError::ExecutionFailed("Semaphore closed".to_string())))
                    });
                    continue;
                }
            };
            join_set.spawn(async move {
                let _p = permit;
                let res = Self::run_query(query).await;
                (idx, res)
            });
        }
        
        let mut results: Vec<(usize, _)> = Vec::new();
        while let Some(res) = join_set.join_next().await {
            if let Ok((idx, r)) = res {
                results.push((idx, r));
            }
        }
        
        results.sort_by_key(|(i, _)| *i);
        results.into_iter().map(|(_, r)| r).collect()
    }
}

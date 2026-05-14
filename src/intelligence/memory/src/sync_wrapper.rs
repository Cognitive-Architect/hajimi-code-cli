//! SyncWrapper: Make rusqlite Connection Sync-safe via channel-based access.
//! REWORK-001: 删除unsafe + 修复SQL注入为参数化查询

use rusqlite::{params, Connection};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

/// Thread-safe wrapper for rusqlite Connection.
pub struct SyncConnection {
    sender: mpsc::Sender<DbRequest>,
}

enum DbRequest {
    Execute {
        sql: String,
        p1: Option<String>,
        p2: Option<String>,
        resp: oneshot::Sender<Result<usize, String>>,
    },
}

impl SyncConnection {
    pub fn new(conn: Connection) -> Self {
        let (sender, mut rx) = mpsc::channel(100);
        tokio::spawn(async move {
            while let Some(req) = rx.recv().await {
                match req {
                    DbRequest::Execute { sql, p1, p2, resp } => {
                        let r = match (p1, p2) {
                            (Some(a), Some(b)) => conn.execute(&sql, params![a, b]),
                            (Some(a), None) => conn.execute(&sql, params![a]),
                            _ => conn.execute(&sql, []),
                        }
                        .map_err(|e| e.to_string());
                        let _ = resp.send(r);
                    }
                }
            }
        });
        Self { sender }
    }
    /// REWORK-001: 参数化查询，支持最多2个参数
    pub async fn execute(
        &self,
        sql: &str,
        p1: Option<&str>,
        p2: Option<&str>,
    ) -> Result<usize, String> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(DbRequest::Execute {
                sql: sql.to_string(),
                p1: p1.map(|s| s.to_string()),
                p2: p2.map(|s| s.to_string()),
                resp: tx,
            })
            .await
            .map_err(|_| "Channel closed".to_string())?;
        rx.await.map_err(|_| "Cancelled".to_string())?
    }
}

/// Sync-safe MemoryGateway.
/// REWORK-001: Arc<SyncConnection>自动实现Send+Sync，无需unsafe
pub struct SyncMemoryGateway {
    conn: Arc<SyncConnection>,
}

impl SyncMemoryGateway {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Arc::new(SyncConnection::new(conn)),
        }
    }
    pub async fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS plans (goal_id TEXT PRIMARY KEY, content TEXT)",
                None,
                None,
            )
            .await?;
        self.conn.execute("CREATE TABLE IF NOT EXISTS reflections (id TEXT PRIMARY KEY, goal_id TEXT, content TEXT)", None, None).await?;
        Ok(())
    }
    /// REWORK-001: 使用参数化查询防止SQL注入
    pub async fn store_plan(&self, goal_id: &str, content: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO plans (goal_id, content) VALUES (?1, ?2)",
                Some(goal_id),
                Some(content),
            )
            .await?;
        Ok(())
    }
    pub fn conn(&self) -> Arc<SyncConnection> {
        self.conn.clone()
    }
}

// REWORK-001: 删除unsafe impl - Arc<SyncConnection>自动实现Send+Sync

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_sync_conn() {
        let c = SyncConnection::new(Connection::open_in_memory().unwrap());
        assert!(c
            .execute("CREATE TABLE t (id INTEGER)", None, None)
            .await
            .is_ok());
    }
    #[tokio::test]
    async fn test_sync_gateway() {
        let g = SyncMemoryGateway::new(Connection::open_in_memory().unwrap());
        assert!(g.init_schema().await.is_ok());
        assert!(g.store_plan("g1", "content").await.is_ok());
    }
    // REWORK-001: SQL注入防护测试
    #[tokio::test]
    async fn test_sql_injection_safe() {
        let g = SyncMemoryGateway::new(Connection::open_in_memory().unwrap());
        g.init_schema().await.unwrap();
        // 尝试SQL注入 - 参数化查询应安全处理
        let malicious_id = "g1'; DROP TABLE plans; --";
        assert!(g.store_plan(malicious_id, "content").await.is_ok());
        // 表应仍然存在
        assert!(g.store_plan("g2", "test").await.is_ok());
    }
}

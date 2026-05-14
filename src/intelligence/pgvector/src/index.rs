//! pgvector HNSW index wrapper
#![deny(unsafe_code)]

use pgvector::Vector;
use sqlx::PgPool;

pub struct PgVectorIndex {
    pool: PgPool,
}

impl PgVectorIndex {
    pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
        // Enable pgvector extension
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&pool)
            .await?;

        // Create table with vector column
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                embedding VECTOR(384),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn create_hnsw_index(&self) -> Result<(), sqlx::Error> {
        // Create HNSW index for cosine similarity
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_embedding_hnsw ON memories
            USING hnsw (embedding vector_cosine_ops)
            WITH (m = 16, ef_construction = 200)
        "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_vector(&self, id: &str, embedding: [f32; 384]) -> Result<(), sqlx::Error> {
        let vec = Vector::from(embedding.to_vec());
        sqlx::query("INSERT INTO memories (id, embedding) VALUES ($1, $2)")
            .bind(id)
            .bind(vec)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn search_vectors(
        &self,
        query: [f32; 384],
        k: usize,
    ) -> Result<Vec<(String, f32)>, sqlx::Error> {
        let vec = Vector::from(query.to_vec());
        let rows = sqlx::query_as::<_, (String, f32)>(
            "SELECT id, embedding <=> $1::vector as distance FROM memories ORDER BY embedding <=> $1::vector LIMIT $2"
        )
        .bind(vec)
        .bind(k as i64)
        .fetch_all(&self.pool).await?;
        Ok(rows)
    }

    /// Ground truth: exact linear scan for recall validation
    pub async fn exact_search(
        &self,
        query: [f32; 384],
        k: usize,
    ) -> Result<Vec<(String, f32)>, sqlx::Error> {
        let vec = Vector::from(query.to_vec());
        let rows = sqlx::query_as::<_, (String, f32)>(
            "SELECT id, embedding <=> $1::vector as distance FROM memories ORDER BY embedding <=> $1::vector LIMIT $2"
        )
        .bind(vec)
        .bind(k as i64)
        .fetch_all(&self.pool).await?;
        Ok(rows)
    }
}

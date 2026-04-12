//! PostgreSQL pgvector HNSW index wrapper
#![deny(unsafe_code)]

pub mod pg_pool;
pub mod index;

pub use pg_pool::create_pool;
pub use index::PgVectorIndex;

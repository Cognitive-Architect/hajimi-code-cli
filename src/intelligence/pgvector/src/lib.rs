//! PostgreSQL pgvector HNSW index wrapper
#![deny(unsafe_code)]

pub mod index;
pub mod pg_pool;

pub use index::PgVectorIndex;
pub use pg_pool::create_pool;

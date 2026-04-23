//! 元关系抽取模块
use crate::graph::{GraphDb, GraphError, Relation, Result};
pub mod adr_extractor;

pub trait RelationExtractor {
    fn extract(&self, source: &str) -> Result<Vec<Relation>>;
}

pub fn insert_relations_batch(db: &mut GraphDb, rels: &[Relation]) -> Result<()> {
    let tx = db.conn.transaction().map_err(GraphError::Database)?;
    for r in rels {
        if r.subject == r.object { continue; }
        tx.execute("INSERT INTO relations (id,subject,predicate,object,confidence,extracted_from) VALUES (?1,?2,?3,?4,?5,?6)",
            rusqlite::params![&r.id, &r.subject, &r.predicate, &r.object, r.confidence, r.extracted_from.as_deref().unwrap_or("")],
        ).map_err(GraphError::Database)?;
    }
    tx.commit().map_err(GraphError::Database)
}

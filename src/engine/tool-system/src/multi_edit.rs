//! Multi-file transaction editing - B-W10/04
//! DEBT-ATOMIC-W10-04 [x] CLEARED: Now uses atomic_write from edit module
use super::{ToolError, ToolOutput};
use super::edit::atomic_write;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;

#[derive(Debug, Clone)]
pub struct EditOp { pub path: PathBuf, pub old_text: String, pub new_text: String }
impl EditOp { pub fn new(p: impl AsRef<Path>, o: impl Into<String>, n: impl Into<String>) -> Self { Self { path: p.as_ref().to_path_buf(), old_text: o.into(), new_text: n.into() } } }

#[derive(Debug, Clone, Default)]
pub struct EditPlan { pub ops: Vec<EditOp> }
impl EditPlan { pub fn new() -> Self { Self::default() } pub fn add(&mut self, o: EditOp) { self.ops.push(o); } pub fn is_empty(&self) -> bool { self.ops.is_empty() } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState { Pending, Committed, RolledBack }

#[derive(Debug)]
struct Snapshot { path: PathBuf, content: String, mtime: SystemTime, temp: PathBuf }

pub struct MultiEditTransaction { pub plan: EditPlan, pub dry_run: bool, state: TransactionState, snapshots: Vec<Snapshot>, temp_dir: PathBuf }

impl MultiEditTransaction {
    pub fn new(plan: EditPlan, dry_run: bool) -> Self { let temp_dir = std::env::temp_dir().join(format!("hajimi_tx_{}", uuid::Uuid::new_v4())); Self { plan, dry_run, state: TransactionState::Pending, snapshots: Vec::new(), temp_dir } }

    fn detect_circular_deps(&self) -> Result<(), ToolError> {
        let mut g: HashMap<&Path, HashSet<&Path>> = HashMap::new();
        for op in &self.plan.ops { let d = g.entry(&op.path).or_default(); for o in &self.plan.ops { if o.path != op.path && o.new_text.contains(&op.old_text) { d.insert(&o.path); } } }
        let mut v = HashSet::new(); let mut s = HashSet::new();
        fn cycle<'a>(n: &'a Path, g: &HashMap<&'a Path, HashSet<&'a Path>>, v: &mut HashSet<&'a Path>, s: &mut HashSet<&'a Path>) -> bool { v.insert(n); s.insert(n); if let Some(e) = g.get(n) { for &x in e { if !v.contains(x) && cycle(x, g, v, s) { return true; } if s.contains(x) { return true; } } } s.remove(n); false }
        for &n in g.keys() { if !v.contains(n) && cycle(n, &g, &mut v, &mut s) { return Err(ToolError::new("Circular dependency")); } } Ok(())
    }

    async fn check_space(&self) -> Result<(), ToolError> { fs::create_dir_all(&self.temp_dir).await.map_err(|e| ToolError::new(format!("Temp dir: {}", e)))?; let t = self.temp_dir.join(".c"); match fs::write(&t, [0u8; 1024]).await { Ok(_) => { let _ = fs::remove_file(&t).await; } Err(e) => return Err(ToolError::new(format!("Disk: {}", e))), } Ok(()) }

    async fn capture(&self, p: &Path) -> Result<Snapshot, ToolError> { let c = fs::read_to_string(p).await.map_err(|e| ToolError::new(format!("Read: {}", e)))?; let m = fs::metadata(p).await.map_err(|e| ToolError::new(format!("Meta: {}", e)))?; let t = m.modified().map_err(|e| ToolError::new(format!("Mtime: {}", e)))?; let tp = self.temp_dir.join(format!("{}", uuid::Uuid::new_v4())); fs::write(&tp, &c).await.map_err(|e| ToolError::new(format!("Snapshot: {}", e)))?; Ok(Snapshot { path: p.to_path_buf(), content: c, mtime: t, temp: tp }) }

    async fn check_mtime(&self, s: &Snapshot) -> Result<(), ToolError> { let m = fs::metadata(&s.path).await.map_err(|e| ToolError::new(format!("Check: {}", e)))?; let t = m.modified().map_err(|e| ToolError::new(format!("Mtime: {}", e)))?; if t != s.mtime { return Err(ToolError::new(format!("{} modified externally", s.path.display()))); } Ok(()) }

    pub async fn commit<F>(&mut self, mut pr: F) -> Result<ToolOutput, ToolError> where F: FnMut(usize, usize, &Path) {
        if self.state != TransactionState::Pending { return Err(ToolError::new("Already completed")); }
        if self.plan.is_empty() { return Ok(ToolOutput::success("No changes")); }
        self.detect_circular_deps()?; self.check_space().await?;
        if self.dry_run { let mut r = String::from("DRY RUN:\n"); for (i, o) in self.plan.ops.iter().enumerate() { pr(i + 1, self.plan.ops.len(), &o.path); r.push_str(&format!("[{}] {}\n", i + 1, o.path.display())); } self.state = TransactionState::Committed; return Ok(ToolOutput::success(r)); }
        let tot = self.plan.ops.len();
        for (i, o) in self.plan.ops.iter().enumerate() { pr(i + 1, tot, &o.path); if !o.path.exists() { return Err(ToolError::new(format!("Not found: {}", o.path.display()))); } self.snapshots.push(self.capture(&o.path).await?); }
        for (i, o) in self.plan.ops.iter().enumerate() { pr(i + 1, tot, &o.path); let s = &self.snapshots[i]; if let Err(e) = self.check_mtime(s).await { let _ = self.rollback_snap(tot).await; return Err(e); } let c = match fs::read_to_string(&o.path).await { Ok(c) => c, Err(e) => { let _ = self.rollback_snap(tot).await; return Err(ToolError::new(format!("Read: {}", e))); } }; if let Err(e) = atomic_write(&o.path, &c.replace(&o.old_text, &o.new_text)).await { let _ = self.rollback_snap(tot).await; return Err(ToolError::new(format!("Atomic write: {}", e))); } }
        self.cleanup().await; self.state = TransactionState::Committed; Ok(ToolOutput::success(format!("Edited {} files", tot)))
    }

    pub async fn rollback(&mut self) -> Result<ToolOutput, ToolError> { if self.state != TransactionState::Pending { return Err(ToolError::new("Not pending")); } self.rollback_snap(self.snapshots.len()).await }

    async fn rollback_snap(&self, n: usize) -> Result<ToolOutput, ToolError> { let mut r = 0; for s in self.snapshots.iter().take(n) { fs::write(&s.path, &s.content).await.map_err(|e| ToolError::new(format!("Rollback {}: {}", s.path.display(), e)))?; r += 1; } self.cleanup().await; Ok(ToolOutput::success(format!("Rolled back {} files", r))) }

    async fn cleanup(&self) { let _ = fs::remove_dir_all(&self.temp_dir).await; }
    pub fn state(&self) -> TransactionState { self.state }
}

impl Drop for MultiEditTransaction {
    fn drop(&mut self) { if self.state == TransactionState::Pending && !self.snapshots.is_empty() { let tp = self.temp_dir.clone(); let sp: Vec<_> = self.snapshots.iter().map(|s| (s.path.clone(), s.content.clone())).collect(); tokio::spawn(async move { let _ = fs::remove_dir_all(&tp).await; for (p, c) in sp { let _ = fs::write(&p, c).await; } }); } }
}

#[cfg(test)]
mod tests { 
    use super::*; 
    #[tokio::test] async fn dry_run() { 
        let mut p = EditPlan::new(); 
        p.add(EditOp::new("/t", "a", "b")); 
        assert!(MultiEditTransaction::new(p, true).dry_run); 
    } 
    #[tokio::test] async fn empty() -> Result<(), Box<dyn std::error::Error>> { 
        let result = MultiEditTransaction::new(EditPlan::new(), false).commit(|_, _, _| {}).await?;
        assert!(result.stdout.contains("No changes")); 
        Ok(())
    } 
    #[tokio::test] async fn circular() { 
        let mut p = EditPlan::new(); 
        p.add(EditOp::new("/a", "x", "y")); 
        p.add(EditOp::new("/b", "y", "x")); 
        assert!(MultiEditTransaction::new(p, false).detect_circular_deps().is_err()); 
    } 
}

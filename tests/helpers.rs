//! 共享测试工具，减少重复unwrap
//! 
//! 使用原则：测试代码允许`expect()`，但禁止裸`unwrap()`
//! expect提供失败上下文，便于调试

use std::path::Path;

/// 安全文件读取，测试失败时提供上下文
pub fn read_test_file<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read test file {}: {}", path.display(), e))
}

/// 安全环境变量获取
pub fn env_var(name: &str) -> String {
    std::env::var(name)
        .unwrap_or_else(|e| panic!("Env var {} not set: {}", name, e))
}

/// 断言Result为Ok，失败时打印错误
pub fn assert_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(v) => v,
        Err(e) => panic!("Expected Ok, got Err: {:?}", e),
    }
}

/// 安全获取临时文件路径
pub fn temp_path(temp: &tempfile::NamedTempFile) -> &str {
    temp.path()
        .to_str()
        .expect("setup: temp file path should be valid UTF-8")
}

/// 安全排序比较 - 用于f32排序
pub fn f32_cmp(a: &f32, b: &f32) -> std::cmp::Ordering {
    a.partial_cmp(b)
        .expect("data: f32 comparison should be valid (no NaN)")
}

/// 安全打开SQLite连接
pub fn open_sqlite(path: &str) -> rusqlite::Connection {
    rusqlite::Connection::open(path)
        .expect(&format!("setup: sqlite connection should open for {}", path))
}

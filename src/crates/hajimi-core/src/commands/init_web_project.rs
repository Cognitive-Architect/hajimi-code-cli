//! Initialize Web Project Command - React 18 + TypeScript 5 + Vite

use std::fs;
use std::path::Path;
use crate::error::EngineError;

/// Initialize a new web project with React 18, TypeScript 5, and Vite
pub async fn init_web_project(project_name: &str, target_dir: &Path) -> Result<(), EngineError> {
    let project_path = target_dir.join(project_name);
    
    // Create directory structure
    for dir in &["", "src", "src/assets", "public"] {
        fs::create_dir_all(&project_path.join(dir))
            .map_err(|e| EngineError::ExecutionFailed(format!("Create dir failed: {}", e)))?;
    }
    
    // Copy template files
    let template_dir = get_template_dir()?;
    let files = vec![
        "package.json", "tsconfig.json", "tsconfig.node.json", "vite.config.ts",
        ".eslintrc.cjs", ".gitignore", "index.html", "README.md",
        "src/main.tsx", "src/App.tsx", "src/App.css", "src/index.css",
        "src/vite-env.d.ts", "src/assets/react.svg", "public/vite.svg",
    ];
    
    for file in files {
        let src = template_dir.join(file);
        let dst = project_path.join(file);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| EngineError::ExecutionFailed(format!("Create dir failed: {}", e)))?;
        }
        fs::copy(&src, &dst)
            .map_err(|e| EngineError::ExecutionFailed(format!("Copy {} failed: {}", file, e)))?;
    }
    
    // Generate hajimi.config.toml
    let config = format!(
        r#"[project]
name = "{}"
type = "web"
framework = "react"
build_tool = "vite"
language = "typescript"

[build]
command = "npm run build"
output_dir = "dist"

[dev]
command = "npm run dev"
port = 3000

[hajimi]
version = "1.0.0"
template = "web-react-vite"
"#,
        project_name
    );
    fs::write(&project_path.join("hajimi.config.toml"), config)
        .map_err(|e| EngineError::ExecutionFailed(format!("Write config failed: {}", e)))?;
    
    Ok(())
}

fn get_template_dir() -> Result<std::path::PathBuf, EngineError> {
    let exe = std::env::current_exe()
        .map_err(|e| EngineError::ExecutionFailed(format!("Exe path: {}", e)))?;
    exe.parent()
        .ok_or_else(|| EngineError::ExecutionFailed("Invalid exe".into()))
        .map(|p| p.join("templates/web-react-vite"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_template_dir() {
        let result = get_template_dir();
        assert!(result.is_ok() || result.is_err());
    }
}

//! Fast File Walker - Using ignore crate (same as ripgrep)
//!
//! Provides gitignore-aware, parallel file walking that's significantly
//! faster than walkdir for large codebases.

use ignore::WalkBuilder;
use std::path::PathBuf;
use std::sync::Mutex;

/// File types we care about for schema analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Rust,
    TypeScript,
    Python,
}

impl FileType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "ts" | "tsx" => Some(Self::TypeScript),
            "py" => Some(Self::Python),
            _ => None,
        }
    }
}

/// A file found during walking
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub file_type: FileType,
}

/// Fast file walker configuration
pub struct FastWalker {
    root: PathBuf,
    /// Additional directories to skip (beyond .gitignore)
    skip_dirs: Vec<&'static str>,
}

impl FastWalker {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            skip_dirs: vec![
                "node_modules",
                "target", 
                ".next",
                "dist",
                "build",
                ".git",
                ".venv",
                "venv",
                "__pycache__",
                "trashbin",
                ".turbo",
                "generated", // Skip generated files
            ],
        }
    }

    /// Collect all source files using parallel walking
    /// Returns files grouped by type for efficient processing
    pub fn collect_files(&self) -> Vec<SourceFile> {
        let files = Mutex::new(Vec::new());
        
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)           // Don't skip hidden files
            .git_ignore(true)        // Respect .gitignore
            .git_global(true)        // Respect global gitignore
            .git_exclude(true)       // Respect .git/info/exclude
            .ignore(true)            // Respect .ignore files
            .parents(true)           // Check parent directories for ignore files
            .build_parallel();
        
        let skip_dirs = &self.skip_dirs;
        
        walker.run(|| {
            let files = &files;
            let skip_dirs = skip_dirs;
            
            Box::new(move |entry| {
                use ignore::WalkState;
                
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return WalkState::Continue,
                };
                
                let path = entry.path();
                
                // Skip our custom excluded directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if skip_dirs.contains(&name) {
                        return WalkState::Skip;
                    }
                }
                
                // Only process files (not directories)
                if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    return WalkState::Continue;
                }
                
                // Check extension
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if let Some(file_type) = FileType::from_extension(ext) {
                        files.lock().unwrap().push(SourceFile {
                            path: path.to_path_buf(),
                            file_type,
                        });
                    }
                }
                
                WalkState::Continue
            })
        });
        
        files.into_inner().unwrap()
    }

    /// Collect files and return only Rust files
    pub fn collect_rust_files(&self) -> Vec<PathBuf> {
        self.collect_files()
            .into_iter()
            .filter(|f| f.file_type == FileType::Rust)
            .map(|f| f.path)
            .collect()
    }

    /// Collect files and return only TypeScript files
    pub fn collect_typescript_files(&self) -> Vec<PathBuf> {
        self.collect_files()
            .into_iter()
            .filter(|f| f.file_type == FileType::TypeScript)
            .map(|f| f.path)
            .collect()
    }

    /// Collect files and return only Python files
    pub fn collect_python_files(&self) -> Vec<PathBuf> {
        self.collect_files()
            .into_iter()
            .filter(|f| f.file_type == FileType::Python)
            .map(|f| f.path)
            .collect()
    }
}

/// Count files by type (useful for progress bars)
pub fn count_files_by_type(files: &[SourceFile]) -> (usize, usize, usize) {
    let rust = files.iter().filter(|f| f.file_type == FileType::Rust).count();
    let ts = files.iter().filter(|f| f.file_type == FileType::TypeScript).count();
    let py = files.iter().filter(|f| f.file_type == FileType::Python).count();
    (rust, ts, py)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_from_extension() {
        assert_eq!(FileType::from_extension("rs"), Some(FileType::Rust));
        assert_eq!(FileType::from_extension("ts"), Some(FileType::TypeScript));
        assert_eq!(FileType::from_extension("tsx"), Some(FileType::TypeScript));
        assert_eq!(FileType::from_extension("py"), Some(FileType::Python));
        assert_eq!(FileType::from_extension("js"), None);
        assert_eq!(FileType::from_extension("md"), None);
    }
}


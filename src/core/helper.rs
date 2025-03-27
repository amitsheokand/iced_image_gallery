use std::path::PathBuf;

pub fn list_image_files(dir: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |ext| {
                    ["jpg", "jpeg", "png", "gif"].contains(&ext.to_str().unwrap_or("").to_lowercase().as_str())
                }) {
                    paths.push(path);
                }
            }
        }
    }
    
    paths
} 
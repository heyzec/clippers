use std::path::PathBuf;

pub fn get_clippers_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        // macOS: ~/Library/Application Support/clippers
        let home_dir = std::env::var("HOME")?;
        Ok(PathBuf::from(&home_dir)
            .join("Library")
            .join("Application Support")
            .join("clippers"))
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: $XDG_DATA_HOME/clippers or ~/.local/share/clippers
        let data_home = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home_dir = std::env::var("HOME").expect("HOME not set");
                PathBuf::from(&home_dir).join(".local").join("share")
            });
        Ok(data_home.join("clippers"))
    }
}

pub fn get_history_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(get_clippers_dir()?.join("history.json"))
}

pub fn get_blobs_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    use std::fs;
    let blobs_dir = get_clippers_dir()?.join("blobs");
    fs::create_dir_all(&blobs_dir)?;
    Ok(blobs_dir)
}

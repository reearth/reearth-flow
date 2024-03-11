use std::path::{Path, PathBuf};

use tokio;

pub async fn empty_dir<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let mut entries = tokio::fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            tokio::fs::remove_dir_all(entry.path()).await?
        } else {
            tokio::fs::remove_file(entry.path()).await?;
        }
    }
    Ok(())
}

/// Helper function to get the cache path.
pub fn get_cache_directory_path(data_dir_path: &Path) -> PathBuf {
    data_dir_path.join("indexer-split-cache").join("splits")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::Builder;

    #[tokio::test]
    async fn test_empty_dir() -> std::io::Result<()> {
        let temp_dir = Builder::new().prefix("foobar").tempdir_in(".").unwrap();
        let file_path = temp_dir.path().join("file");
        tokio::fs::File::create(file_path).await?;

        let subdir = temp_dir.path().join("subdir");
        tokio::fs::create_dir(&subdir).await?;

        let subfile_path = subdir.join("subfile");
        tokio::fs::File::create(subfile_path).await?;

        empty_dir(temp_dir.path()).await?;
        assert!(tokio::fs::read_dir(temp_dir.path())
            .await?
            .next_entry()
            .await?
            .is_none());
        Ok(())
    }
}

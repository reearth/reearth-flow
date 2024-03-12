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

async fn is_dir(entry: &tokio::fs::DirEntry) -> std::io::Result<bool> {
    let metadata = entry.metadata().await?;
    Ok(metadata.is_dir())
}

#[async_recursion::async_recursion]
async fn copy_tree_inner(src: PathBuf, dst: PathBuf) -> std::io::Result<()> {
    if !dst.exists() {
        tokio::fs::create_dir_all(&dst).await?;
    }

    let mut entries = tokio::fs::read_dir(src).await?;

    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if is_dir(&entry).await? {
            copy_tree_inner(src_path, dst_path).await?;
        } else {
            if dst_path.exists() {
                tokio::fs::remove_file(&dst_path).await?;
            }
            tokio::fs::copy(src_path, dst_path).await?;
        }
    }
    Ok(())
}

pub async fn copy_tree<P, Q>(src: P, dest: Q) -> std::io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    copy_tree_inner(src.as_ref().to_path_buf(), dest.as_ref().to_path_buf()).await
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

    #[tokio::test]
    async fn test_copy_tree() -> std::io::Result<()> {
        let temp_dir = Builder::new().prefix("foobar").tempdir_in(".").unwrap();
        let src_dir = temp_dir.path().join("src");
        let dest_dir = temp_dir.path().join("dest");

        tokio::fs::create_dir(&src_dir).await?;
        tokio::fs::create_dir(&dest_dir).await?;

        let file_path = src_dir.join("file");
        tokio::fs::File::create(&file_path).await?;

        let subdir = src_dir.join("subdir");
        tokio::fs::create_dir(&subdir).await?;

        let subfile_path = subdir.join("subfile");
        tokio::fs::File::create(&subfile_path).await?;

        copy_tree(&src_dir, &dest_dir).await?;

        let dest_file_path = dest_dir.join("file");
        assert!(tokio::fs::metadata(&dest_file_path).await.is_ok());

        let dest_subdir = dest_dir.join("subdir");
        assert!(tokio::fs::metadata(&dest_subdir).await.is_ok());

        let dest_subfile_path = dest_subdir.join("subfile");
        assert!(tokio::fs::metadata(&dest_subfile_path).await.is_ok());

        Ok(())
    }
}

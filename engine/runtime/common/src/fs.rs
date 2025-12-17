use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};

use async_zip::{tokio::write::ZipFileWriter, Compression, ZipDateTime, ZipEntryBuilder};
use tokio::{self, fs::File, io::AsyncReadExt};

pub struct Metadata {
    pub size: i64,
    pub atime: i64,
    pub mtime: i64,
    pub ctime: i64,
    pub is_dir: bool,
}

pub fn metadata<P: AsRef<Path>>(path: &P) -> std::io::Result<Metadata> {
    let metadata = std::fs::metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(Metadata {
            size: metadata.len() as i64,
            atime: metadata.atime(),
            mtime: metadata.mtime(),
            ctime: metadata.ctime(),
            is_dir: metadata.is_dir(),
        })
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        Ok(Metadata {
            size: metadata.file_size() as i64,
            atime: metadata.last_access_time() as i64,
            mtime: metadata.last_write_time() as i64,
            ctime: metadata.creation_time() as i64,
            is_dir: metadata.is_dir(),
        })
    }
}

/// Synchronously copies a directory tree from source to destination.
///
/// # Arguments
///
/// * `src` - Source directory path
/// * `dest` - Destination directory path
///
/// # Returns
///
/// Returns `std::io::Result<()>` indicating success or failure
pub fn copy_sync_tree<P, Q>(src: P, dest: Q) -> std::io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let src = src.as_ref();
    let dst = dest.as_ref();
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_sync_tree(&src_path, &dst_path)?;
        } else {
            if dst_path.exists() {
                std::fs::remove_file(&dst_path)?;
            }
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

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

pub async fn list_tree<P>(root: &P) -> std::io::Result<Vec<(PathBuf, tokio::fs::File)>>
where
    P: AsRef<Path>,
{
    list_tree_inner(root.as_ref().to_path_buf()).await
}

#[async_recursion::async_recursion]
async fn list_tree_inner(root: PathBuf) -> std::io::Result<Vec<(PathBuf, tokio::fs::File)>> {
    let mut result = Vec::new();
    let mut entries = tokio::fs::read_dir(root).await?;
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();

        if is_dir(&entry).await? {
            let children = list_tree_inner(entry_path).await?;
            result.extend(children);
        } else {
            let entry_file = tokio::fs::File::options()
                .read(true)
                .write(false)
                .create(false)
                .create_new(false)
                .truncate(false)
                .open(entry_path.clone())
                .await?;
            result.push((entry_path, entry_file));
        }
    }
    Ok(result)
}

pub async fn create_zip_file<P, Q>(input_root_path: P, output_path: Q) -> std::io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let mut entries = list_tree(&input_root_path).await?;
    let input_dir_str = input_root_path
        .as_ref()
        .to_str()
        .ok_or(std::io::Error::from(std::io::ErrorKind::InvalidData))?;
    let mut output = File::options()
        .create_new(true)
        .write(true)
        .read(true)
        .open(output_path)
        .await?;
    let mut output_writer = ZipFileWriter::with_tokio(&mut output);
    for (p, entry) in entries.iter_mut() {
        let entry_path = p.as_path();
        let entry_str = entry_path
            .as_os_str()
            .to_str()
            .ok_or(std::io::Error::from(std::io::ErrorKind::InvalidData))?;
        let buffer = read_file(entry).await?;
        let splits = input_dir_str
            .split(MAIN_SEPARATOR_STR)
            .collect::<Vec<&str>>();
        let filename = format!(
            "{}{}",
            splits[splits.len() - 1],
            &entry_str[input_dir_str.len()..]
        );
        let builder = ZipEntryBuilder::new(filename.into(), Compression::Deflate)
            .unix_permissions(0o644)
            .last_modification_date(ZipDateTime::from_chrono(&chrono::Utc::now()));
        output_writer
            .write_entry_whole(builder, &buffer)
            .await
            .map_err(|e| std::io::Error::other(format!("Failed to write zip entry: {e}")))?;
    }
    output_writer
        .close()
        .await
        .map_err(|e| std::io::Error::other(format!("Failed to close zip file: {e}")))?;
    Ok(())
}

pub fn get_dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            total += metadata.len();
        } else if metadata.is_dir() {
            total += get_dir_size(&entry.path())?;
        }
    }
    Ok(total)
}

pub async fn read_file(file: &mut File) -> std::io::Result<Vec<u8>> {
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
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

    #[test]
    fn test_copy_sync_tree() -> std::io::Result<()> {
        let temp_dir = Builder::new().prefix("foobar").tempdir_in(".")?;
        let src_dir = temp_dir.path().join("src");
        let dest_dir = temp_dir.path().join("dest");

        std::fs::create_dir(&src_dir)?;
        std::fs::create_dir(&dest_dir)?;

        let file_path = src_dir.join("file");
        std::fs::File::create(&file_path)?;

        let subdir = src_dir.join("subdir");
        std::fs::create_dir(&subdir)?;

        let subfile_path = subdir.join("subfile");
        std::fs::File::create(&subfile_path)?;

        copy_sync_tree(&src_dir, &dest_dir)?;

        let dest_file_path = dest_dir.join("file");
        assert!(std::fs::metadata(&dest_file_path).is_ok());

        let dest_subdir = dest_dir.join("subdir");
        assert!(std::fs::metadata(&dest_subdir).is_ok());

        let dest_subfile_path = dest_subdir.join("subfile");
        assert!(std::fs::metadata(&dest_subfile_path).is_ok());

        Ok(())
    }
}

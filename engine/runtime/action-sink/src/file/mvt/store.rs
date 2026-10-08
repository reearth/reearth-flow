//! Append-only byte records, spilled to a file in the executor cache once
//! [`SPILL_THRESHOLD`] bytes are buffered; only each record's byte range stays
//! in memory. Each reader opens its own handle, so any number of threads can
//! read records concurrently from a shared `&RecordStore`.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const SPILL_THRESHOLD: usize = 8 * 1024 * 1024;

pub(super) struct RecordStore {
    /// Created on first flush; removed on drop.
    path: PathBuf,
    /// Bytes already appended to the file.
    flushed: u64,
    /// Records not yet appended to the file, starting at `flushed`.
    tail: Vec<u8>,
    /// Offset and length of each record.
    entries: Vec<(u64, u32)>,
}

impl std::fmt::Debug for RecordStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordStore")
            .field("path", &self.path)
            .field("len", &self.entries.len())
            .finish_non_exhaustive()
    }
}

impl Drop for RecordStore {
    fn drop(&mut self) {
        if self.flushed > 0 {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

impl RecordStore {
    /// An empty store whose file will be a new uniquely named file in `dir`.
    pub(super) fn new(dir: &Path) -> Self {
        Self {
            path: dir.join(format!("{}.bin", uuid::Uuid::new_v4())),
            flushed: 0,
            tail: Vec::new(),
            entries: Vec::new(),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    /// Append `record`, returning its id: the number of records before it.
    pub(super) fn push(&mut self, record: &[u8]) -> io::Result<u32> {
        let id = self.entries.len() as u32;
        self.entries
            .push((self.flushed + self.tail.len() as u64, record.len() as u32));
        self.tail.extend_from_slice(record);
        if self.tail.len() >= SPILL_THRESHOLD {
            self.flush()?;
        }
        Ok(id)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.flushed == 0 {
            if let Some(dir) = self.path.parent() {
                std::fs::create_dir_all(dir)?;
            }
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?
            .write_all(&self.tail)?;
        self.flushed += self.tail.len() as u64;
        self.tail.clear();
        Ok(())
    }

    pub(super) fn reader(&self) -> io::Result<RecordReader<'_>> {
        let file = match self.flushed {
            0 => None,
            _ => Some(File::open(&self.path)?),
        };
        Ok(RecordReader {
            store: self,
            file,
            buf: Vec::new(),
        })
    }
}

/// One thread's read handle on a [`RecordStore`].
pub(super) struct RecordReader<'a> {
    store: &'a RecordStore,
    /// `None` while every record is still in the store's tail.
    file: Option<File>,
    buf: Vec<u8>,
}

impl RecordReader<'_> {
    pub(super) fn read(&mut self, id: u32) -> io::Result<&[u8]> {
        let store = self.store;
        let (offset, len) = store.entries[id as usize];
        let len = len as usize;
        if offset >= store.flushed {
            let start = (offset - store.flushed) as usize;
            return Ok(&store.tail[start..start + len]);
        }
        let file = self
            .file
            .as_mut()
            .expect("a record below `flushed` was written to the file");
        self.buf.resize(len, 0);
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut self.buf)?;
        Ok(&self.buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Records read back unchanged whether they sit in the file or the tail,
    // and the file is removed with the store.
    #[test]
    fn records_round_trip_from_file_and_tail() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = RecordStore::new(dir.path());
        let records: Vec<Vec<u8>> = (0..5u8).map(|i| vec![i; i as usize * 3]).collect();
        for (i, record) in records.iter().enumerate() {
            assert_eq!(store.push(record).unwrap(), i as u32);
            if i == 2 {
                store.flush().unwrap();
            }
        }
        assert!(store.flushed > 0 && !store.tail.is_empty());

        let mut reader = store.reader().unwrap();
        for (i, record) in records.iter().enumerate() {
            assert_eq!(reader.read(i as u32).unwrap(), record.as_slice());
        }
        drop(reader);
        drop(store);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}

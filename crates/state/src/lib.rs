//! # Example
//!
//! ```rust,no_run,ignore
//! use serde::{Serialize,Deserialize};
//!
//! #[derive(Serialize,Deserialize)]
//! struct Foo {
//!     foo: String
//! }
//!
//! let db = State::new("data").unwrap();
//! let f = Foo { foo: "bar".to_owned() };
//! let id = db.save(&f).unwrap();
//! let obj = db.get::<Foo>(&id).unwrap();
//! db.delete(&id).unwrap();
//! ```
//!
//! Creating a state instance that is living in the memory can be done like this:
//!
//! ```rust,no_run,ignore
//! let db = State::new(IN_MEMORY).unwrap();
//! ```

mod file_state;
mod json_state;
mod memory_state;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::Result,
    path::{Path, PathBuf},
    sync::Arc,
};

use file_state::FileState;
use json_state::JsonState;
use memory_state::MemoryState;

#[derive(Debug, Clone)]
pub struct State(StateType);

#[derive(Debug, Clone)]
enum StateType {
    File(Arc<RwLock<FileState>>, PathBuf),
    Memory(MemoryState),
}

pub const IN_MEMORY: &str = "::memory::";

impl State {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref() == Path::new(IN_MEMORY) {
            Ok(Self(StateType::Memory(MemoryState::default())))
        } else {
            let s = FileState::new(path)?;
            let p = s.path().to_path_buf();
            Ok(Self(StateType::File(Arc::new(RwLock::new(s)), p)))
        }
    }

    /// Returns the storage path for the backing JSON state.
    pub fn path(&self) -> &Path {
        match &self.0 {
            StateType::File(_, p) => p,
            StateType::Memory(_) => Path::new(IN_MEMORY),
        }
    }

    pub fn save<T>(&self, obj: &T) -> Result<String>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        match &self.0 {
            StateType::File(f, _) => f.write().save(obj),
            StateType::Memory(m) => m.save(obj),
        }
    }

    pub fn save_with_id<T>(&self, obj: &T, id: &str) -> Result<String>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        match &self.0 {
            StateType::File(f, _) => f.write().save_with_id(obj, id),
            StateType::Memory(m) => m.save_with_id(obj, id),
        }
    }

    pub fn get<T>(&self, id: &str) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        match &self.0 {
            StateType::File(f, _) => f.read().get(id),
            StateType::Memory(m) => m.get(id),
        }
    }

    pub fn all<T>(&self) -> Result<BTreeMap<String, T>>
    where
        for<'de> T: Deserialize<'de>,
    {
        match &self.0 {
            StateType::File(f, _) => f.read().all(),
            StateType::Memory(m) => m.all(),
        }
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        match &self.0 {
            StateType::File(f, _) => f.write().delete(id),
            StateType::Memory(m) => m.delete(id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::{Deserialize, Serialize};
    use std::thread;
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize)]
    struct Data {
        x: i32,
    }

    fn multi_threaded_write(state: State) {
        let mut threads: Vec<thread::JoinHandle<()>> = vec![];
        for i in 0..20 {
            let db = state.clone();
            let x = Data { x: i };
            threads.push(thread::spawn(move || {
                db.save_with_id(&x, &i.to_string()).unwrap();
            }));
        }
        for t in threads {
            t.join().unwrap();
        }
        let all = state.all::<Data>().unwrap();
        assert_eq!(all.len(), 20);
        for (id, data) in all {
            assert_eq!(data.x.to_string(), id);
        }
    }

    #[test]
    fn multi_threaded_write_with_dir() {
        #[derive(Serialize, Deserialize)]
        struct Data {
            x: i32,
        }
        let dir = tempdir().expect("Could not create temporary directory");
        let state = State::new(dir.path()).unwrap();
        multi_threaded_write(state);
    }

    #[test]
    fn multi_threaded_write_in_memory() {
        #[derive(Serialize, Deserialize)]
        struct Data {
            x: i32,
        }
        let state = State::new(IN_MEMORY).unwrap();
        multi_threaded_write(state);
    }
}

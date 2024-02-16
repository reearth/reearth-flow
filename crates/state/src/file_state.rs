use std::{
    collections::BTreeMap,
    fs::{create_dir_all, metadata, read_dir, remove_file, rename, OpenOptions},
    io::{prelude::*, Error, ErrorKind, Result, Write},
    path::{Path, PathBuf},
};

use fs4::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::json_state::JsonState;

#[derive(Debug, Clone)]
pub struct FileState {
    path: PathBuf,
}

impl JsonState for FileState {
    fn save<T>(&self, obj: &T) -> Result<String>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        self.save_with_id(obj, &Uuid::new_v4().to_string())
    }

    fn save_with_id<T>(&self, obj: &T, id: &str) -> Result<String>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        self.save_object_to_file(obj, &self.id_to_path(id))?;
        Ok(id.to_owned())
    }

    fn get<T>(&self, id: &str) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        let json = get_json_from_file(&self.id_to_path(id))?;
        decode(json)
    }

    fn all<T>(&self) -> Result<BTreeMap<String, T>>
    where
        for<'de> T: Deserialize<'de>,
    {
        if !metadata(&self.path)?.is_dir() {
            return Err(Error::new(ErrorKind::NotFound, "invalid path"));
        }

        let entries = read_dir(&self.path)?
            .filter_map(|e| {
                e.and_then(|x| {
                    x.metadata().and_then(|m| {
                        if m.is_file() {
                            path_buf_to_id(&x.path())
                        } else {
                            Err(Error::new(ErrorKind::Other, "not a file"))
                        }
                    })
                })
                .ok()
            })
            .filter_map(|id| {
                self.get(&id)
                    .map_or_else(|_| None, |x| Some((id.clone(), x)))
            })
            .collect::<BTreeMap<String, T>>();

        Ok(entries)
    }

    fn delete(&self, id: &str) -> Result<()> {
        remove_file(self.id_to_path(id))
    }
}

impl FileState {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::new_with_cfg(path)
    }

    fn id_to_path(&self, id: &str) -> PathBuf {
        self.path.join(id).with_extension("json")
    }

    fn object_to_string<T: Serialize>(&self, obj: &T) -> Result<String> {
        serde_json::to_string(obj).map_err(|err| Error::new(ErrorKind::Other, err))
    }

    fn save_object_to_file<T: Serialize>(&self, obj: &T, file_name: &Path) -> Result<()> {
        let json_string = self.object_to_string(obj)?;
        let mut tmp_filename = file_name.to_path_buf();
        tmp_filename.set_file_name(&Uuid::new_v4().to_string());
        tmp_filename.set_extension("tmp");
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(file_name)?;
        let mut tmp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_filename)?;
        file.lock_exclusive()?;
        tmp_file.lock_exclusive()?;

        if let Err(err) = Write::write_all(&mut tmp_file, json_string.as_bytes()) {
            Err(err)
        } else {
            tmp_file.unlock()?;
            file.unlock()?;
            drop(file);
            drop(tmp_file);
            rename(tmp_filename, file_name)
        }
    }

    pub fn new_with_cfg<P: AsRef<Path>>(path: P) -> Result<Self> {
        let s = Self {
            path: path.as_ref().to_path_buf(), // TODO: probably change this to take an owned PathBuf parameter
        };

        if let Err(err) = create_dir_all(&s.path) {
            if err.kind() != ErrorKind::AlreadyExists {
                return Err(err);
            }
        }
        Ok(s)
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn decode<T>(o: Value) -> Result<T>
where
    for<'de> T: Deserialize<'de>,
{
    serde_json::from_value(o).map_err(|err| Error::new(ErrorKind::Other, err))
}

fn get_string_from_file(file_name: &Path) -> Result<String> {
    let mut f = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(file_name)?;
    let mut buffer = String::new();
    f.lock_shared()?;
    f.read_to_string(&mut buffer)?;
    f.unlock()?;
    Ok(buffer)
}

fn get_json_from_file(file_name: &Path) -> Result<Value> {
    let s = get_string_from_file(file_name)?;
    serde_json::from_str(&s).map_err(|err| Error::new(ErrorKind::Other, err))
}

fn path_buf_to_id(p: &Path) -> Result<String> {
    p.file_stem()
        .and_then(|n| n.to_os_string().into_string().ok())
        .ok_or_else(|| Error::new(ErrorKind::Other, "invalid id"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::{Deserialize, Serialize};
    use std::{collections::BTreeMap, fs::File, io::ErrorKind, path::Path, thread};
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize)]
    struct X {
        x: u32,
    }

    #[derive(Serialize, Deserialize)]
    struct Y {
        y: i32,
    }

    #[derive(Serialize, Deserialize)]
    struct Empty {}

    #[derive(Serialize, Deserialize)]
    struct Z {
        z: f32,
    }

    #[test]
    fn new_multi_threaded() {
        let mut threads: Vec<thread::JoinHandle<()>> = vec![];
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        for _ in 0..20 {
            let d = path.clone();
            threads.push(thread::spawn(move || {
                assert!(FileState::new(d).is_ok());
            }));
        }
        for c in threads {
            c.join().unwrap();
        }
    }

    #[test]
    fn save() {
        let dir = tempdir().unwrap();
        let db = FileState::new(&dir).unwrap();
        let data = X { x: 56 };
        let id = db.save(&data).unwrap();
        let mut f = File::open(dir.path().join(id).with_extension("json")).unwrap();
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).unwrap();
        assert_eq!(buffer, "{\"x\":56}");
    }

    #[test]
    fn save_and_read_multi_threaded() {
        let dir = tempdir().unwrap().path().to_path_buf();
        let db = FileState::new(&dir).unwrap();
        let mut threads: Vec<thread::JoinHandle<()>> = vec![];
        let x = X { x: 56 };
        db.save_with_id(&x, "bla").unwrap();
        for i in 0..20 {
            let d = dir.clone();
            let x = X { x: i };
            threads.push(thread::spawn(move || {
                let db = FileState::new(&d).unwrap();
                db.save_with_id(&x, "bla").unwrap();
            }));
        }
        for _ in 0..20 {
            let d = dir.clone();
            threads.push(thread::spawn(move || {
                let db = FileState::new(d).unwrap();
                db.get::<X>("bla").unwrap();
            }));
        }
        for c in threads {
            c.join().unwrap();
        }
    }

    #[test]
    fn save_empty_obj() {
        let dir = tempdir().unwrap().path().to_path_buf();
        let db = FileState::new(&dir).unwrap();
        let id = db.save(&Empty {}).unwrap();
        let mut f = File::open(dir.join(id).with_extension("json")).unwrap();
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).unwrap();
        assert_eq!(buffer, "{}");
    }

    #[test]
    fn save_with_id() {
        let dir = tempdir().unwrap().path().to_path_buf();
        let db = FileState::new(&dir).unwrap();
        let data = Y { y: -7 };
        db.save_with_id(&data, "foo").unwrap();
        let mut f = File::open(dir.join("foo.json")).unwrap();
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).unwrap();
        assert_eq!(buffer, "{\"y\":-7}");
    }

    #[test]
    fn get() {
        let dir = tempdir().unwrap().path().to_path_buf();
        let db = FileState::new(&dir).unwrap();
        let mut file = File::create(dir.join("foo.json")).unwrap();
        Write::write_all(&mut file, b"{\"z\":9.9}").unwrap();
        let obj: Z = db.get("foo").unwrap();
        assert_eq!(obj.z, 9.9);
    }

    #[test]
    fn get_non_existent() {
        let dir = tempdir().unwrap().path().to_path_buf();
        let db = FileState::new(dir).unwrap();
        let res = db.get::<X>("foobarobject");
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().kind(), ErrorKind::NotFound);
    }

    #[test]
    fn all() {
        let dir = tempdir().unwrap().path().to_path_buf();
        let db = FileState::new(&dir).unwrap();

        #[cfg(feature = "serde_json")]
        #[derive(Deserialize, Serialize)]
        struct X {
            x: u32,
            y: u32,
        }

        let mut file = File::create(dir.join("foo.json")).unwrap();
        Write::write_all(&mut file, b"{\"x\":1, \"y\":0}").unwrap();

        let mut file = File::create(dir.join("bar.json")).unwrap();
        Write::write_all(&mut file, b"{\"y\":2}").unwrap();

        let all_x: BTreeMap<String, X> = db.all().unwrap();
        let all_y: BTreeMap<String, Y> = db.all().unwrap();
        assert_eq!(all_x.get("foo").unwrap().x, 1);
        assert!(all_x.get("bar").is_none());
        assert_eq!(all_y.get("bar").unwrap().y, 2);
    }

    #[test]
    fn delete() {
        let dir = tempdir().unwrap();
        let db = FileState::new(&dir).unwrap();
        let data = Y { y: 88 };
        let id = db.save(&data).unwrap();
        let f_name = dir.path().join(&id).with_extension("json");
        db.get::<Y>(&id).unwrap();
        assert!(Path::new(&f_name).exists());
        db.delete(&id).unwrap();
        assert!(!Path::new(&f_name).exists());
        assert!(db.get::<Y>(&id).is_err());
        assert!(db.delete(&id).is_err());
    }

    #[test]
    fn delete_non_existent() {
        let dir = tempdir().unwrap().path().to_path_buf();
        let db = FileState::new(dir).unwrap();
        let res = db.delete("blabla");
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().kind(), ErrorKind::NotFound);
    }
}

use std::sync::{Arc, Mutex};

pub struct Count {
    pub count: Arc<Mutex<usize>>,
}

impl Count {
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn increment(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
    }

    pub fn decrement(&self) {
        let mut count = self.count.lock().unwrap();
        *count -= 1;
    }

    pub fn get(&self) -> usize {
        let count = self.count.lock().unwrap();
        *count
    }
}

impl Default for Count {
    fn default() -> Self {
        Self::new()
    }
}

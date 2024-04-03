use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct ArcRWLock<T: Sized> {
    inner: Arc<RwLock<T>>,
}

#[derive(Debug)]
pub struct WeakRwLock<T: Sized> {
    inner: Weak<RwLock<T>>,
}

impl<T> Clone for ArcRWLock<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> PartialEq for ArcRWLock<T> {
    fn eq(&self, other: &Self) -> bool {
        // RefNodes are equal if the two Rc point to the same RefCell.
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T> ArcRWLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    pub fn as_inner(&self) -> &Arc<RwLock<T>> {
        &self.inner
    }

    pub fn unwrap(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.inner.read().unwrap()
    }

    pub fn downgrade(self) -> WeakRwLock<T> {
        WeakRwLock {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn borrow(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.inner.read().unwrap()
    }

    pub fn borrow_mut(&self) -> std::sync::RwLockWriteGuard<'_, T> {
        self.inner.write().unwrap()
    }
}

impl<T> Clone for WeakRwLock<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> WeakRwLock<T> {
    pub fn as_inner(&self) -> &Weak<RwLock<T>> {
        &self.inner
    }

    pub fn upgrade(self) -> Option<ArcRWLock<T>> {
        self.inner.upgrade().map(|inner| ArcRWLock { inner })
    }
}

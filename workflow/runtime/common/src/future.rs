use std::boxed::Box;
use std::clone::Clone;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

pub struct SharedFuture {
    inner: Arc<Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>>,
}

impl SharedFuture {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Self {
            inner: Arc::new(Mutex::new(Box::pin(future))),
        }
    }
}

impl Clone for SharedFuture {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Debug for SharedFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedFuture").finish()
    }
}

impl Future for SharedFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.lock().unwrap();
        inner.as_mut().poll(cx)
    }
}

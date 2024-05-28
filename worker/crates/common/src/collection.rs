use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use futures::Future;
use rayon::prelude::*;

pub fn filter<T, P>(collection: &[T], predict: P) -> Vec<T>
where
    T: Send + Sync + Clone,
    P: Fn(&T) -> bool + Send + Sync,
{
    match collection.len() {
        0..=1000 => collection
            .iter()
            .filter(|&row| predict(row))
            .cloned()
            .collect::<Vec<_>>(),
        _ => collection
            .par_iter()
            .filter(|&row| predict(row))
            .cloned()
            .collect::<Vec<_>>(),
    }
}

pub fn map<T, P, R>(collection: &[T], predict: P) -> Vec<R>
where
    T: Send + Sync + Clone,
    P: Fn(&T) -> R + Send + Sync,
    R: Send + Sync,
{
    match collection.len() {
        0..=1000 => collection.iter().map(predict).collect::<Vec<_>>(),
        _ => collection.par_iter().map(predict).collect::<Vec<_>>(),
    }
}

pub fn par_map<T, P, R>(collection: &[T], predict: P) -> Vec<R>
where
    T: Send + Sync + Clone,
    P: Fn(&T) -> R + Send + Sync,
    R: Send + Sync,
{
    collection.par_iter().map(predict).collect::<Vec<_>>()
}
pub fn par_flat_map<T, F, PI>(collection: &[T], predict: F) -> Vec<PI::Item>
where
    T: Send + Sync + Clone,
    F: Fn(&T) -> PI + Send + Sync,
    PI: IntoParallelIterator,
{
    collection.par_iter().flat_map(predict).collect::<Vec<_>>()
}

pub async fn join_parallel<T: Send + 'static>(
    futs: impl IntoIterator<Item = impl Future<Output = T> + Send + 'static>,
) -> Vec<T> {
    let tasks: Vec<_> = futs.into_iter().map(tokio::spawn).collect();
    futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(Result::unwrap)
        .collect()
}

pub fn vec_to_map<T, P, R>(collection: &[T], predict: P) -> HashMap<String, R>
where
    T: Send + Sync + Clone,
    P: Fn(&T) -> (String, R) + Send + Sync,
    R: Send + Sync,
{
    match collection.len() {
        0..=1000 => collection
            .iter()
            .map(predict)
            .collect::<HashMap<String, R>>(),
        _ => collection
            .par_iter()
            .map(predict)
            .collect::<HashMap<String, R>>(),
    }
}

pub fn insert_vec_element<K, V>(map: &mut HashMap<K, Vec<V>>, key: K, value: V)
where
    K: Eq + Hash,
{
    match map.entry(key) {
        Entry::Occupied(mut entry) => {
            entry.get_mut().push(value);
        }
        Entry::Vacant(entry) => {
            entry.insert(vec![value]);
        }
    }
}

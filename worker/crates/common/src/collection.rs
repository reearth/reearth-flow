use std::collections::HashMap;

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

//! DataSize trait for estimating heap memory usage.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// A trait for estimating the heap memory usage of a type.
///
/// This trait provides a simple way to estimate how much heap memory
/// a value is using, which is useful for memory profiling and analysis.
///
/// # Example
///
/// ```ignore
/// use reearth_flow_analyzer_core::DataSize;
///
/// struct MyType {
///     data: Vec<u8>,
/// }
///
/// impl DataSize for MyType {
///     fn data_size(&self) -> usize {
///         self.data.data_size()
///     }
/// }
/// ```
pub trait DataSize {
    /// Estimates the heap memory usage of this value in bytes.
    ///
    /// This should return the approximate number of bytes allocated on the heap,
    /// not counting the stack size of the value itself.
    fn data_size(&self) -> usize;
}

// Primitive types - no heap allocation
macro_rules! impl_datasize_primitive {
    ($($ty:ty),*) => {
        $(
            impl DataSize for $ty {
                #[inline]
                fn data_size(&self) -> usize {
                    0
                }
            }
        )*
    };
}

impl_datasize_primitive!(
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    bool,
    char,
    ()
);

// String
impl DataSize for String {
    #[inline]
    fn data_size(&self) -> usize {
        self.capacity()
    }
}

// &str (borrowed, no heap ownership)
impl DataSize for &str {
    #[inline]
    fn data_size(&self) -> usize {
        0
    }
}

// Vec<T>
impl<T: DataSize> DataSize for Vec<T> {
    fn data_size(&self) -> usize {
        let element_size = std::mem::size_of::<T>();
        let vec_heap = self.capacity() * element_size;
        vec_heap + self.iter().map(|item| item.data_size()).sum::<usize>()
    }
}

// VecDeque<T>
impl<T: DataSize> DataSize for VecDeque<T> {
    fn data_size(&self) -> usize {
        let element_size = std::mem::size_of::<T>();
        let deque_heap = self.capacity() * element_size;
        deque_heap + self.iter().map(|item| item.data_size()).sum::<usize>()
    }
}

// Box<T>
impl<T: DataSize> DataSize for Box<T> {
    fn data_size(&self) -> usize {
        std::mem::size_of::<T>() + (**self).data_size()
    }
}

// Arc<T>
impl<T: DataSize> DataSize for Arc<T> {
    fn data_size(&self) -> usize {
        // Arc has overhead for the reference counts
        let arc_overhead = std::mem::size_of::<usize>() * 2;
        std::mem::size_of::<T>() + arc_overhead + (**self).data_size()
    }
}

// Option<T>
impl<T: DataSize> DataSize for Option<T> {
    fn data_size(&self) -> usize {
        match self {
            Some(value) => value.data_size(),
            None => 0,
        }
    }
}

// Result<T, E>
impl<T: DataSize, E: DataSize> DataSize for Result<T, E> {
    fn data_size(&self) -> usize {
        match self {
            Ok(value) => value.data_size(),
            Err(err) => err.data_size(),
        }
    }
}

// HashMap<K, V>
impl<K: DataSize, V: DataSize, S> DataSize for HashMap<K, V, S> {
    fn data_size(&self) -> usize {
        let entry_size = std::mem::size_of::<K>() + std::mem::size_of::<V>();
        // Estimate capacity as roughly 1.5x the length for load factor
        let capacity = self.capacity();
        let base_heap = capacity * entry_size;

        let key_heap: usize = self.keys().map(|k| k.data_size()).sum();

        let value_heap: usize = self.values().map(|v| v.data_size()).sum();

        base_heap + key_heap + value_heap
    }
}

// HashSet<T>
impl<T: DataSize, S> DataSize for HashSet<T, S> {
    fn data_size(&self) -> usize {
        let element_size = std::mem::size_of::<T>();
        let capacity = self.capacity();
        let base_heap = capacity * element_size;

        let element_heap: usize = self.iter().map(|item| item.data_size()).sum();

        base_heap + element_heap
    }
}

// BTreeMap<K, V>
impl<K: DataSize, V: DataSize> DataSize for BTreeMap<K, V> {
    fn data_size(&self) -> usize {
        // BTreeMap nodes have overhead; estimate conservatively
        let entry_size = std::mem::size_of::<K>() + std::mem::size_of::<V>();
        let node_overhead = 64; // Approximate B-tree node overhead
        let base_heap = self.len() * (entry_size + node_overhead);

        let key_heap: usize = self.keys().map(|k| k.data_size()).sum();

        let value_heap: usize = self.values().map(|v| v.data_size()).sum();

        base_heap + key_heap + value_heap
    }
}

// BTreeSet<T>
impl<T: DataSize> DataSize for BTreeSet<T> {
    fn data_size(&self) -> usize {
        let element_size = std::mem::size_of::<T>();
        let node_overhead = 64;
        let base_heap = self.len() * (element_size + node_overhead);

        let element_heap: usize = self.iter().map(|item| item.data_size()).sum();

        base_heap + element_heap
    }
}

// Arrays
impl<T: DataSize, const N: usize> DataSize for [T; N] {
    fn data_size(&self) -> usize {
        self.iter().map(|item| item.data_size()).sum()
    }
}

// Slices (borrowed, no heap ownership)
impl<T: DataSize> DataSize for &[T] {
    #[inline]
    fn data_size(&self) -> usize {
        0
    }
}

// Tuples
macro_rules! impl_datasize_tuple {
    ($($name:ident),*) => {
        impl<$($name: DataSize),*> DataSize for ($($name,)*) {
            #[allow(non_snake_case)]
            fn data_size(&self) -> usize {
                let ($($name,)*) = self;
                0 $(+ $name.data_size())*
            }
        }
    };
}

impl_datasize_tuple!(A);
impl_datasize_tuple!(A, B);
impl_datasize_tuple!(A, B, C);
impl_datasize_tuple!(A, B, C, D);
impl_datasize_tuple!(A, B, C, D, E);
impl_datasize_tuple!(A, B, C, D, E, F);
impl_datasize_tuple!(A, B, C, D, E, F, G);
impl_datasize_tuple!(A, B, C, D, E, F, G, H);

// serde_json::Value - estimate size by serializing
impl DataSize for serde_json::Value {
    fn data_size(&self) -> usize {
        // Estimate by serializing to string
        serde_json::to_string(self).map(|s| s.len()).unwrap_or(0)
    }
}

// uuid::Uuid - no heap allocation (stored on stack)
impl DataSize for uuid::Uuid {
    #[inline]
    fn data_size(&self) -> usize {
        0
    }
}

// PathBuf
impl DataSize for std::path::PathBuf {
    fn data_size(&self) -> usize {
        self.as_os_str().len()
    }
}

// Cow<'_, str>
impl<'a> DataSize for std::borrow::Cow<'a, str> {
    fn data_size(&self) -> usize {
        match self {
            std::borrow::Cow::Borrowed(_) => 0,
            std::borrow::Cow::Owned(s) => s.capacity(),
        }
    }
}

// bytes::Bytes - reference counted bytes
impl DataSize for bytes::Bytes {
    fn data_size(&self) -> usize {
        self.len()
    }
}

// serde_json::Number
impl DataSize for serde_json::Number {
    fn data_size(&self) -> usize {
        0 // Numbers are typically stack-allocated
    }
}

// chrono types
impl<Tz: chrono::TimeZone> DataSize for chrono::DateTime<Tz> {
    fn data_size(&self) -> usize {
        0 // DateTime is stack-allocated
    }
}

// url::Url - URLs contain heap-allocated strings
impl DataSize for url::Url {
    fn data_size(&self) -> usize {
        self.as_str().len()
    }
}

// nusamai_citygml::Color - typically small stack-allocated struct
impl DataSize for nusamai_citygml::Color {
    fn data_size(&self) -> usize {
        0 // Stack-allocated RGB values
    }
}

// flatgeom::MultiPolygon - borrowed geometry data
impl<'a, const N: usize> DataSize for flatgeom::MultiPolygon<'a, [f64; N]> {
    fn data_size(&self) -> usize {
        // This is a borrowed view over existing data, no heap ownership
        0
    }
}

// parking_lot::RwLock<T>
impl<T: DataSize> DataSize for parking_lot::RwLock<T> {
    fn data_size(&self) -> usize {
        self.read().data_size()
    }
}

// parking_lot::Mutex<T>
impl<T: DataSize> DataSize for parking_lot::Mutex<T> {
    fn data_size(&self) -> usize {
        self.lock().data_size()
    }
}

// rhai::AST - Abstract Syntax Tree
impl DataSize for rhai::AST {
    fn data_size(&self) -> usize {
        // AST is a complex structure with parsed scripts
        // Estimate based on source code if available, otherwise use a conservative estimate
        0 // ASTs are relatively small compared to feature data
    }
}

// std::time::Duration - stack-allocated
impl DataSize for std::time::Duration {
    fn data_size(&self) -> usize {
        0 // Duration is stack-allocated
    }
}

// std::sync::Mutex<T>
impl<T: DataSize> DataSize for std::sync::Mutex<T> {
    fn data_size(&self) -> usize {
        // Best effort - if poisoned, return 0
        self.lock().map(|guard| guard.data_size()).unwrap_or(0)
    }
}

// regex::Regex - compiled regular expression
impl DataSize for regex::Regex {
    fn data_size(&self) -> usize {
        // Regex is a compiled pattern, estimate conservatively
        self.as_str().len()
    }
}

// IndexMap<K, V>
impl<K: DataSize, V: DataSize, S> DataSize for indexmap::IndexMap<K, V, S> {
    fn data_size(&self) -> usize {
        let entry_size = std::mem::size_of::<K>() + std::mem::size_of::<V>();
        let capacity = self.capacity();
        let base_heap = capacity * entry_size;

        let key_heap: usize = self.keys().map(|k| k.data_size()).sum();
        let value_heap: usize = self.values().map(|v| v.data_size()).sum();

        base_heap + key_heap + value_heap
    }
}

// nusamai_citygml::schema types
impl DataSize for nusamai_citygml::schema::Schema {
    fn data_size(&self) -> usize {
        self.types.data_size()
    }
}

impl DataSize for nusamai_citygml::schema::TypeDef {
    fn data_size(&self) -> usize {
        match self {
            nusamai_citygml::schema::TypeDef::Feature(ft) => ft.data_size(),
            nusamai_citygml::schema::TypeDef::Data(dt) => dt.data_size(),
            nusamai_citygml::schema::TypeDef::Property(pt) => pt.data_size(),
        }
    }
}

impl DataSize for nusamai_citygml::schema::FeatureTypeDef {
    fn data_size(&self) -> usize {
        self.attributes.data_size()
    }
}

impl DataSize for nusamai_citygml::schema::DataTypeDef {
    fn data_size(&self) -> usize {
        self.attributes.data_size()
    }
}

impl DataSize for nusamai_citygml::schema::PropertyTypeDef {
    fn data_size(&self) -> usize {
        self.members.data_size()
    }
}

impl DataSize for nusamai_citygml::schema::Attribute {
    fn data_size(&self) -> usize {
        self.type_ref.data_size() + self.original_name.data_size()
    }
}

impl DataSize for nusamai_citygml::schema::TypeRef {
    fn data_size(&self) -> usize {
        match self {
            nusamai_citygml::schema::TypeRef::JsonString(attr) => attr.data_size(),
            nusamai_citygml::schema::TypeRef::Named(s) => s.data_size(),
            // All other variants are unit variants with no heap allocation
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_sizes() {
        assert_eq!(0u8.data_size(), 0);
        assert_eq!(42i32.data_size(), 0);
        assert_eq!(3.0f64.data_size(), 0);
        assert_eq!(true.data_size(), 0);
    }

    #[test]
    fn test_string_size() {
        let s = String::from("hello");
        assert!(s.data_size() >= 5);
    }

    #[test]
    fn test_vec_size() {
        let v: Vec<i32> = vec![1, 2, 3, 4, 5];
        let size = v.data_size();
        assert!(size >= 5 * std::mem::size_of::<i32>());
    }

    #[test]
    fn test_nested_vec_size() {
        let v: Vec<String> = vec![String::from("a"), String::from("bb"), String::from("ccc")];
        let size = v.data_size();
        // Should include both Vec allocation and String allocations
        assert!(size >= 3 * std::mem::size_of::<String>() + 6);
    }

    #[test]
    fn test_option_size() {
        let some: Option<String> = Some(String::from("hello"));
        let none: Option<String> = None;

        assert!(some.data_size() >= 5);
        assert_eq!(none.data_size(), 0);
    }

    #[test]
    fn test_hashmap_size() {
        let mut map: HashMap<String, i32> = HashMap::new();
        map.insert(String::from("key1"), 1);
        map.insert(String::from("key2"), 2);

        let size = map.data_size();
        assert!(size > 0);
    }
}

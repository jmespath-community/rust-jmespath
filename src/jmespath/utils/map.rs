#[cfg(not(feature = "preserve_order"))]
/// A type that represents an abstraction over an associative array.
///
/// This maps to [`::std::collections::btree_map::BTreeMap<K, V>`].
///
/// By default, this does not preserve the insertion order of keys.
/// Please, enable the "preserve_order" feature when compiling this
/// crate to opt in to [`::indexmap::IndexMap<K, V>`] instead.
///
/// ```toml
/// [dependencies]
/// jmespath_community = { version = "1.0", features = ["preserve_order"] }
/// ```
///
pub type Map<K, V> = ::std::collections::btree_map::BTreeMap<K, V>;
#[cfg(feature = "preserve_order")]
/// A type that represents an abstraction over an associative array.
///
/// This maps to [`::indexmap::IndexMap<K, V>`] that preserves the
/// insertion order of keys.
///
/// Please, disable the "preserve_order" feature when compiling
/// this crate to opt out to [`::std::collections::btree_map::BTreeMap<K, V>`]
/// instead.
///
/// ```toml
/// [dependencies]
/// jmespath_community = { version = "1.0", features = [] }
/// ```
pub type Map<K, V> = ::indexmap::IndexMap<K, V>;

/// Creates a [`Map`] from a list of key-value pairs
/// This macro is taken from the [maplit](https://github.com/bluss/maplit/blob/master/src/lib.rs)
/// crate to minimize external dependencies.
///
/// ## Example
///
/// ```
/// use jmespath_community as jmespath;
/// use jmespath::map;
/// use jmespath::Map;
///
/// let map = map!{
///     "a" => 1,
///     "b" => 2,
/// };
/// assert_eq!(map["a"], 1);
/// assert_eq!(map["b"], 2);
/// assert_eq!(map.get("c"), None);
/// ```
#[macro_export]
macro_rules! map {
    // trailing comma case
    ($($key:expr => $value:expr,)+) => (map!($($key => $value),+));

    ( $($key:expr => $value:expr),* ) => {
        {
            let mut _map = Map::new();
            $( let _ = _map.insert($key, $value); )*
            _map
        }
    };
}

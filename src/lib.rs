use lazy_static::lazy_static;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::{collections::HashMap, sync::Mutex};
use time::ext::NumericalDuration;
use time::OffsetDateTime;

pub struct CacheEntry {
    value: String,
    expiring: i64,
}

lazy_static! {
    static ref CACHE: Mutex<HashMap<String, CacheEntry>> = Mutex::new(HashMap::new());
}

#[derive(Clone, Debug)]
pub struct Cache(String);

impl Cache {
    pub fn new(prefix: &str) -> Self {
        Self(prefix.to_string())
    }

    fn get_key<K: Serialize>(&self, key: &K) -> String {
        format!("{}:{}", self.0, serde_json::to_string(key).unwrap())
    }

    pub fn get<V: DeserializeOwned, K: Serialize>(&self, key: &K) -> Option<V> {
        let cache = CACHE.lock().unwrap();
        
        match cache.get(&self.get_key(key)) {
            Some(entry) => {
                if entry.expiring > OffsetDateTime::now_utc().unix_timestamp() {
                    let result = serde_json::from_str::<V>(&entry.value.clone()).unwrap();
                    return Some(result);
                }
                None
            }
            None => None,
        }
    }

    pub fn insert<K: Serialize, V: Serialize>(&self, key: &K, value: &V, ttl: i64) {
        let mut cache = CACHE.lock().unwrap();
        let cache_key = self.get_key(key);
        let expiring = OffsetDateTime::now_utc()
            .checked_add(ttl.seconds())
            .unwrap()
            .unix_timestamp();

        cache.insert(
            cache_key,
            CacheEntry {
                expiring,
                value: serde_json::to_string(value).unwrap(),
            },
        );
    }
}


#[cfg(test)]
mod test_cache {
    use serde::{Deserialize, Serialize};
    use super::Cache;

    const PREFIX: &str = "prefix";
    const KEY: &str = "key";
    const VALUE: &str = "value";

    #[test]
    fn ok() {
        let cache = Cache::new(PREFIX);
        cache.insert(&KEY, &VALUE, 10);

        assert_eq!(cache.get::<String, _>(&KEY).unwrap(), VALUE);
    }

    #[test]
    fn expiring() {
      let cache = Cache::new(PREFIX);
      cache.insert(&KEY, &VALUE, -1);

      assert_eq!(cache.get::<String, _>(&KEY), None);
    }

    #[test]
    fn payload() {
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct Value {
            pub name: String,
        }

        let value = Value { name: VALUE.into() };

        let cache = Cache::new(PREFIX);
        cache.insert(&KEY, &value, 10);

        assert_eq!(cache.get::<Value, _>(&KEY).unwrap().name, VALUE);
    }

    #[test]
    fn key() {
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct Key {
            pub name: String,
        }

        let key = Key { name: KEY.into() };

        let cache = Cache::new(PREFIX);
        cache.insert(&key, &VALUE, 10);

        assert_eq!(cache.get::<String, _>(&key).unwrap(), VALUE);
    }

}

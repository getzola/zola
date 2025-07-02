use crate::Result;
use bincode;
use dashmap::DashMap;
use errors::Context;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::hash::Hash;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Generic cache using DashMap, storing data in a binary file
#[derive(Debug)]
pub struct GenericCache<K, V>
where
    K: Eq + Hash + Serialize + for<'de> Deserialize<'de>,
    V: Serialize + for<'de> Deserialize<'de>,
{
    cache_file: PathBuf,
    cache: DashMap<K, V>,
}

impl<K, V> GenericCache<K, V>
where
    K: Eq + Hash + Serialize + for<'de> Deserialize<'de>,
    V: Serialize + for<'de> Deserialize<'de>,
{
    /// Get the directory where the cache is stored
    pub fn dir(&self) -> &Path {
        self.cache_file.parent().unwrap()
    }

    /// Create a new cache for a specific type
    pub fn new(base_cache_dir: &Path, filename: &str) -> crate::Result<Self> {
        // Full path to the cache file
        let cache_file = base_cache_dir.join(filename);

        // Attempt to load existing cache
        let cache = match Self::read_cache(&cache_file) {
            Ok(maybe_cache) => match maybe_cache {
                Some(c) => {
                    println!("Loaded cache from {:?} ({:?})", cache_file, c.len());
                    c
                }
                None => DashMap::new(),
            },
            Err(e) => {
                println!("Failed to load cache: {}", e);
                DashMap::new()
            }
        };

        Ok(Self { cache_file, cache })
    }

    /// Read cache from file
    fn read_cache(cache_file: &Path) -> Result<Option<DashMap<K, V>>> {
        if !cache_file.exists() {
            return Ok(None);
        }

        let mut file = File::open(cache_file)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        bincode::deserialize(&buffer).context("Failed to deserialize cache").map(Some)
    }

    /// Write cache to file
    pub fn write(&self) -> Result<()> {
        fs::create_dir_all(self.dir())?;
        let serialized = bincode::serialize(&self.cache).context("Failed to serialize cache")?;

        let mut file =
            OpenOptions::new().write(true).create(true).truncate(true).open(&self.cache_file)?;

        file.write_all(&serialized)?;
        Ok(())
    }

    /// Get a reference to the underlying DashMap
    pub fn inner(&self) -> &DashMap<K, V> {
        &self.cache
    }

    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.cache.get(key).map(|r| r.value().clone())
    }

    pub fn insert(&self, key: K, value: V) {
        self.cache.insert(key, value);
    }

    /// Clear the cache and remove the file
    pub fn clear(&self) -> Result<()> {
        self.cache.clear();

        if self.cache_file.exists() {
            fs::remove_file(&self.cache_file)?;
        }

        Ok(())
    }
}

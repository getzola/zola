mod config;
pub mod highlighting;
mod theme;

use std::{marker::PhantomData, path::Path};

pub use crate::config::{
    languages::LanguageOptions,
    link_checker::LinkChecker,
    link_checker::LinkCheckerLevel,
    search::{IndexFormat, Search},
    slugify::Slugify,
    taxonomies::TaxonomyConfig,
    Config,
};
use errors::Result;
use serde::Deserialize;

/// Get and parse the config.
/// If it doesn't succeed, exit
pub fn get_config(filename: &Path) -> Result<Config> {
    Config::from_file(filename)
}

/// This is used to print an error message for deprecated fields
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Deprecated<T> {
    _type: PhantomData<T>,
}

impl<T> Deprecated<T> {
    pub fn new() -> Self {
        Self { _type: PhantomData }
    }
}

pub trait DeprecationReason {
    const REASON: &'static str;
}

impl<'de, T: DeprecationReason> Deserialize<'de> for Deprecated<T> {
    fn deserialize<D>(_deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(serde::de::Error::custom(format!("Failed to parse a deprecated option: {}", T::REASON)))
    }
}

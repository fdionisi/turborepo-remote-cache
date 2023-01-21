use std::path::PathBuf;

use async_trait::async_trait;
use bytes::Bytes;
use hyper::Body;

#[derive(Debug, thiserror::Error)]
pub enum StorageAdapterError {
    #[error("unknown error")]
    Unknown,
}

#[async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn get(&self, path: PathBuf) -> Result<Bytes, StorageAdapterError>;

    async fn exists(&self, path: PathBuf) -> Result<bool, StorageAdapterError>;

    async fn upload<'a>(&self, path: PathBuf, artifact: Bytes) -> Result<(), StorageAdapterError>;

    async fn upload_<'a>(&self, path: PathBuf, artifact: Body) -> Result<(), StorageAdapterError>;
}

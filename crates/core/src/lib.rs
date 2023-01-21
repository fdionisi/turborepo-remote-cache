use std::{path::PathBuf, sync::Arc};

use bytes::Bytes;
use hyper::Body;
use turborepo_storage_adapter::{StorageAdapter, StorageAdapterError};

#[derive(Debug)]
pub enum TurborepoError {
    Unknown,
    StorageAdapter(StorageAdapterError),
}

impl std::error::Error for TurborepoError {}

impl std::fmt::Display for TurborepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<StorageAdapterError> for TurborepoError {
    fn from(value: StorageAdapterError) -> Self {
        TurborepoError::StorageAdapter(value)
    }
}

/// Core functionality to check exsistence, download and upload artifacts.
///
/// At its core, it is a thin wrapper around
pub struct TurborepoCore {
    storage: Arc<dyn StorageAdapter + Sync + Send>,
}

pub struct TurborepoCoreBuilder
// where
//     S: StorageAdapter + 'static,
{
    storage: Option<Arc<dyn StorageAdapter + Sync + Send>>,
}

impl TurborepoCore {
    pub fn builder() -> TurborepoCoreBuilder
// where
    //     S: StorageAdapter + 'static,
    {
        TurborepoCoreBuilder { storage: None }
    }

    pub async fn get_cached_artifact(
        &self,
        artifact_id: String,
        team_id: String,
    ) -> Result<Bytes, TurborepoError> {
        Ok(self
            .storage
            .get(Self::artifact_path(&artifact_id, &team_id))
            .await?)
    }

    pub async fn create_cached_artifact(
        &self,
        artifact_id: String,
        team_id: String,
        artifact: Body,
    ) -> Result<(), TurborepoError> {
        Ok(self
            .storage
            .upload_(Self::artifact_path(&artifact_id, &team_id), artifact)
            .await?)
    }

    pub async fn exists_cached_artifact(
        &self,
        artifact_id: &String,
        team_id: &String,
    ) -> Result<bool, TurborepoError> {
        Ok(self
            .storage
            .exists(Self::artifact_path(artifact_id, team_id))
            .await?)
    }

    fn artifact_path(artifact_id: &String, team_id: &String) -> PathBuf {
        PathBuf::from(format!("{team_id}/{artifact_id}"))
    }
}

impl TurborepoCoreBuilder
// where
//     S: StorageAdapter + Sync + Send + 'static,
{
    pub async fn build(&mut self) -> Result<TurborepoCore, TurborepoError> {
        let storage = self.storage.take().unwrap();

        Ok(TurborepoCore { storage })
    }

    pub fn with_storage<S: StorageAdapter + Send + Sync + Sized + 'static>(
        &mut self,
        storage: Arc<S>,
    ) -> &mut Self {
        self.storage.replace(storage);

        self
    }
}

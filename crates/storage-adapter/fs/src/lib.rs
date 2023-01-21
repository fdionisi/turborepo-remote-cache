use std::path::PathBuf;

use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use hyper::Body;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use turborepo_storage_adapter::{StorageAdapter, StorageAdapterError};

pub struct FsStorageAdapter {
    bucket: String,
}

impl FsStorageAdapter {
    pub fn builder() -> FsStorageAdapterBuilder {
        FsStorageAdapterBuilder { bucket: None }
    }

    async fn _exists(&self, path: &PathBuf) -> Result<(), StorageAdapterError> {
        let mut dir = path.clone();
        dir.pop();

        if let Err(_) = fs::read_dir(PathBuf::from(&self.bucket).join(&dir)).await {
            fs::create_dir_all(PathBuf::from(&self.bucket).join(&dir))
                .await
                .unwrap();
        }

        match fs::metadata(PathBuf::from(&self.bucket).join(path)).await {
            Ok(_) => Ok(()),
            Err(_) => Err(StorageAdapterError::Unknown),
        }
    }
}

#[async_trait]
impl StorageAdapter for FsStorageAdapter {
    async fn get(&self, path: PathBuf) -> Result<Bytes, StorageAdapterError> {
        let mut dir = path.clone();
        dir.pop();

        if let Err(_) = fs::read_dir(PathBuf::from(&self.bucket).join(&dir)).await {
            fs::create_dir_all(PathBuf::from(&self.bucket).join(&dir))
                .await
                .unwrap();
        }

        self._exists(&path).await?;

        let mut file = fs::File::open(PathBuf::from(&self.bucket).join(path))
            .await
            .unwrap();

        let mut buf = vec![];
        file.read_to_end(&mut buf).await.unwrap();

        Ok(buf.into())
    }

    async fn exists(&self, path: PathBuf) -> Result<bool, StorageAdapterError> {
        let mut dir = path.clone();
        dir.pop();

        if let Err(_) = fs::read_dir(PathBuf::from(&self.bucket).join(&dir)).await {
            fs::create_dir_all(PathBuf::from(&self.bucket).join(&dir))
                .await
                .unwrap();
        }

        Ok(matches!(
            fs::metadata(PathBuf::from(&self.bucket).join(path)).await,
            Ok(_)
        ))
    }

    async fn upload<'a>(&self, path: PathBuf, artifact: Bytes) -> Result<(), StorageAdapterError> {
        let mut dir = path.clone();
        dir.pop();

        if let Err(_) = fs::read_dir(PathBuf::from(&self.bucket).join(&dir)).await {
            fs::create_dir_all(PathBuf::from(&self.bucket).join(&dir))
                .await
                .unwrap();
        }

        let mut file = fs::File::create(PathBuf::from(&self.bucket).join(path))
            .await
            .unwrap();

        file.write_all(&artifact.to_vec()).await.unwrap();

        Ok(())
    }

    async fn upload_<'a>(
        &self,
        path: PathBuf,
        mut artifact: Body,
    ) -> Result<(), StorageAdapterError> {
        let mut dir = path.clone();
        dir.pop();

        if let Err(_) = fs::read_dir(PathBuf::from(&self.bucket).join(&dir)).await {
            fs::create_dir_all(PathBuf::from(&self.bucket).join(&dir))
                .await
                .unwrap();
        }

        let mut file = fs::File::create(PathBuf::from(&self.bucket).join(path))
            .await
            .unwrap();

        while let Some(chunk) = artifact.next().await {
            file.write(&chunk.unwrap().to_vec()).await.unwrap();
        }

        Ok(())
    }
}

pub struct FsStorageAdapterBuilder {
    bucket: Option<String>,
}

impl FsStorageAdapterBuilder {
    pub async fn build(&self) -> FsStorageAdapter {
        let bucket = self.bucket.clone().unwrap();
        FsStorageAdapter { bucket }
    }

    pub fn with_bucket(&mut self, bucket: String) -> &mut Self {
        self.bucket.replace(bucket);

        self
    }
}

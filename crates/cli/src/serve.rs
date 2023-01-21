use std::{fmt, sync::Arc};

use clap::{Parser, ValueEnum};
use turborepo_aws_s3_storage_adapter::AwsS3StorageAdapter;
use turborepo_core::TurborepoCore;
use turborepo_fs_storage_adapter::FsStorageAdapter;
use turborepo_server::TurborepoServer;

#[derive(Clone, Debug, ValueEnum)]
enum Storage {
    Fs,
    Aws,
}

#[derive(Debug, Parser)]
pub struct Serve {
    #[arg(long, default_missing_value = "127.0.0.1")]
    api_address: String,
    #[arg(long)]
    api_port: u16,
    #[arg(long)]
    bucket: String,
    #[arg(long)]
    token: String,
    #[arg(long, value_enum, default_value_t = Storage::Fs, default_missing_value = "fs",)]
    storage: Storage,
}

impl fmt::Display for Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Storage::Fs => "fs",
                Storage::Aws => "aws",
            }
        )
    }
}

impl Serve {
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        match self.storage {
            Storage::Aws => {
                TurborepoServer::builder()
                    .with_token(self.token.clone())
                    .with_core(
                        TurborepoCore::builder()
                            .with_storage(Arc::new(
                                AwsS3StorageAdapter::builder()
                                    .with_bucket(self.bucket.clone())
                                    .build()
                                    .await,
                            ))
                            .build()
                            .await?,
                    )
                    .build()
                    .listen()
                    .await?;
            }
            Storage::Fs => {
                TurborepoServer::builder()
                    .with_token(self.token.clone())
                    .with_core(
                        TurborepoCore::builder()
                            .with_storage(Arc::new(
                                FsStorageAdapter::builder()
                                    .with_bucket(self.bucket.clone())
                                    .build()
                                    .await,
                            ))
                            .build()
                            .await?,
                    )
                    .build()
                    .listen()
                    .await?
            }
        };

        Ok(())
    }
}

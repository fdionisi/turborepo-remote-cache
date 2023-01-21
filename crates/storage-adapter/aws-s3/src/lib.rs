use std::path::PathBuf;

use async_trait::async_trait;
use aws_sdk_s3::{
    model::{CompletedMultipartUpload, CompletedPart},
    output::CreateMultipartUploadOutput,
    Client,
};
use aws_smithy_http::{body::SdkBody, byte_stream::ByteStream};
use bytes::Bytes;
use futures::StreamExt;
use hyper::Body;
use turborepo_storage_adapter::{StorageAdapter, StorageAdapterError};

pub struct AwsS3StorageAdapter {
    client: Client,
    bucket: String,
}

impl AwsS3StorageAdapter {
    pub fn builder() -> AwsS3StorageAdapterBuilder {
        AwsS3StorageAdapterBuilder { bucket: None }
    }
}

impl AwsS3StorageAdapter {
    async fn _exists(&self, path: &PathBuf) -> Result<(), StorageAdapterError> {
        self.client
            .get_object()
            .bucket(&self.bucket)
            .key(path.to_str().unwrap())
            .send()
            .await
            .map(|_| ())
            .map_err(|_| StorageAdapterError::Unknown)
    }
}

#[async_trait]
impl StorageAdapter for AwsS3StorageAdapter {
    async fn get(&self, path: PathBuf) -> Result<Bytes, StorageAdapterError> {
        self._exists(&path).await?;

        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(path.to_string_lossy())
            .send()
            .await
            .unwrap();

        let inner = object.body.collect().await.unwrap();
        Ok(inner.into_bytes())
    }

    async fn exists(&self, path: PathBuf) -> Result<bool, StorageAdapterError> {
        Ok(self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(path.to_str().unwrap())
            .send()
            .await
            .map(|_| true)
            .unwrap_or(false))
    }

    async fn upload<'a>(&self, path: PathBuf, artifact: Bytes) -> Result<(), StorageAdapterError> {
        let create_multipart_upload_output: CreateMultipartUploadOutput = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(path.to_str().unwrap())
            .send()
            .await
            .unwrap();

        let upload_id = create_multipart_upload_output.upload_id().unwrap();

        let mut upload_parts = vec![];

        let sdk_body = SdkBody::from(artifact);
        let body = ByteStream::new(sdk_body);
        let upload_part_res = self
            .client
            .upload_part()
            .bucket(&self.bucket)
            .key(path.to_str().unwrap())
            .upload_id(upload_id)
            .body(body)
            .part_number(1)
            .send()
            .await
            .unwrap();

        upload_parts.push(
            CompletedPart::builder()
                .e_tag(upload_part_res.e_tag.unwrap_or_default())
                .part_number(1)
                .build(),
        );

        let completed_multipart_upload: CompletedMultipartUpload =
            CompletedMultipartUpload::builder()
                .set_parts(Some(upload_parts))
                .build();

        let _complete_multipart_upload_res = self
            .client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(path.to_str().unwrap())
            .multipart_upload(completed_multipart_upload)
            .upload_id(upload_id)
            .send()
            .await
            .unwrap();

        Ok(())
    }

    async fn upload_<'a>(
        &self,
        path: PathBuf,
        mut artifact: Body,
    ) -> Result<(), StorageAdapterError> {
        let create_multipart_upload_output: CreateMultipartUploadOutput = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(path.to_str().unwrap())
            .send()
            .await
            .unwrap();

        let upload_id = create_multipart_upload_output.upload_id().unwrap();

        let mut upload_parts = vec![];

        while let Some(chunk) = artifact.next().await {
            if let Err(_) = chunk {
                continue;
            }

            let sdk_body = SdkBody::from(chunk.unwrap());
            let body = ByteStream::new(sdk_body);
            let upload_part_res = self
                .client
                .upload_part()
                .bucket(&self.bucket)
                .key(path.to_str().unwrap())
                .upload_id(upload_id)
                .body(body)
                .part_number(1)
                .send()
                .await
                .unwrap();

            upload_parts.push(
                CompletedPart::builder()
                    .e_tag(upload_part_res.e_tag.unwrap_or_default())
                    .part_number(1)
                    .build(),
            );
        }

        let completed_multipart_upload: CompletedMultipartUpload =
            CompletedMultipartUpload::builder()
                .set_parts(Some(upload_parts))
                .build();

        let _complete_multipart_upload_res = self
            .client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(path.to_str().unwrap())
            .multipart_upload(completed_multipart_upload)
            .upload_id(upload_id)
            .send()
            .await
            .unwrap();

        Ok(())
    }
}

pub struct AwsS3StorageAdapterBuilder {
    bucket: Option<String>,
}

impl AwsS3StorageAdapterBuilder {
    pub async fn build(&self) -> AwsS3StorageAdapter {
        let bucket = self.bucket.clone().take().unwrap();
        let aws_config = aws_config::from_env().load().await;

        let client = Client::new(&aws_config);
        AwsS3StorageAdapter { client, bucket }
    }

    pub fn with_bucket(&mut self, bucket: String) -> &mut Self {
        self.bucket.replace(bucket);

        self
    }
}

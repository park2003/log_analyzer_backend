use crate::application::StorageService;
use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_config;
use aws_sdk_s3::{Client as S3Client, config::Region};
use bytes::Bytes;

pub struct S3StorageService {
    client: S3Client,
    bucket: String,
}

impl S3StorageService {
    pub async fn new(bucket: String) -> Result<Self> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = S3Client::new(&config);

        Ok(Self { client, bucket })
    }

    pub async fn new_with_config(bucket: String, region: Option<String>) -> Result<Self> {
        let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());

        if let Some(region_str) = region {
            config_loader = config_loader.region(Region::new(region_str));
        }

        let config = config_loader.load().await;
        let client = S3Client::new(&config);

        Ok(Self { client, bucket })
    }

    fn parse_s3_uri(&self, uri: &str) -> Result<(String, String)> {
        // Parse s3://bucket/path/to/object
        if !uri.starts_with("s3://") {
            anyhow::bail!("Invalid S3 URI: {}", uri);
        }

        let without_prefix = uri.strip_prefix("s3://").unwrap();
        let parts: Vec<&str> = without_prefix.splitn(2, '/').collect();

        if parts.len() != 2 {
            anyhow::bail!("Invalid S3 URI format: {}", uri);
        }

        Ok((parts[0].to_string(), parts[1].to_string()))
    }
}

#[async_trait]
impl StorageService for S3StorageService {
    async fn list_images(&self, uri: &str) -> Result<Vec<String>> {
        let (bucket, prefix) = self.parse_s3_uri(uri)?;

        let mut images = Vec::new();
        let mut continuation_token = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&bucket)
                .prefix(&prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await.context("Failed to list S3 objects")?;

            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        // Filter for image files
                        if key.ends_with(".jpg")
                            || key.ends_with(".jpeg")
                            || key.ends_with(".png")
                            || key.ends_with(".webp")
                        {
                            images.push(format!("s3://{bucket}/{key}"));
                        }
                    }
                }
            }

            if response.is_truncated.unwrap_or(false) {
                continuation_token = response.next_continuation_token;
            } else {
                break;
            }
        }

        Ok(images)
    }

    async fn download_image(&self, uri: &str) -> Result<Vec<u8>> {
        let (bucket, key) = self.parse_s3_uri(uri)?;

        let response = self
            .client
            .get_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
            .context(format!("Failed to download object from S3: {uri}"))?;

        let data = response
            .body
            .collect()
            .await
            .context("Failed to read S3 object body")?;

        Ok(data.into_bytes().to_vec())
    }

    async fn upload_dataset(&self, data: Vec<String>, uri: &str) -> Result<()> {
        let (bucket, prefix) = self.parse_s3_uri(uri)?;

        // Create a manifest file listing all curated images
        let manifest = data.join("\n");
        let manifest_key = format!("{prefix}/manifest.txt");

        self.client
            .put_object()
            .bucket(&bucket)
            .key(&manifest_key)
            .body(Bytes::from(manifest).into())
            .send()
            .await
            .context("Failed to upload manifest to S3")?;

        // Copy selected images to curated directory
        for image_uri in data {
            let (source_bucket, source_key) = self.parse_s3_uri(&image_uri)?;

            // Extract filename from source key
            let filename = source_key
                .split('/')
                .next_back()
                .ok_or_else(|| anyhow::anyhow!("Invalid image URI: {}", image_uri))?;

            let dest_key = format!("{prefix}/{filename}");
            let copy_source = format!("{source_bucket}/{source_key}");

            self.client
                .copy_object()
                .bucket(&bucket)
                .key(&dest_key)
                .copy_source(&copy_source)
                .send()
                .await
                .context(format!("Failed to copy object: {image_uri}"))?;
        }

        Ok(())
    }
}

// Alternative implementation using local filesystem for development
pub struct LocalStorageService {
    base_path: std::path::PathBuf,
}

impl LocalStorageService {
    pub fn new(base_path: std::path::PathBuf) -> Self {
        Self { base_path }
    }
}

#[async_trait]
impl StorageService for LocalStorageService {
    async fn list_images(&self, uri: &str) -> Result<Vec<String>> {
        let path = self.base_path.join(uri);
        let mut images = Vec::new();

        let mut entries = tokio::fs::read_dir(&path)
            .await
            .context(format!("Failed to read directory: {path:?}"))?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(ext) = path.extension()
                && (ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "webp")
                && let Some(path_str) = path.to_str()
            {
                images.push(path_str.to_string());
            }
        }

        Ok(images)
    }

    async fn download_image(&self, uri: &str) -> Result<Vec<u8>> {
        let path = self.base_path.join(uri);
        tokio::fs::read(&path)
            .await
            .context(format!("Failed to read file: {path:?}"))
    }

    async fn upload_dataset(&self, data: Vec<String>, uri: &str) -> Result<()> {
        let dest_path = self.base_path.join(uri);

        // Create destination directory
        tokio::fs::create_dir_all(&dest_path)
            .await
            .context(format!("Failed to create directory: {dest_path:?}"))?;

        // Write manifest
        let manifest_path = dest_path.join("manifest.txt");
        let manifest = data.join("\n");
        tokio::fs::write(&manifest_path, manifest)
            .await
            .context("Failed to write manifest")?;

        // Copy files
        for source_path in data {
            let source = std::path::Path::new(&source_path);
            if let Some(filename) = source.file_name() {
                let dest = dest_path.join(filename);
                tokio::fs::copy(&source, &dest)
                    .await
                    .context(format!("Failed to copy file: {source:?} -> {dest:?}"))?;
            }
        }

        Ok(())
    }
}

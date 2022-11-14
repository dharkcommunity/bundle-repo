use s3::creds::error::CredentialsError;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::Bucket;

use crate::config::BucketInfo;

#[derive(thiserror::Error, Debug)]
pub enum LoadBucketError {
    #[error("Invalid credentials: {0}")]
    CredentialsError(#[from] CredentialsError),

    #[error("{0}")]
    S3Error(#[from] S3Error),
}

pub fn load_bucket(bucket_info: &BucketInfo) -> Result<Bucket, LoadBucketError> {
    let credentials = Credentials::new(
        Some(&bucket_info.credentials.access_key),
        Some(&bucket_info.credentials.secret_key),
        None,
        None,
        None,
    )?;

    let bucket = Bucket::new(
        &bucket_info.name,
        bucket_info.region.clone().into_region(),
        credentials,
    )?;
    Ok(bucket)
}

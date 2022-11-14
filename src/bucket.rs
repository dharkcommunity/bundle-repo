use s3::creds::Credentials;
use s3::Bucket;

use crate::config::BucketInfo;

pub fn load_bucket(bucket_info: BucketInfo) -> Bucket {
    let mut credentials = Credentials::default().unwrap();
    credentials.access_key = Some(bucket_info.credentials.access_key);
    Bucket::new(
        &bucket_info.name,
        bucket_info.region.into_region(),
        credentials,
    )
    .unwrap()
}

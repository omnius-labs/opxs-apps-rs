use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageConvertJobSqsMessage {
    #[serde(rename = "Records")]
    pub records: Vec<S3EventRecord>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct S3EventRecord {
    pub s3: S3Entity,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct S3Entity {
    pub bucket: S3Bucket,
    pub object: S3Object,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct S3Bucket {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct S3Object {
    pub key: String,
}

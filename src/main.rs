#![allow(dead_code)]

mod client;

use std::error::Error;
use client::HttpClient;
use serde::Deserialize;

const AWS_ECS_IP: &str = "169.254.170.2";
const AWS_EC2_IP: &str = "169.254.169.254";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = HttpClient::default();
    let creds = if let Ok(relative_uri) = std::env::var("AWS_CONTAINER_CREDENTIALS_RELATIVE_URI") {
        AwsCredential::get_from_ecs(relative_uri, &client).await.unwrap()
    } else {
        AwsCredential::get_from_ec2(&client).await.unwrap()
    };
    println!("{:?}", creds);

    Ok(())
}

#[derive(Debug, Deserialize)]
struct AwsCredential {
    #[serde(rename = "AccessKeyId")]
    access_key: String,

    #[serde(rename = "SecretAccessKey")]
    secret_key: String,

    #[serde(rename = "Token")]
    session_token: Option<String>,
}

impl AwsCredential {

    /// Obtains credentials from the ECS endpoint.
    async fn get_from_ecs(relative_uri: String, http_client: &HttpClient) -> Result<Self, Box<dyn Error>> {
        // Use the local IP address that AWS uses for ECS agents.
        let uri = format!("http://{}/{}", AWS_ECS_IP, relative_uri);

        http_client
            .get_and_deserialize_json(&uri, &[])
            .await
    }

    /// Obtains temporary credentials for an EC2 instance to use for authentication.
    async fn get_from_ec2(http_client: &HttpClient) -> Result<Self, Box<dyn Error>> {
        let temporary_token = http_client
            .put_and_read_string(
                &format!("http://{}/latest/api/token", AWS_EC2_IP),
                &[("X-aws-ec2-metadata-token-ttl-seconds", "30")],
            )
            .await?;

        let role_name_uri = format!(
            "http://{}/latest/meta-data/iam/security-credentials/",
            AWS_EC2_IP
        );

        let role_name = http_client
            .get_and_read_string(
                &role_name_uri,
                &[("X-aws-ec2-metadata-token", &temporary_token[..])],
            )
            .await?;

        let credential_uri = format!("{}/{}", role_name_uri, role_name);

        http_client
            .get_and_deserialize_json(
                &credential_uri,
                &[("X-aws-ec2-metadata-token", &temporary_token[..])],
            )
            .await
    }
}

use reqwest::{Client, StatusCode};
use uuid::Uuid;

use crate::config::ImageServiceConfig;

#[derive(Debug)]
pub enum ImageServiceApiError {
    Http(reqwest::Error),
    UnexpectedStatus(StatusCode, String),
}

impl std::fmt::Display for ImageServiceApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageServiceApiError::Http(e) => write!(f, "HTTP error: {}", e),
            ImageServiceApiError::UnexpectedStatus(code, body) => {
                write!(f, "Unexpected status '{}': {}", code, body)
            }
        }
    }
}

impl std::error::Error for ImageServiceApiError {}

impl From<reqwest::Error> for ImageServiceApiError {
    fn from(err: reqwest::Error) -> Self {
        ImageServiceApiError::Http(err)
    }
}

pub struct ImageServiceClient {
    config: ImageServiceConfig,
    client: Client,
}

impl ImageServiceClient {
    pub fn new(config: ImageServiceConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Approves an image by UUID
    pub async fn approve_image(&self, id: Uuid) -> Result<(), ImageServiceApiError> {
        log::debug!("Approving image with id '{}'", id);
        let url = format!("{}/approve/{}", self.config.url, id);
        let res = self
            .client
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => Ok(()),
            status => {
                let body = res.text().await.unwrap_or_default();
                log::error!(
                    "Failed to approve image with id '{}': Got '{}' with body '{}'",
                    id,
                    status,
                    body
                );
                Err(ImageServiceApiError::UnexpectedStatus(status, body))
            }
        }
    }

    /// Unapproves an image by UUID
    pub async fn unapprove_image(&self, id: Uuid) -> Result<(), ImageServiceApiError> {
        log::debug!("Unapproving image with id '{}'", id);
        let url = format!("{}/unapprove/{}", self.config.url, id);
        let res = self
            .client
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => Ok(()),
            status => {
                let body = res.text().await.unwrap_or_default();
                log::error!(
                    "Failed to unapprove image with id '{}': Got '{}' with body '{}'",
                    id,
                    status,
                    body
                );
                Err(ImageServiceApiError::UnexpectedStatus(status, body))
            }
        }
    }
}

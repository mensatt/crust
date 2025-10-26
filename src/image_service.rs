use anyhow::{anyhow, Context, Result};
use log::{error, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ImageServiceConfig {
    pub url: String,
    pub api_key: String,
}

#[derive(Clone)]
pub struct ImageServiceClient {
    config: ImageServiceConfig,
    client: reqwest::Client,
}

impl ImageServiceClient {
    pub fn new(config: ImageServiceConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Submit an image to the image service
    /// Returns the UUID of the submitted image on success
    pub async fn submit_image(&self, image_id: Uuid) -> Result<Uuid> {
        let url = format!("{}submit/{}", self.config.url, image_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .context("Failed to send request to image service")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Image service responded with status code: {}",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .context("Failed to read response body from image service")?;

        Uuid::parse_str(&body).context("Failed to parse UUID from image service response")
    }

    /// Submit multiple images to the image service
    /// Returns a vector of successfully submitted image UUIDs
    /// Errors for individual images are logged but don't fail the entire operation
    pub async fn submit_images(&self, image_ids: Vec<Uuid>) -> Vec<Uuid> {
        let mut submitted = Vec::new();

        for image_id in image_ids {
            match self.submit_image(image_id).await {
                Ok(uuid) => submitted.push(uuid),
                Err(e) => {
                    error!("Failed to submit image {}: {}", image_id, e);
                    // Continue with other images despite this failure
                }
            }
        }

        submitted
    }

    /// Rotate an image in the image service
    pub async fn rotate_image(&self, image_id: Uuid, angle: i64) -> Result<()> {
        let url = format!("{}rotate?id={}&angle={}", self.config.url, image_id, angle);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .context("Failed to send rotation request to image service")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Image service responded with status code: {} for rotation",
                response.status()
            ));
        }

        Ok(())
    }

    /// Approve an image in the image service
    /// Returns the UUID of the approved image on success
    pub async fn approve_image(&self, image_id: Uuid) -> Result<Uuid> {
        let url = format!("{}approve/{}", self.config.url, image_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .context("Failed to send approve request to image service")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Image service responded with status code: {} for approval",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .context("Failed to read response body from image service")?;

        Uuid::parse_str(&body).context("Failed to parse UUID from image service response")
    }

    /// Approve multiple images in the image service
    /// Returns a vector of successfully approved image UUIDs
    /// If any image fails to approve, returns an error with the list of successfully approved images
    pub async fn approve_images(&self, image_ids: Vec<Uuid>) -> Result<Vec<Uuid>> {
        let mut approved = Vec::new();

        for image_id in image_ids {
            match self.approve_image(image_id).await {
                Ok(uuid) => approved.push(uuid),
                Err(e) => {
                    error!("Failed to approve image {}: {}", image_id, e);
                    // Return error with partial success for rollback purposes
                    return Err(anyhow!(
                        "Failed to approve image {}: {}. Successfully approved: {:?}",
                        image_id,
                        e,
                        approved
                    ));
                }
            }
        }

        Ok(approved)
    }

    /// Unapprove an image in the image service
    /// Returns the UUID of the unapproved image on success
    pub async fn unapprove_image(&self, image_id: Uuid) -> Result<Uuid> {
        let url = format!("{}unapprove/{}", self.config.url, image_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .context("Failed to send unapprove request to image service")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Image service responded with status code: {} for unapproval",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .context("Failed to read response body from image service")?;

        Uuid::parse_str(&body).context("Failed to parse UUID from image service response")
    }

    /// Unapprove multiple images in the image service
    /// Returns a vector of successfully unapproved image UUIDs
    /// If any image fails to unapprove, returns an error with the list of successfully unapproved images
    pub async fn unapprove_images(&self, image_ids: Vec<Uuid>) -> Result<Vec<Uuid>> {
        let mut unapproved = Vec::new();

        for image_id in image_ids {
            match self.unapprove_image(image_id).await {
                Ok(uuid) => unapproved.push(uuid),
                Err(e) => {
                    error!("Failed to unapprove image {}: {}", image_id, e);
                    // Return error with partial success for rollback purposes
                    return Err(anyhow!(
                        "Failed to unapprove image {}: {}. Successfully unapproved: {:?}",
                        image_id,
                        e,
                        unapproved
                    ));
                }
            }
        }

        Ok(unapproved)
    }

    /// Delete an image from the image service
    /// Returns the UUID of the deleted image on success
    pub async fn delete_image(&self, image_id: Uuid) -> Result<Uuid> {
        let url = format!("{}image/{}", self.config.url, image_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .context("Failed to send delete request to image service")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Image service responded with status code: {} for deletion",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .context("Failed to read response body from image service")?;

        Uuid::parse_str(&body).context("Failed to parse UUID from image service response")
    }

    /// Delete multiple images from the image service
    /// Returns a vector of successfully deleted image UUIDs
    /// Errors for individual images are logged but don't fail the entire operation
    pub async fn delete_images(&self, image_ids: Vec<Uuid>) -> Vec<Uuid> {
        let mut deleted = Vec::new();

        for image_id in image_ids {
            match self.delete_image(image_id).await {
                Ok(uuid) => deleted.push(uuid),
                Err(e) => {
                    warn!("Failed to delete image {}: {}", image_id, e);
                    // Continue with other images despite this failure
                }
            }
        }

        deleted
    }
}

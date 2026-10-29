use crate::domain::utils::webp::png_to_webp;
use crate::ports::image_store::{Illustration, Image, ImageStore};
use async_trait::async_trait;
use cards_sdk::CardInfo;
use contracts::image::{ImageFile, ImageKind};
use std::env;
use uuid::Uuid;

pub struct FileSystem {
    image_dir: String,
    illustration_dir: String,
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem {
    /// # Panics
    /// Panics if the `IMAGES_DIR` environment variable is not set.
    #[must_use]
    pub fn new() -> Self {
        let base_dir = env::var("IMAGES_DIR").expect("Images dir wasn't in env vars");
        Self::from_base_dir(&base_dir)
    }

    #[must_use]
    pub fn from_base_dir(base_dir: &str) -> Self {
        Self {
            image_dir: format!("{base_dir}/images/"),
            illustration_dir: format!("{base_dir}/illustrations/"),
        }
    }

    fn path(&self, file: ImageFile) -> String {
        let dir = match file.kind() {
            ImageKind::Card => &self.image_dir,
            ImageKind::Illustration => &self.illustration_dir,
        };
        format!("{dir}{file}")
    }

    pub async fn save(&self, path: &str, bytes: Vec<u8>) {
        if let Err(why) = tokio::fs::write(&path, bytes).await {
            log::error!("Failed to save file: {why}");
        }
    }

    async fn delete(&self, file: ImageFile) {
        if let Err(why) = tokio::fs::remove_file(self.path(file)).await {
            log::warn!("Failed to delete orphaned {file}: {why}");
        }
    }
}

#[async_trait]
impl ImageStore for FileSystem {
    async fn card_image_exists(&self, card: &CardInfo) -> bool {
        let path = self.path(ImageFile::card(card.image.id));

        tokio::fs::try_exists(&path).await.unwrap_or(false)
    }

    async fn card_illustration_exists(&self, card: &CardInfo) -> bool {
        let Some(illustration) = card.illustration.as_ref() else {
            return true;
        };

        let path = self.path(ImageFile::illustration(illustration.id));

        tokio::fs::try_exists(&path).await.unwrap_or(false)
    }

    /// Converts the downloaded PNG to WebP before saving. Writes nothing if
    /// conversion fails, so the next sync retries the download.
    async fn save_image(&self, image: Image) {
        let Image(id, png) = image;

        let webp = match tokio::task::spawn_blocking(move || png_to_webp(&png)).await {
            Ok(Ok(webp)) => webp,
            Ok(Err(why)) => {
                log::error!("Failed to convert image {id} to WebP: {why}");
                return;
            }
            Err(why) => {
                log::error!("WebP conversion task for image {id} failed: {why}");
                return;
            }
        };

        self.save(&self.path(ImageFile::card(id)), webp).await;
    }

    async fn save_illustration(&self, illustration: Illustration) {
        let Illustration(id, bytes) = illustration;

        if bytes.is_empty() {
            log::error!("Downloaded illustration {id} was empty, not saving");
            return;
        }

        self.save(&self.path(ImageFile::illustration(id)), bytes)
            .await;
    }

    async fn delete_image(&self, id: Uuid) {
        self.delete(ImageFile::card(id)).await;
    }

    async fn delete_illustration(&self, id: Uuid) {
        self.delete(ImageFile::illustration(id)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::utils::webp::fixture_png;
    use std::path::Path;

    fn store(dir: &Path) -> FileSystem {
        std::fs::create_dir_all(dir.join("images")).unwrap();
        std::fs::create_dir_all(dir.join("illustrations")).unwrap();
        FileSystem::from_base_dir(dir.to_str().unwrap())
    }

    #[tokio::test]
    async fn save_image_writes_webp() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();

        store(dir.path()).save_image(Image(id, fixture_png())).await;

        let bytes = std::fs::read(dir.path().join(format!("images/{id}.webp"))).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert!(!dir.path().join(format!("images/{id}.png")).exists());
    }

    #[tokio::test]
    async fn save_image_writes_nothing_for_bad_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let (empty, garbage) = (Uuid::new_v4(), Uuid::new_v4());
        let fs = store(dir.path());

        fs.save_image(Image(empty, vec![])).await;
        fs.save_image(Image(garbage, b"garbage".to_vec())).await;

        assert!(!dir.path().join(format!("images/{empty}.webp")).exists());
        assert!(!dir.path().join(format!("images/{garbage}.webp")).exists());
    }

    #[tokio::test]
    async fn save_illustration_writes_jpg_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();

        store(dir.path())
            .save_illustration(Illustration(id, b"\xFF\xD8jpeg".to_vec()))
            .await;

        let bytes = std::fs::read(dir.path().join(format!("illustrations/{id}.jpg"))).unwrap();
        assert_eq!(bytes, b"\xFF\xD8jpeg");
    }

    #[tokio::test]
    async fn save_illustration_writes_nothing_for_empty_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();

        store(dir.path())
            .save_illustration(Illustration(id, vec![]))
            .await;

        assert!(!dir.path().join(format!("illustrations/{id}.jpg")).exists());
    }

    #[tokio::test]
    async fn delete_removes_new_names() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let fs = store(dir.path());
        std::fs::write(dir.path().join(format!("images/{id}.webp")), b"x").unwrap();
        std::fs::write(dir.path().join(format!("illustrations/{id}.jpg")), b"x").unwrap();

        fs.delete_image(id).await;
        fs.delete_illustration(id).await;

        assert!(!dir.path().join(format!("images/{id}.webp")).exists());
        assert!(!dir.path().join(format!("illustrations/{id}.jpg")).exists());
    }
}

mod file_system;

use crate::adapters::outbound::image_store::file_system::FileSystem;
use crate::ports::outbound::image_store::ImageStore;

#[must_use]
pub fn init_image_store() -> impl ImageStore {
    FileSystem::create()
}

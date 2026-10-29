use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    bytes: Vec<u8>,
}

impl Image {
    #[must_use]
    pub fn new(image: Vec<u8>) -> Self {
        Self { bytes: image }
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

/// Which kind of stored card image a file is. Determines its extension:
/// card images are lossy WebP, illustrations are Scryfall's `art_crop` JPEGs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Card,
    Illustration,
}

impl ImageKind {
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Card => "webp",
            Self::Illustration => "jpg",
        }
    }
}

/// The file name of a stored card image or illustration, used for both the
/// on-disk path under `IMAGES_DIR` and the Discord attachment name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageFile {
    id: Uuid,
    kind: ImageKind,
}

impl ImageFile {
    #[must_use]
    pub const fn card(id: Uuid) -> Self {
        Self {
            id,
            kind: ImageKind::Card,
        }
    }

    #[must_use]
    pub const fn illustration(id: Uuid) -> Self {
        Self {
            id,
            kind: ImageKind::Illustration,
        }
    }

    #[must_use]
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    #[must_use]
    pub fn kind(&self) -> ImageKind {
        self.kind
    }
}

impl fmt::Display for ImageFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.id, self.kind.extension())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    const ID: &str = "550e8400-e29b-41d4-a716-446655440000";

    #[test]
    fn card_file_name_is_webp() {
        let id = Uuid::parse_str(ID).unwrap();
        assert_eq!(ImageFile::card(id).to_string(), format!("{ID}.webp"));
    }

    #[test]
    fn illustration_file_name_is_jpg() {
        let id = Uuid::parse_str(ID).unwrap();
        assert_eq!(ImageFile::illustration(id).to_string(), format!("{ID}.jpg"));
    }

    #[test]
    fn kind_and_id_round_trip() {
        let id = Uuid::parse_str(ID).unwrap();
        let file = ImageFile::illustration(id);
        assert_eq!(file.kind(), ImageKind::Illustration);
        assert_eq!(file.id(), &id);
    }
}

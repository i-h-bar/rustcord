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

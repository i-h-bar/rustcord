#[cfg_attr(test, derive(Clone, PartialEq))]
#[derive(Debug)]
pub struct Image {
    bytes: Vec<u8>,
}

impl Image {
    pub fn new(image: Vec<u8>) -> Self {
        Self { bytes: image }
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

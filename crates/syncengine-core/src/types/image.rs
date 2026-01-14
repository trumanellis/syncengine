//! Card Image Type - Flexible image storage for cards
//!
//! Supports multiple image formats: Iroh blobs, data URIs, and SVG strings.

use serde::{Deserialize, Serialize};

/// Image data for cards (avatars, quest images, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CardImage {
    /// Iroh blob content hash (content-addressed storage)
    BlobId(String),

    /// Base64 data URI (for QR codes, small generated images)
    /// Format: "data:image/png;base64,..."
    DataUri(String),

    /// SVG string (for generated graphics, icons)
    Svg(String),
}

impl CardImage {
    /// Check if image data is available
    pub fn is_available(&self) -> bool {
        match self {
            CardImage::BlobId(id) => !id.is_empty(),
            CardImage::DataUri(uri) => !uri.is_empty(),
            CardImage::Svg(svg) => !svg.is_empty(),
        }
    }

    /// Get a descriptive string for the image type
    pub fn image_type(&self) -> &'static str {
        match self {
            CardImage::BlobId(_) => "blob",
            CardImage::DataUri(_) => "data-uri",
            CardImage::Svg(_) => "svg",
        }
    }

    /// Create from Iroh blob hash
    pub fn from_blob(blob_id: String) -> Self {
        CardImage::BlobId(blob_id)
    }

    /// Create from data URI
    pub fn from_data_uri(uri: String) -> Self {
        CardImage::DataUri(uri)
    }

    /// Create from SVG string
    pub fn from_svg(svg: String) -> Self {
        CardImage::Svg(svg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_id_available() {
        let img = CardImage::BlobId("abc123".to_string());
        assert!(img.is_available());
        assert_eq!(img.image_type(), "blob");
    }

    #[test]
    fn test_empty_blob_not_available() {
        let img = CardImage::BlobId(String::new());
        assert!(!img.is_available());
    }

    #[test]
    fn test_data_uri_available() {
        let img = CardImage::DataUri("data:image/png;base64,iVBORw0KG...".to_string());
        assert!(img.is_available());
        assert_eq!(img.image_type(), "data-uri");
    }

    #[test]
    fn test_svg_available() {
        let img = CardImage::Svg("<svg>...</svg>".to_string());
        assert!(img.is_available());
        assert_eq!(img.image_type(), "svg");
    }

    #[test]
    fn test_from_constructors() {
        let blob = CardImage::from_blob("hash".to_string());
        assert_eq!(blob, CardImage::BlobId("hash".to_string()));

        let uri = CardImage::from_data_uri("data:...".to_string());
        assert_eq!(uri, CardImage::DataUri("data:...".to_string()));

        let svg = CardImage::from_svg("<svg/>".to_string());
        assert_eq!(svg, CardImage::Svg("<svg/>".to_string()));
    }
}

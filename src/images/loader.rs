//! Image cache for synchronous access in VirtualWorld
//!
//! Typst's World::file() trait requires synchronous access to binary data.
//! This module provides an in-memory cache that pre-loads images from IndexedDB.

use std::collections::HashMap;

use typst::foundations::Bytes;

use super::{extension_from_mime_type, ImageError, ImageStore};
use crate::world::VirtualWorld;

/// In-memory cache for images
///
/// Images are loaded asynchronously from IndexedDB and stored here
/// for synchronous access by the Typst compiler via VirtualWorld.
#[derive(Debug, Clone, Default)]
pub struct ImageCache {
    /// Cached image data: image_id -> (bytes, extension)
    images: HashMap<String, (Bytes, String)>,
}

impl ImageCache {
    /// Create a new empty image cache
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    /// Add an image to the cache
    pub fn add(&mut self, id: String, data: Vec<u8>, extension: String) {
        self.images.insert(id, (Bytes::from(data), extension));
    }

    /// Get image data by ID
    pub fn get(&self, id: &str) -> Option<&Bytes> {
        self.images.get(id).map(|(bytes, _)| bytes)
    }

    /// Get image extension by ID
    pub fn get_extension(&self, id: &str) -> Option<&str> {
        self.images.get(id).map(|(_, ext)| ext.as_str())
    }

    /// Check if an image is cached
    pub fn contains(&self, id: &str) -> bool {
        self.images.contains_key(id)
    }

    /// Get the number of cached images
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// Clear all cached images
    pub fn clear(&mut self) {
        self.images.clear();
    }

    /// Get all cached image IDs
    pub fn image_ids(&self) -> Vec<&String> {
        self.images.keys().collect()
    }

    /// Pre-load images from IndexedDB for a set of image IDs
    ///
    /// This should be called before compilation to ensure all
    /// referenced images are available synchronously.
    pub async fn preload_images(
        &mut self,
        store: &ImageStore,
        image_ids: &[String],
    ) -> Result<(), ImageError> {
        for id in image_ids {
            if !self.contains(id) {
                // Get metadata for extension
                let metadata = store.get_metadata(id).await?;
                let ext = extension_from_mime_type(&metadata.mime_type);

                // Get image data
                let data = store.get_image_data(id).await?;

                // Add to cache
                self.add(id.clone(), data, ext.to_string());
            }
        }

        Ok(())
    }

    /// Pre-load all images from IndexedDB
    pub async fn preload_all(&mut self, store: &ImageStore) -> Result<(), ImageError> {
        let images = store.list_images().await?;

        for metadata in images {
            if !self.contains(&metadata.id) {
                let ext = extension_from_mime_type(&metadata.mime_type);
                let data = store.get_image_data(&metadata.id).await?;
                self.add(metadata.id, data, ext.to_string());
            }
        }

        Ok(())
    }

    /// Populate a VirtualWorld with cached images
    ///
    /// Adds all cached images to the VirtualWorld's virtual file system
    /// so they can be accessed by Typst's `#image()` function.
    pub fn populate_world(&self, world: &mut VirtualWorld) {
        for (id, (bytes, ext)) in &self.images {
            let path = format!("{}.{}", id, ext);
            world.add_file(&path, bytes.clone());
        }
    }
}

/// Extract image IDs referenced in a document's JSON data
///
/// Looks for the `images` field in SlickSheetData and returns all referenced IDs.
#[allow(dead_code)]
pub fn extract_image_ids_from_data(data: &crate::data::SlickSheetData) -> Vec<String> {
    data.images.values().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_cache_new() {
        let cache = ImageCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_image_cache_add_and_get() {
        let mut cache = ImageCache::new();
        let data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header bytes

        cache.add("img_123".to_string(), data.clone(), "png".to_string());

        assert!(cache.contains("img_123"));
        assert!(!cache.contains("img_456"));
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get("img_123").unwrap();
        assert_eq!(retrieved.as_slice(), &data);

        assert_eq!(cache.get_extension("img_123"), Some("png"));
    }

    #[test]
    fn test_image_cache_clear() {
        let mut cache = ImageCache::new();
        cache.add("img_1".to_string(), vec![1, 2, 3], "png".to_string());
        cache.add("img_2".to_string(), vec![4, 5, 6], "jpg".to_string());

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert!(cache.is_empty());
        assert!(!cache.contains("img_1"));
        assert!(!cache.contains("img_2"));
    }

    #[test]
    fn test_image_ids() {
        let mut cache = ImageCache::new();
        cache.add("img_aaa".to_string(), vec![1], "png".to_string());
        cache.add("img_bbb".to_string(), vec![2], "jpg".to_string());

        let ids = cache.image_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&&"img_aaa".to_string()));
        assert!(ids.contains(&&"img_bbb".to_string()));
    }

    #[test]
    fn test_extract_image_ids_from_data() {
        use std::collections::HashMap;

        let mut images = HashMap::new();
        images.insert("logo".to_string(), "img_123".to_string());
        images.insert("banner".to_string(), "img_456".to_string());

        let data = crate::data::SlickSheetData {
            images,
            ..Default::default()
        };

        let ids = extract_image_ids_from_data(&data);
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"img_123".to_string()));
        assert!(ids.contains(&"img_456".to_string()));
    }
}

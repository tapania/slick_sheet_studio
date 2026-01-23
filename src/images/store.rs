//! IndexedDB-based image storage
//!
//! Provides persistent storage for images in the browser using IndexedDB.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{IdbDatabase, IdbRequest, IdbTransaction};

use super::{
    detect_mime_type, extension_from_mime_type, generate_image_id, is_supported_mime_type,
    ImageError, ImageMetadata, MAX_IMAGE_SIZE,
};

/// Database name for image storage
const DB_NAME: &str = "slick_sheet_images";
/// Database version
const DB_VERSION: u32 = 1;
/// Object store for image metadata
const METADATA_STORE: &str = "metadata";
/// Object store for image binary data
const DATA_STORE: &str = "data";

/// Image store backed by IndexedDB
#[derive(Clone)]
pub struct ImageStore {
    db: IdbDatabase,
}

impl ImageStore {
    /// Open or create the image store database
    pub async fn open() -> Result<Self, ImageError> {
        let window = web_sys::window()
            .ok_or_else(|| ImageError::StorageError("No window object".to_string()))?;

        let indexed_db = window
            .indexed_db()
            .map_err(|e| ImageError::StorageError(format!("IndexedDB not available: {:?}", e)))?
            .ok_or_else(|| ImageError::StorageError("IndexedDB is null".to_string()))?;

        // Open database with version upgrade handler
        let open_request = indexed_db
            .open_with_u32(DB_NAME, DB_VERSION)
            .map_err(|e| ImageError::StorageError(format!("Failed to open database: {:?}", e)))?;

        // Set up upgrade handler to create object stores
        let on_upgrade = Closure::once(Box::new(move |event: web_sys::IdbVersionChangeEvent| {
            let target = event.target().unwrap();
            let request: IdbRequest = target.unchecked_into();
            let db: IdbDatabase = request.result().unwrap().unchecked_into();

            // Check if stores exist using DomStringList::contains
            let store_names = db.object_store_names();

            // Create metadata store if it doesn't exist
            if !store_names.contains(METADATA_STORE) {
                let _ = db.create_object_store(METADATA_STORE);
            }

            // Create data store if it doesn't exist
            if !store_names.contains(DATA_STORE) {
                let _ = db.create_object_store(DATA_STORE);
            }
        }) as Box<dyn FnOnce(_)>);

        open_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        on_upgrade.forget();

        // Wait for database to open
        let db = wait_for_request(&open_request).await?;
        let db: IdbDatabase = db.unchecked_into();

        Ok(Self { db })
    }

    /// Store a new image
    ///
    /// Returns the generated image ID and metadata
    pub async fn store_image(
        &self,
        filename: String,
        data: Vec<u8>,
    ) -> Result<ImageMetadata, ImageError> {
        // Validate size
        if data.len() > MAX_IMAGE_SIZE {
            return Err(ImageError::FileTooLarge(data.len()));
        }

        // Detect MIME type from bytes
        let mime_type = detect_mime_type(&data)
            .ok_or_else(|| ImageError::UnsupportedFormat("Unknown format".to_string()))?;

        // Validate MIME type
        if !is_supported_mime_type(mime_type) {
            return Err(ImageError::UnsupportedFormat(mime_type.to_string()));
        }

        // Generate unique ID
        let id = generate_image_id();

        // Create metadata
        let metadata = ImageMetadata::new(id.clone(), filename, mime_type.to_string(), data.len());

        // Start transaction
        let transaction = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of2(&METADATA_STORE.into(), &DATA_STORE.into()),
                web_sys::IdbTransactionMode::Readwrite,
            )
            .map_err(|e| ImageError::StorageError(format!("Transaction failed: {:?}", e)))?;

        // Store metadata
        let metadata_store = transaction
            .object_store(METADATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Store access failed: {:?}", e)))?;

        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| ImageError::StorageError(format!("Serialization failed: {}", e)))?;

        let put_metadata = metadata_store
            .put_with_key(&JsValue::from_str(&metadata_json), &JsValue::from_str(&id))
            .map_err(|e| ImageError::StorageError(format!("Put metadata failed: {:?}", e)))?;

        wait_for_request(&put_metadata).await?;

        // Store binary data
        let data_store = transaction
            .object_store(DATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Store access failed: {:?}", e)))?;

        let uint8_array = js_sys::Uint8Array::from(data.as_slice());
        let put_data = data_store
            .put_with_key(&uint8_array, &JsValue::from_str(&id))
            .map_err(|e| ImageError::StorageError(format!("Put data failed: {:?}", e)))?;

        wait_for_request(&put_data).await?;

        // Wait for transaction to complete
        wait_for_transaction(&transaction).await?;

        Ok(metadata)
    }

    /// Store a new AI-generated image with prompt and alt description
    ///
    /// Returns the generated image ID and metadata
    pub async fn store_generated_image(
        &self,
        filename: String,
        data: Vec<u8>,
        generation_prompt: String,
        alt_description: String,
    ) -> Result<ImageMetadata, ImageError> {
        // Validate size
        if data.len() > MAX_IMAGE_SIZE {
            return Err(ImageError::FileTooLarge(data.len()));
        }

        // Detect MIME type from bytes
        let mime_type = detect_mime_type(&data)
            .ok_or_else(|| ImageError::UnsupportedFormat("Unknown format".to_string()))?;

        // Validate MIME type
        if !is_supported_mime_type(mime_type) {
            return Err(ImageError::UnsupportedFormat(mime_type.to_string()));
        }

        // Generate unique ID
        let id = generate_image_id();

        // Create metadata with generation info
        let metadata = ImageMetadata::new_generated(
            id.clone(),
            filename,
            mime_type.to_string(),
            data.len(),
            generation_prompt,
            alt_description,
        );

        // Start transaction
        let transaction = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of2(&METADATA_STORE.into(), &DATA_STORE.into()),
                web_sys::IdbTransactionMode::Readwrite,
            )
            .map_err(|e| ImageError::StorageError(format!("Transaction failed: {:?}", e)))?;

        // Store metadata
        let metadata_store = transaction
            .object_store(METADATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Store access failed: {:?}", e)))?;

        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| ImageError::StorageError(format!("Serialization failed: {}", e)))?;

        let put_metadata = metadata_store
            .put_with_key(&JsValue::from_str(&metadata_json), &JsValue::from_str(&id))
            .map_err(|e| ImageError::StorageError(format!("Put metadata failed: {:?}", e)))?;

        wait_for_request(&put_metadata).await?;

        // Store binary data
        let data_store = transaction
            .object_store(DATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Store access failed: {:?}", e)))?;

        let uint8_array = js_sys::Uint8Array::from(data.as_slice());
        let put_data = data_store
            .put_with_key(&uint8_array, &JsValue::from_str(&id))
            .map_err(|e| ImageError::StorageError(format!("Put data failed: {:?}", e)))?;

        wait_for_request(&put_data).await?;

        // Wait for transaction to complete
        wait_for_transaction(&transaction).await?;

        Ok(metadata)
    }

    /// Get image binary data by ID
    pub async fn get_image_data(&self, id: &str) -> Result<Vec<u8>, ImageError> {
        let transaction = self
            .db
            .transaction_with_str(DATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Transaction failed: {:?}", e)))?;

        let store = transaction
            .object_store(DATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Store access failed: {:?}", e)))?;

        let request = store
            .get(&JsValue::from_str(id))
            .map_err(|e| ImageError::StorageError(format!("Get failed: {:?}", e)))?;

        let result = wait_for_request(&request).await?;

        if result.is_undefined() || result.is_null() {
            return Err(ImageError::NotFound(id.to_string()));
        }

        let uint8_array: js_sys::Uint8Array = result.unchecked_into();
        Ok(uint8_array.to_vec())
    }

    /// Get image metadata by ID
    pub async fn get_metadata(&self, id: &str) -> Result<ImageMetadata, ImageError> {
        let transaction = self
            .db
            .transaction_with_str(METADATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Transaction failed: {:?}", e)))?;

        let store = transaction
            .object_store(METADATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Store access failed: {:?}", e)))?;

        let request = store
            .get(&JsValue::from_str(id))
            .map_err(|e| ImageError::StorageError(format!("Get failed: {:?}", e)))?;

        let result = wait_for_request(&request).await?;

        if result.is_undefined() || result.is_null() {
            return Err(ImageError::NotFound(id.to_string()));
        }

        let json_str = result
            .as_string()
            .ok_or_else(|| ImageError::InvalidData("Metadata is not a string".to_string()))?;

        serde_json::from_str(&json_str)
            .map_err(|e| ImageError::InvalidData(format!("Invalid metadata JSON: {}", e)))
    }

    /// List all stored images
    pub async fn list_images(&self) -> Result<Vec<ImageMetadata>, ImageError> {
        let transaction = self
            .db
            .transaction_with_str(METADATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Transaction failed: {:?}", e)))?;

        let store = transaction
            .object_store(METADATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Store access failed: {:?}", e)))?;

        let request = store
            .get_all()
            .map_err(|e| ImageError::StorageError(format!("Get all failed: {:?}", e)))?;

        let result = wait_for_request(&request).await?;

        let array: js_sys::Array = result.unchecked_into();
        let mut images = Vec::new();

        for i in 0..array.length() {
            let item = array.get(i);
            if let Some(json_str) = item.as_string() {
                if let Ok(metadata) = serde_json::from_str::<ImageMetadata>(&json_str) {
                    images.push(metadata);
                }
            }
        }

        // Sort by creation date (newest first)
        images.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(images)
    }

    /// Delete an image by ID
    pub async fn delete_image(&self, id: &str) -> Result<(), ImageError> {
        let transaction = self
            .db
            .transaction_with_str_sequence_and_mode(
                &js_sys::Array::of2(&METADATA_STORE.into(), &DATA_STORE.into()),
                web_sys::IdbTransactionMode::Readwrite,
            )
            .map_err(|e| ImageError::StorageError(format!("Transaction failed: {:?}", e)))?;

        // Delete metadata
        let metadata_store = transaction
            .object_store(METADATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Store access failed: {:?}", e)))?;

        let delete_metadata = metadata_store
            .delete(&JsValue::from_str(id))
            .map_err(|e| ImageError::StorageError(format!("Delete metadata failed: {:?}", e)))?;

        wait_for_request(&delete_metadata).await?;

        // Delete data
        let data_store = transaction
            .object_store(DATA_STORE)
            .map_err(|e| ImageError::StorageError(format!("Store access failed: {:?}", e)))?;

        let delete_data = data_store
            .delete(&JsValue::from_str(id))
            .map_err(|e| ImageError::StorageError(format!("Delete data failed: {:?}", e)))?;

        wait_for_request(&delete_data).await?;

        // Wait for transaction to complete
        wait_for_transaction(&transaction).await?;

        Ok(())
    }

    /// Get the file path for an image ID (used in Typst templates)
    ///
    /// Returns the path with extension based on MIME type
    pub fn get_image_path(&self, id: &str, mime_type: &str) -> String {
        let ext = extension_from_mime_type(mime_type);
        format!("{}.{}", id, ext)
    }
}

/// Wait for an IDB request to complete
async fn wait_for_request(request: &IdbRequest) -> Result<JsValue, ImageError> {
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let resolve_clone = resolve.clone();
        let reject_clone = reject.clone();

        let onsuccess = Closure::once(Box::new(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let request: IdbRequest = target.unchecked_into();
            let result = request.result().unwrap_or(JsValue::UNDEFINED);
            resolve_clone.call1(&JsValue::UNDEFINED, &result).unwrap();
        }) as Box<dyn FnOnce(_)>);

        let onerror = Closure::once(Box::new(move |_event: web_sys::Event| {
            // Simply report a generic error - the actual error is hard to extract
            reject_clone
                .call1(
                    &JsValue::UNDEFINED,
                    &JsValue::from_str("IndexedDB request failed"),
                )
                .unwrap();
        }) as Box<dyn FnOnce(_)>);

        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        request.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onsuccess.forget();
        onerror.forget();
    });

    JsFuture::from(promise)
        .await
        .map_err(|e| ImageError::StorageError(format!("Request failed: {:?}", e)))
}

/// Wait for an IDB transaction to complete
async fn wait_for_transaction(transaction: &IdbTransaction) -> Result<(), ImageError> {
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let resolve_clone = resolve.clone();
        let reject_clone = reject.clone();

        let oncomplete = Closure::once(Box::new(move |_event: web_sys::Event| {
            resolve_clone.call0(&JsValue::UNDEFINED).unwrap();
        }) as Box<dyn FnOnce(_)>);

        let onerror = Closure::once(Box::new(move |_event: web_sys::Event| {
            reject_clone
                .call1(
                    &JsValue::UNDEFINED,
                    &JsValue::from_str("Transaction failed"),
                )
                .unwrap();
        }) as Box<dyn FnOnce(_)>);

        transaction.set_oncomplete(Some(oncomplete.as_ref().unchecked_ref()));
        transaction.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        oncomplete.forget();
        onerror.forget();
    });

    JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(|e| ImageError::StorageError(format!("Transaction failed: {:?}", e)))
}

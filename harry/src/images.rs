//! Pictures the app shows in components (an image slot of a list item): registered once under a key and
//! drawn by harry from the renderer's raw image storage. The app hands `storage()` to its
//! renderer (`RendererHelper::initialize_renderer`: `renderer.replace_raw_image_storage`) and
//! registers its pictures (`register_encoded` for a PNG or JPEG, `register_rgba` for decoded
//! pixels); a component names a picture by its key and the glue sets it on the part
//! (`ImageContentData`, harry's image customization of a node, per row of a list).
//!
//! A key registered again gets its new picture in place (the renderer's storage updates the
//! resource, as for camera frames); `unregister` frees a key, its slot is reused by the next new
//! key: the storage has no removal, so the pictures of a feed that changes every few minutes
//! stay bounded by the pictures shown at once.

use customizations::ImageContentData;
use har_platform_api::resource_manager::ResourceManager;
use har_types::resources::{RawImageData, ResourceToken};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

struct Store {
    manager: ResourceManager,
    images: HashMap<String, ImageContentData>,
    /// Slots of unregistered keys, for the next new key.
    free: Vec<ResourceToken>,
}

fn store() -> &'static Mutex<Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(Store { manager: ResourceManager::new(), images: HashMap::new(), free: Vec::new() }))
}

/// The raw image storage of the pictures, for the renderer (a clone shares the storage).
pub fn storage() -> ResourceManager {
    store().lock().expect("image store").manager.clone()
}

/// Registers decoded RGBA pixels (`width` x `height`, 4 bytes a pixel) under `key`; a key
/// registered again gets the new picture in its slot.
pub fn register_rgba(key: impl Into<String>, rgba: Vec<u8>, width: u32, height: u32) -> Result<(), String> {
    if rgba.len() != (width as usize) * (height as usize) * 4 {
        return Err(format!("{} bytes for {width}x{height} RGBA", rgba.len()));
    }
    let key = key.into();
    let mut store = store().lock().map_err(|e| e.to_string())?;
    let data = RawImageData::new(rgba, width, height);
    let slot = store.images.get(&key).map(|i| i.id).or_else(|| store.free.pop());
    let token = match slot {
        Some(token) => {
            store.manager.update_resource(token, data).map_err(|e| format!("{e:?}"))?;
            token
        }
        None => store.manager.add_resource(data)?,
    };
    store.images.insert(key, ImageContentData::new(token, width, height));
    Ok(())
}

/// Registers RGB pixels (3 bytes a pixel), as RGBA.
pub fn register_rgb(key: impl Into<String>, rgb: &[u8], width: u32, height: u32) -> Result<(), String> {
    if rgb.len() != (width as usize) * (height as usize) * 3 {
        return Err(format!("{} bytes for {width}x{height} RGB", rgb.len()));
    }
    let rgba = rgb.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect();
    register_rgba(key, rgba, width, height)
}

/// Frees `key`: components naming it show their placeholder, its slot goes to the next new key.
pub fn unregister(key: &str) {
    if let Ok(mut store) = store().lock() {
        if let Some(image) = store.images.remove(key) {
            store.free.push(image.id);
        }
    }
}

/// The keys registered with `prefix`.
pub fn keys(prefix: &str) -> Vec<String> {
    store().lock().map(|s| s.images.keys().filter(|k| k.starts_with(prefix)).cloned().collect()).unwrap_or_default()
}

/// Registers an encoded picture (PNG, JPEG) under `key`.
pub fn register_encoded(key: impl Into<String>, bytes: &[u8]) -> Result<(), String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?.to_rgba8();
    let (width, height) = img.dimensions();
    register_rgba(key, img.into_raw(), width, height)
}

/// The picture registered under `key`.
pub fn get(key: &str) -> Option<ImageContentData> {
    store().lock().ok()?.images.get(key).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_registered_picture_is_found_by_its_key() {
        register_rgba("test/red", vec![255, 0, 0, 255].repeat(4), 2, 2).unwrap();
        let img = get("test/red").unwrap();
        assert_eq!((img.width, img.height), (2, 2));
        assert!(get("test/none").is_none());
        assert!(register_rgba("test/bad", vec![0; 3], 1, 1).is_err());
    }

    #[test]
    fn a_freed_slot_is_reused_and_a_key_updates_in_place() {
        register_rgb("slot/a", &[1, 2, 3], 1, 1).unwrap();
        let a = get("slot/a").unwrap().id;
        register_rgb("slot/a", &[4, 5, 6, 7, 8, 9], 2, 1).unwrap();
        assert_eq!((get("slot/a").unwrap().id, get("slot/a").unwrap().width), (a, 2));
        unregister("slot/a");
        assert!(get("slot/a").is_none());
        register_rgb("slot/b", &[1, 2, 3], 1, 1).unwrap();
        assert_eq!(get("slot/b").unwrap().id, a);
        assert_eq!(keys("slot/"), vec!["slot/b".to_string()]);
    }
}

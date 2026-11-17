//! Pictures the app shows in components (an image slot of a list item): registered once under a key and
//! drawn by harry from the renderer's raw image storage. The app hands `storage()` to its
//! renderer (`RendererHelper::initialize_renderer`: `renderer.replace_raw_image_storage`) and
//! registers its pictures (`register_encoded` for a PNG or JPEG, `register_rgba` for decoded
//! pixels); a component names a picture by its key and the glue sets it on the part
//! (`ImageContentData`, harry's image customization of a node, per row of a list).
//!
//! A new key takes a new slot of the storage; the renderer uploads it at its first frame. A key
//! registered again, and a new key in the slot of a freed one (`unregister`: the storage has no
//! removal, so the pictures of a feed that changes every few minutes stay bounded by the pictures
//! shown at once), change a picture the renderer may already have on the GPU: it uploads it
//! again only when the looper tells it, as for camera frames. So the app hands the looper's
//! resource client to `set_client` at its start (`create_resource_manager_client(looper_tx)`),
//! and a changed picture goes to the renderer through it (`ResourceManagerClient::update`, the
//! renderer marks the texture dirty). Without a client (a host test) the storage is written
//! directly, and a texture the renderer already has stays as it was.

use customizations::ImageContentData;
use har_platform_api::looper::ResourceManagerClient;
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

/// The looper's resource client, for a picture that changes in a slot the renderer has.
static CLIENT: Mutex<Option<ResourceManagerClient>> = Mutex::new(None);

/// Hands the looper's resource client over (the app, at its start: a changed picture reaches
/// the renderer only through it).
pub fn set_client(client: ResourceManagerClient) {
    if let Ok(mut c) = CLIENT.lock() {
        *c = Some(client);
    }
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
        // a slot the renderer may hold a texture of: the looper updates it and marks it dirty
        Some(token) => {
            match CLIENT.lock().map_err(|e| e.to_string())?.as_ref() {
                Some(client) => client.update(token, data).map_err(|e| format!("{e:?}"))?,
                None => store.manager.update_resource(token, data).map_err(|e| format!("{e:?}"))?,
            }
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

/// A solid colour (0xRRGGBB) as a picture of one pixel, for a part whose colour the code sets:
/// harry has no colour to set on a node, it draws a picture stretched over a plain frame.
/// Registered at the first use under `color/<rrggbb>` and kept, so the colours come from a
/// palette (a part's few colours), not from free data.
pub fn color(rgb: u32) -> Option<ImageContentData> {
    let key = format!("color/{:06x}", rgb & 0xff_ffff);
    if let Some(image) = get(&key) {
        return Some(image);
    }
    let [_, r, g, b] = rgb.to_be_bytes();
    register_rgba(key.as_str(), vec![r, g, b, 255], 1, 1).ok()?;
    get(&key)
}

/// Tests that register new keys while another expects the next free slot run one at a time
/// (the store is the process's, the tests run in threads).
#[cfg(test)]
pub(crate) static TEST_STORE: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_registered_picture_is_found_by_its_key() {
        let _one = TEST_STORE.lock().unwrap_or_else(|e| e.into_inner());
        register_rgba("test/red", vec![255, 0, 0, 255].repeat(4), 2, 2).unwrap();
        let img = get("test/red").unwrap();
        assert_eq!((img.width, img.height), (2, 2));
        assert!(get("test/none").is_none());
        assert!(register_rgba("test/bad", vec![0; 3], 1, 1).is_err());
    }

    #[test]
    fn a_colour_is_one_pixel_registered_once() {
        let _one = TEST_STORE.lock().unwrap_or_else(|e| e.into_inner());
        let a = color(0x33b679).unwrap();
        assert_eq!((a.width, a.height), (1, 1));
        assert_eq!(color(0xff33_b679).unwrap().id, a.id, "the same colour, the same picture");
        assert_ne!(color(0x039be5).unwrap().id, a.id);
        assert!(keys("color/").contains(&"color/33b679".to_string()));
    }

    #[test]
    fn a_changed_picture_goes_to_the_renderer_through_the_looper() {
        use har_platform_api::looper::{create_resource_manager_client, LooperMessage};
        use har_types::resources::ResourceManagerMessage;
        let _one = TEST_STORE.lock().unwrap_or_else(|e| e.into_inner());
        let (tx, rx) = std::sync::mpsc::channel();
        set_client(create_resource_manager_client(tx));
        // a new key: a new slot, the renderer uploads it at its first frame
        register_rgb("client/a", &[1, 2, 3], 1, 1).unwrap();
        assert!(rx.try_recv().is_err(), "nothing to tell the looper");
        let token = get("client/a").unwrap().id;
        // the same key again, same size: the looper gets the new pixels for the slot
        register_rgb("client/a", &[4, 5, 6], 1, 1).unwrap();
        match rx.try_recv() {
            Ok(LooperMessage::ResourceManagerMessage(ResourceManagerMessage::Update(t, data))) => {
                assert_eq!(t, token);
                assert_eq!(data, RawImageData::new(vec![4, 5, 6, 255], 1, 1));
            }
            other => panic!("expected an update of the slot, got {}", if other.is_ok() { "another message" } else { "none" }),
        }
        // a new key in a freed slot is a change of that slot too
        unregister("client/a");
        register_rgb("client/b", &[7, 8, 9], 1, 1).unwrap();
        assert!(matches!(rx.try_recv(), Ok(LooperMessage::ResourceManagerMessage(ResourceManagerMessage::Update(t, _))) if t == token));
        unregister("client/b");
        *CLIENT.lock().unwrap() = None;
    }

    #[test]
    fn a_freed_slot_is_reused_and_a_key_updates_in_place() {
        let _one = TEST_STORE.lock().unwrap_or_else(|e| e.into_inner());
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

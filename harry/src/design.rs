//! The design's sizes, read from the app's .dcf. A sheet's card hugs its content (Figma auto
//! layout, harry lays it out the same), and the app slides the card to the middle of the screen
//! with a position override, so it needs the content's height. harry does not hand layout
//! results to the app; the design does: the .dcf the app ships holds every node's size, and this
//! reads the variants of the component sets once at the start (a few hundred ms for a 13 MB
//! document).

use dc_bundle::definition_file::load_design_def;
use dc_bundle::view::view_data::View_data_type;
use dc_bundle::view::View;
use std::collections::HashMap;
use std::path::Path;

/// The file of a design document in an app's data folder: `<id>.dcf`; an id that carries the
/// extension already is the file name as it is. harry opens `<folder>/<document id>` without
/// adding anything, so an app hands it this name (`--document-id tv.dcf`, the default of the
/// apps) and uses the same name for what it reads from the file itself (`Sizes`).
pub fn document_file(id: &str) -> String {
    if id.ends_with(".dcf") {
        id.to_string()
    } else {
        format!("{id}.dcf")
    }
}

/// Width and height of the variants of the design's component sets, by `set/variant`
/// (`#wifi-content/step=networks`), and the frame of every `#named` node of a screen in its root
/// frame's coordinates (the screen's pixels): where the design puts it.
#[derive(Clone, Debug, Default)]
pub struct Sizes {
    variants: HashMap<String, (f32, f32)>,
    frames: HashMap<String, Frame>,
}

/// A node's place in its screen as the design draws it: x, y, width, height in pixels from
/// the root frame's top left. harry lays an auto layout out again at run time; for a node with a
/// fixed place (a slot in a tile) the two agree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Sizes {
    /// Reads the .dcf; an error names the file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let (_, doc) = load_design_def(path).map_err(|e| format!("{}: {e:?}", path.display()))?;
        let mut sizes = Sizes::default();
        for view in doc.views.values() {
            sizes.collect(view);
            // the root's own place on the design's page is the origin of its screen
            if let Some(origin) = view.design_absolute_bounding_box.as_ref() {
                sizes.place(view, (origin.x.unwrap_or(0.0), origin.y.unwrap_or(0.0)));
            }
        }
        Ok(sizes)
    }

    /// No design: every lookup is None (the sheets keep their default geometry).
    pub fn none() -> Self {
        Sizes::default()
    }

    /// A component set is a container of variants, named `prop=value, prop=value`.
    fn collect(&mut self, view: &View) {
        let Some(View_data_type::Container(c)) = view.data.as_ref().and_then(|d| d.view_data_type.as_ref()) else {
            return;
        };
        let is_set = !c.children.is_empty() && c.children.iter().all(|v| v.name.contains('='));
        for child in &c.children {
            if is_set {
                if let Ok(size) = child.style().layout_style().bounding_box() {
                    self.variants.insert(format!("{}/{}", view.name, child.name), (size.width, size.height));
                }
            }
            self.collect(child);
        }
    }

    /// The frames of a screen's #named nodes; a component set's variants are the Library's,
    /// not a place on a screen, and their names repeat: left out.
    fn place(&mut self, view: &View, origin: (f32, f32)) {
        if view.name.starts_with('#') {
            if let Some(b) = view.design_absolute_bounding_box.as_ref() {
                self.frames.entry(view.name.clone()).or_insert(Frame {
                    x: b.x.unwrap_or(0.0) - origin.0,
                    y: b.y.unwrap_or(0.0) - origin.1,
                    width: b.width.unwrap_or(0.0),
                    height: b.height.unwrap_or(0.0),
                });
            }
        }
        let Some(View_data_type::Container(c)) = view.data.as_ref().and_then(|d| d.view_data_type.as_ref()) else {
            return;
        };
        if !c.children.is_empty() && c.children.iter().all(|v| v.name.contains('=')) {
            return;
        }
        for child in &c.children {
            self.place(child, origin);
        }
    }

    /// Where the design puts the node `name` on its screen (a slot: the place of the
    /// component in it), if the design has the node.
    pub fn frame(&self, name: &str) -> Option<Frame> {
        self.frames.get(name).copied()
    }

    /// The size of a variant of a set, if the design has it.
    pub fn variant(&self, set: &str, variant: &str) -> Option<(f32, f32)> {
        self.variants.get(&format!("{set}/{variant}")).copied()
    }

    /// The height of a variant of a set.
    pub fn height(&self, set: &str, variant: &str) -> Option<f32> {
        self.variant(set, variant).map(|(_, h)| h)
    }

    /// How many variants were read.
    pub fn len(&self) -> usize {
        self.variants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.variants.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_document_file_carries_the_extension_once() {
        assert_eq!(document_file("tv"), "tv.dcf");
        assert_eq!(document_file("tv.dcf"), "tv.dcf");
    }

    /// The Library's own .dcf (data/library.dcf, design/build-library.sh --dcf): LIBRARY_DCF, else
    /// the copy next to the test (Soong data).
    fn dcf() -> std::path::PathBuf {
        if let Ok(path) = std::env::var("LIBRARY_DCF") {
            return path.into();
        }
        let dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let path = dir.join("data/library.dcf");
        assert!(path.exists(), "{} missing; set LIBRARY_DCF", path.display());
        path
    }

    #[test]
    fn the_variants_sizes_come_from_the_design() {
        let sizes = Sizes::load(dcf()).expect("library.dcf");
        assert!(sizes.len() > 50, "{} variants", sizes.len());
        // the button: the compact size is the narrower one (the design's widths 200 and 116)
        let regular = sizes.variant("#button", "kind=primary, state=default, size=regular").expect("#button regular");
        let compact = sizes.variant("#button", "kind=primary, state=default, size=compact").expect("#button compact");
        assert!(compact.0 < regular.0, "compact {compact:?} regular {regular:?}");
        assert_eq!(sizes.height("#button", "kind=primary, state=default, size=regular"), Some(regular.1));
        assert_eq!(sizes.height("#nothing", "state=default"), None);
    }

    #[test]
    fn a_set_has_its_place_in_the_library() {
        let sizes = Sizes::load(dcf()).expect("library.dcf");
        // the root is the Library frame: a set's frame is its place in it, inside the frame, and
        // it holds the set's variants, so it is at least as wide as the regular button
        let f = sizes.frame("#button").expect("#button");
        let (regular_width, _) =
            sizes.variant("#button", "kind=primary, state=default, size=regular").expect("#button regular");
        assert!(f.x >= 0.0 && f.y >= 0.0, "inside the Library frame: {f:?}");
        assert!(f.width >= regular_width && f.height > 0.0, "holds its variants: {f:?}, regular {regular_width}");
        assert_eq!(sizes.frame("#nothing"), None);
    }
}

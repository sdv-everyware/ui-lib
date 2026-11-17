//! Slots: a library component placed on a screen, as harry's list of one item.
//!
//! harry looks customizations up by node name, and the parts of a component (`#toggle/thumb`)
//! have the same name in every instance. A list item has its own customizations for its whole
//! subtree (pre_renderer/generation.rs and layout.rs, `get_customized_children`), so a
//! component cloned into a list is customized on its own: two toggles on one screen move
//! their thumbs independently. This is the mechanism of harry's list-test-app and
//! gui-playground-app; nothing in harry changes.
//!
//! ```text
//! #wifi/reveal                    slot frame in Figma (a preview instance inside)
//!  └─ #wifi/reveal@$List-c        clone of #toggle "state=default", made by harry
//!      ├─ #toggle/on              opacity of THIS item
//!      └─ #toggle/thumb           meter value of THIS item
//! ```
//!
//! In the UI model: `#[Design(node = "#wifi/reveal", customizer = "SlotCustomizer")] reveal: Slot`
//! and in the presenter `model.reveal = slot("#wifi/reveal", &state.reveal)`.
//!
//! - The clone root is interactive; harry reports taps by its name, which `Input` maps back
//!   to the slot name. A control inside a clone (a tile's toggle) is interactive too: harry
//!   reports it first, the clone root next, and `Input` pairs the two into "<slot>/<row>/<part>"
//!   (`row_of`, `part_of`).
//! - Meter value and opacity of a list item only change the display hash
//!   (pre_render_cache.rs), so a moving thumb or a fading list skips the tree rebuild.
//! - The variant is the clone's node selector: a state change (pressed) rebuilds the tree,
//!   as a variant change does.
//! - Texts of the component come from code, not from overrides of the Figma instance.

use crate::{animation, Follow, Meter};
use customizations::render_customizations::{
    CustomizableListItem, List, ListCustomization, ListItem, NodePositionOverride, NodeSelector,
    RenderCustomization, RenderCustomizations,
};
use customizations::{AnimationOverride, Customizer, CustomizerOutput, ImageContentData};
use everyware_ui_core::bar_chart::BarChart;
use everyware_ui_core::button::{Button, IconButton};
use everyware_ui_core::dropdown::Dropdown;
use everyware_ui_core::input_field::{caret_phase_on, InputField, BLINK_MS};
use everyware_ui_core::keyboard::Keyboard;
use everyware_ui_core::list_item::{ListItem as ListRow, ListItemKind, ListItemLines, GLYPHS};
use everyware_ui_core::list_view::Row;
use everyware_ui_core::list_view::{ListView, Scroll};
use everyware_ui_core::radio::Radio;
use everyware_ui_core::segmented::Segmented;
use everyware_ui_core::tabs::Tabs;
use everyware_ui_core::slider::Slider;
use everyware_ui_core::skeleton::Skeleton;
use everyware_ui_core::symbol::SymbolIcon;
use everyware_ui_core::tile::{Tile, TileKind};
use everyware_ui_core::symbol::Symbol;
use everyware_ui_core::toggle::Toggle;
use everyware_ui_core::pressable::Pressable;
use everyware_ui_core::transitions::{self, Change, ComponentKind};
use everyware_ui_core::Component;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// The id of the one item of a slot; harry names the clone "<slot>@$List-<id>".
const ITEM_ID: &str = "c";
/// The id prefix of the rows of a list slot (`rows`) and the cells of a grid: "r0", "r1", ...
const ROW_ID: &str = "r";
/// The id prefix of the rows of a scrolling list (`list`): "s0", "s1", ... A row scrolled partly
/// out of its slot reaches over what lies beside the list (harry hits by bounds, without
/// clipping), so `Input` takes a touch on one only where its slot is under the finger too
/// (`list_row_slot`).
const LIST_ROW_ID: &str = "s";
/// How far a ribbon (a horizontal list) lifts its chosen card, in pixels; its slot is that
/// much higher than a card.
pub const RIBBON_LIFT: f32 = 20.0;

/// What harry puts between the list name and the item id in the clone's name.
const CLONE_MARK: &str = "@$List-";
/// A sweeping part (a skeleton's shimmer) moves this often: 30 frames a second, not every
/// frame, because each frame redraws the whole screen.
const SWEEP_FRAME: Duration = Duration::from_millis(33);
/// The Library's #scrollbar: as high as the screen, the thumb this high (components-patch.js
/// SB_MAX, SB_THUMB; the contract test checks the height against library.dcf).
pub const SCROLLBAR_MAX: f32 = 1080.0;
pub const SCROLLBAR_THUMB: f32 = 48.0;

/// The thumb's meter (0..100 along the whole component) for a list `view_h` high scrolled to
/// `scroll` of `max`: the component's bottom is the list's bottom, the track the lowest
/// `view_h` of it, and the thumb runs from the track's top to its bottom.
fn thumb_meter(scroll: f32, max: f32, view_h: f32) -> f32 {
    let along = if max > 0.0 { (scroll / max).clamp(0.0, 1.0) } else { 0.0 };
    let top = SCROLLBAR_MAX - view_h.min(SCROLLBAR_MAX);
    let y = top + along * (view_h.min(SCROLLBAR_MAX) - SCROLLBAR_THUMB).max(0.0);
    // a list lower than the thumb: the thumb at the component's bottom, not past it
    (y / (SCROLLBAR_MAX - SCROLLBAR_THUMB) * 100.0).clamp(0.0, 100.0)
}

/// A UI model field for a slot: a list with the slot's component.
pub type Slot = List<ComponentItem>;

/// The node name of the clone in `slot`.
pub fn clone_name(slot: &str) -> String {
    format!("{slot}{CLONE_MARK}{ITEM_ID}")
}

/// The node name of row `row` of the fixed list (`rows`) or the grid in `slot`.
pub fn row_name(slot: &str, row: usize) -> String {
    format!("{slot}{CLONE_MARK}{ROW_ID}{row}")
}

/// The node name of row `row` of the scrolling list (`list`) in `slot`.
pub fn list_row_name(slot: &str, row: usize) -> String {
    format!("{slot}{CLONE_MARK}{LIST_ROW_ID}{row}")
}

/// The slot of a row of a scrolling list, for a node name harry reports ("<slot>@$List-s<row>");
/// None for every other node.
pub(crate) fn list_row_slot(node: &str) -> Option<&str> {
    let (slot, id) = node.split_once(CLONE_MARK)?;
    id.strip_prefix(LIST_ROW_ID)?.parse::<usize>().ok().map(|_| slot)
}

/// The row of the list in `slot` a gesture's node names ("<slot>/<row>", as the input reports a
/// row after slot_of), or none for the slot itself (a drag that starts between the rows) and any
/// other node. `ListView::on` takes it as it is.
pub fn row_of(slot: &str, node: &str) -> Option<usize> {
    node.strip_prefix(slot)?.strip_prefix('/')?.split('/').next()?.parse().ok()
}

/// The part a node name points at: "<slot>/<row>/<part>" (a row of a list) or "<slot>/<part>"
/// (a component in a plain slot) gives "<part>", the name harry reports for a control inside
/// the component (`tile::PART_TOGGLE`, `#segmented/segment-2` ...); None for the row or the
/// slot itself. A part starts with '#', a row is a number.
pub fn part_of<'a>(slot: &str, node: &'a str) -> Option<&'a str> {
    let rest = node.strip_prefix(slot)?.strip_prefix('/')?;
    if rest.starts_with('#') {
        return Some(rest);
    }
    let (_row, part) = rest.split_once('/')?;
    part.starts_with('#').then_some(part)
}

/// The slot a node belongs to, for a node name harry reports: the clone root becomes its
/// slot, a row of a list slot "<slot>/<row>", any other name stays.
pub fn slot_of(node: &str) -> String {
    match node.split_once(CLONE_MARK) {
        Some((slot, id)) => match id.strip_prefix(ROW_ID).or_else(|| id.strip_prefix(LIST_ROW_ID)).and_then(|r| r.parse::<usize>().ok()) {
            Some(row) => format!("{slot}/{row}"),
            None => slot.to_string(),
        },
        None => node.to_string(),
    }
}

/// A component in a slot: which variant of which component set, and what to set on its parts.
#[derive(Clone, Debug)]
pub struct ComponentItem {
    id: String,
    slot: String,
    /// Where the clone sits in the slot (a row of a list below the first).
    position: (f32, f32),
    /// A row of a horizontal list: the scroll moves it in x.
    horizontal: bool,
    /// A scrolling list: the content's scroll, subtracted from the row's y frame by frame
    /// (a fling steps here, not in the app), and the slot's height for the hit test.
    scroll: Option<(Scroll, f32)>,
    /// A row of a scrolling list: how much of the list it takes (the row pitch). A row that
    /// is out of the slot is not drawn (opacity 0: harry skips its subtree).
    extent: f32,
    component: &'static str,
    variant: String,
    animation: AnimationOverride,
    texts: Vec<(String, String)>,
    meters: Vec<(String, Meter)>,
    opacities: Vec<(String, Meter)>,
    interactive: Vec<(String, bool)>,
    blinks: Vec<(String, Option<Instant>)>,
    /// Parts that sweep (a progress marker 0..100 per period, from the start again) since a
    /// time; none stands at 0.
    sweeps: Vec<(String, Option<Instant>, Duration)>,
    /// The clone takes taps (a control); false for a display-only part laid over something
    /// that does (a skeleton over the browser).
    taps: bool,
    /// The whole clone shown or not (opacity 1 or 0; a hidden clone takes no taps), none: as
    /// the design has it.
    shown: Option<bool>,
    /// Parts shown or hidden.
    visibility: Vec<(String, bool)>,
    /// Pictures on parts (a registered image, crate::images).
    images: Vec<(String, ImageContentData)>,
    /// Parts a variant may or may not have (an icon slot only in kind=icon): checked to be
    /// in at least one variant, not in every.
    optional: Vec<String>,
    /// A scrollbar: its thumb (a marker) follows the scroll frame by frame, the clone fades
    /// out after the content stopped.
    thumb: Option<(String, f32)>,
}

impl ComponentItem {
    /// The variant `variant` of the component set `component` in the slot frame `slot`.
    pub fn new(slot: &str, component: &'static str, variant: String) -> Self {
        ComponentItem {
            id: ITEM_ID.to_string(),
            slot: slot.to_string(),
            position: (0.0, 0.0),
            horizontal: false,
            scroll: None,
            extent: 0.0,
            component,
            variant,
            animation: AnimationOverride::DisableAnimation,
            texts: Vec::new(),
            meters: Vec::new(),
            opacities: Vec::new(),
            interactive: Vec::new(),
            blinks: Vec::new(),
            sweeps: Vec::new(),
            taps: true,
            shown: None,
            visibility: Vec::new(),
            images: Vec::new(),
            optional: Vec::new(),
            thumb: None,
        }
    }

    /// Row `row` of a list slot, at `y` from the slot's top (harry puts list items at absolute
    /// positions; the row pitch is the component's, see `rows`).
    pub fn row(mut self, row: usize, y: f32) -> Self {
        self.id = format!("{ROW_ID}{row}");
        self.position = (0.0, y);
        self
    }

    /// A row of a horizontal list moved up by `lift` (the chosen card of a ribbon). The rows
    /// stand RIBBON_LIFT below the slot's top, so a lifted card stays inside the slot, which
    /// clips: the slot is RIBBON_LIFT higher than a card.
    pub fn lifted(mut self, lift: f32) -> Self {
        self.position.1 = RIBBON_LIFT - lift;
        self
    }

    /// Row `row` of a horizontal list slot, at `x` from the slot's left.
    pub fn row_x(mut self, row: usize, x: f32) -> Self {
        self.id = format!("{ROW_ID}{row}");
        self.position = (x, 0.0);
        self.horizontal = true;
        self
    }

    /// Cell `index` of a grid slot, at (`x`, `y`) from the slot's top left. Taps come as
    /// "<slot>/<index>", as a row's do.
    pub fn cell(mut self, index: usize, x: f32, y: f32) -> Self {
        self.id = format!("{ROW_ID}{index}");
        self.position = (x, y);
        self
    }

    /// The whole component shown or hidden, without a change of variant: the clone's opacity
    /// is 1 or 0 (display only, no rebuild), and a hidden clone takes no taps, so they reach
    /// what lies under it. harry has no hidden layers (a hidden layer is gone from the .dcf):
    /// this is how a component is hidden.
    pub fn shown(mut self, on: bool) -> Self {
        self.shown = Some(on);
        self
    }

    /// The animation into the variant (on the clone root).
    pub fn animation(mut self, animation: AnimationOverride) -> Self {
        self.animation = animation;
        self
    }

    /// Text of a part.
    pub fn text(mut self, node: impl Into<String>, text: impl Into<String>) -> Self {
        self.texts.push((node.into(), text.into()));
        self
    }

    /// Meter value (0..100) of a part (progress bar or marker), moving to it.
    pub fn meter(mut self, node: impl Into<String>, meter: Meter) -> Self {
        self.meters.push((node.into(), meter));
        self
    }

    /// Opacity (0..100) of a part, moving to it.
    pub fn opacity(mut self, node: impl Into<String>, meter: Meter) -> Self {
        self.opacities.push((node.into(), meter));
        self
    }

    /// A part that blinks (a caret): on and off every BLINK_MS from `since`; none hides it.
    pub fn blink(mut self, node: impl Into<String>, since: Option<Instant>) -> Self {
        self.blinks.push((node.into(), since));
        self
    }

    /// A part that sweeps (a progress marker): 0..100 every `period` from `since`, from the
    /// start again after each pass; none stands at 0. harry is woken every SWEEP_FRAME
    /// while it sweeps.
    pub fn sweep(mut self, node: impl Into<String>, since: Option<Instant>, period: Duration) -> Self {
        self.sweeps.push((node.into(), since, period));
        self
    }

    /// The clone takes no taps: they go to what is under it.
    pub fn passive(mut self) -> Self {
        self.taps = false;
        self
    }

    /// A picture on a part (a registered image, crate::images).
    pub fn image(mut self, node: impl Into<String>, image: ImageContentData) -> Self {
        self.images.push((node.into(), image));
        self
    }

    /// A part that takes taps by its own name (the clone root always does).
    pub fn interactive(mut self, node: impl Into<String>, on: bool) -> Self {
        self.interactive.push((node.into(), on));
        self
    }

    /// A part shown or hidden (a variant change rebuilds the tree, so for rare changes).
    pub fn visible(mut self, node: impl Into<String>, on: bool) -> Self {
        self.visibility.push((node.into(), on));
        self
    }

    /// Marks a part as one not every variant has (the contract test wants it in at least
    /// one): an icon slot that only kind=icon draws.
    pub fn optional(mut self, node: impl Into<String>) -> Self {
        self.optional.push(node.into());
        self
    }

    /// Row `row` of a scrolling list, named apart from the rows of a fixed list
    /// (`list_row_slot`).
    fn list_row(mut self, row: usize) -> Self {
        self.id = format!("{LIST_ROW_ID}{row}");
        self
    }

    /// A row of a scrolling list: `scroll` moves it, `view_h` is the slot's height, `pitch`
    /// what the row takes of the list.
    fn scrolled(mut self, scroll: Scroll, view_h: f32, pitch: f32) -> Self {
        self.scroll = Some((scroll, view_h));
        self.extent = pitch;
        self
    }
}

impl ComponentItem {
    /// The component set of the item.
    pub fn component(&self) -> &'static str {
        self.component
    }

    /// The variant of the set the item asks for (the name of a variant in Figma).
    pub fn variant(&self) -> &str {
        &self.variant
    }

    /// The names of the parts the code sets, less the optional ones: they must be in every
    /// variant of the set (the contract with the Figma library, checked by everyware_ui_test).
    pub fn part_names(&self) -> Vec<&str> {
        let texts = self.texts.iter().map(|(n, _)| n);
        let meters = self.meters.iter().chain(&self.opacities).map(|(n, _)| n);
        let rest = self.interactive.iter().map(|(n, _)| n).chain(self.blinks.iter().map(|(n, _)| n)).chain(self.sweeps.iter().map(|(n, _, _)| n)).chain(self.visibility.iter().map(|(n, _)| n));
        let thumb = self.thumb.iter().map(|(n, _)| n);
        let mut names: Vec<&str> = texts.chain(meters).chain(rest).chain(thumb).map(String::as_str).filter(|n| !self.optional.iter().any(|o| o == n)).collect();
        names.sort_unstable();
        names.dedup();
        names
    }

    /// The optional parts: in at least one variant of the set.
    /// Every part set so far whose name starts with `prefix` is optional: a part of one kind of a
    /// set (the toggle inside #tile kind=toggle), in that kind's variants only.
    pub fn optional_under(mut self, prefix: &str) -> Self {
        let names: Vec<String> = self.part_names().into_iter().filter(|n| n.starts_with(prefix)).map(String::from).collect();
        self.optional.extend(names);
        self
    }

    pub fn optional_part_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.optional.iter().map(String::as_str).collect();
        names.sort_unstable();
        names.dedup();
        names
    }
}

impl CustomizableListItem for ComponentItem {
    fn node_selector(&self) -> NodeSelector {
        NodeSelector::ComponentWithVariant {
            component_id: self.component.to_string(),
            variant_qualifier: self.variant.clone(),
        }
    }

    fn unique_id(&self) -> &String {
        &self.id
    }

    fn describe_content(&self) -> String {
        self.variant.clone()
    }
}

/// The customizer of a slot field. It keeps the shown value of every moving part, so it is
/// one per field (the derive makes one per field) and the values survive the clone of the
/// UI model on every frame.
#[derive(Clone, Debug, Default)]
pub struct SlotCustomizer {
    follows: HashMap<String, Follow>,
}

impl SlotCustomizer {
    fn step(&mut self, key: String, meter: &Meter, now: Instant) -> (f32, bool) {
        self.follows.entry(key).or_default().step(meter.target, meter.transition, now)
    }
}

impl Customizer<Slot> for SlotCustomizer {
    fn format(&mut self, slot: Slot) -> CustomizerOutput {
        let now = Instant::now();
        let mut moving = false;
        // The next caret flip: harry wakes up then, not every frame.
        let mut next_flip: Option<Duration> = None;
        // A scrolling list: the rows' y follow the scroll; a fling steps here every frame.
        let scroll = slot.items().first().and_then(|item| item.scroll).map(|(scroll, view_h)| {
            let (y, flinging) = scroll.at(now);
            moving |= flinging;
            (y, view_h)
        });
        // A scrollbar: shown while the content moves and SCROLLBAR_DELAY after, then fades;
        // harry wakes up when the delay ends.
        let scrollbar = slot.items().first().and_then(|item| item.thumb.as_ref().zip(item.scroll)).map(|((node, max), (scroll, _))| {
            let (shown, until) = scroll.scrollbar_shown(now);
            if let Some(until) = until {
                let wait = until.saturating_duration_since(now);
                next_flip = Some(next_flip.map_or(wait, |n| n.min(wait)));
            }
            (node.clone(), *max, shown)
        });
        let items = slot
            .items()
            .iter()
            .map(|item| {
                let mut c = RenderCustomizations::new();
                let root = format!("{}{CLONE_MARK}{}", item.slot, item.id);
                // Fetched variants keep their place in the component set; the clone sits at
                // the slot's origin, a row of a list at its y minus the scroll (a scrollbar
                // stays: its thumb follows the scroll).
                let scrolled = scroll.map_or(0.0, |(s, _)| s);
                let (x, y) = match (item.thumb.is_some(), item.horizontal) {
                    (true, _) => item.position,
                    (false, true) => (item.position.0 - scrolled, item.position.1),
                    (false, false) => (item.position.0, item.position.1 - scrolled),
                };
                c.set_node_position_override(root.clone(), NodePositionOverride::new(Some(x), Some(y)));
                // the row's place along the list
                let y = if item.horizontal { x } else { y };
                // A row out of its slot is not drawn: at opacity 0 harry skips its subtree, in
                // its own passes and in the display list. Most rows of a long list are out.
                let in_view = item.thumb.is_some() || scroll.map_or(true, |(_, view_h)| y + item.extent > 0.0 && y < view_h);
                // A row takes taps while some of it is in the slot, however high it is. harry
                // hits by bounds without clipping, so the part of a row out of the slot lies over
                // what is beside the list: `Input` takes a touch on a row only where the slot is
                // under the finger too, and a touch beside the list goes on to what is there.
                c.set_interactive(root.clone(), in_view && item.taps && item.shown != Some(false));
                match (in_view, item.shown) {
                    (false, _) => c.set_opacity(root.clone(), 0.0),
                    (true, Some(on)) => c.set_opacity(root.clone(), if on { 1.0 } else { 0.0 }),
                    (true, None) => {}
                }
                if let Some((node, max, shown)) = &scrollbar {
                    let (y, view_h) = scroll.unwrap_or((0.0, 0.0));
                    c.set_meter_value(node.clone(), thumb_meter(y, *max, view_h));
                    let target = if *shown { 100.0 } else { 0.0 };
                    let fade = (!*shown).then(|| transitions::transition(ComponentKind::Scrollbar, Change::Hide)).flatten();
                    let (v, m) = self.step(format!("{}/opacity/{root}", item.id), &Meter::new(target, fade), now);
                    moving |= m;
                    c.set_opacity(root.clone(), v / 100.0);
                }
                for (node, on) in &item.visibility {
                    c.set_visibility(node.clone(), *on);
                }
                for (node, image) in &item.images {
                    c.set_image(node.clone(), image.clone());
                }
                c.set_animation_override(root, item.animation.clone());
                for (node, text) in &item.texts {
                    c.set_text_content(node.clone(), text.clone());
                }
                for (node, meter) in &item.meters {
                    let (v, m) = self.step(format!("{}/meter/{node}", item.id), meter, now);
                    moving |= m;
                    c.set_meter_value(node.clone(), v);
                }
                for (node, meter) in &item.opacities {
                    let (v, m) = self.step(format!("{}/opacity/{node}", item.id), meter, now);
                    moving |= m;
                    c.set_opacity(node.clone(), v / 100.0);
                }
                for (node, since) in &item.blinks {
                    let on = since.is_some_and(|since| caret_phase_on(since, now));
                    if let Some(since) = since {
                        let ms = now.saturating_duration_since(*since).as_millis() as u64;
                        let wait = Duration::from_millis(BLINK_MS - ms % BLINK_MS);
                        next_flip = Some(next_flip.map_or(wait, |n| n.min(wait)));
                    }
                    c.set_opacity(node.clone(), if on { 1.0 } else { 0.0 });
                }
                for (node, since, period) in &item.sweeps {
                    let at = match since {
                        Some(since) => {
                            next_flip = Some(next_flip.map_or(SWEEP_FRAME, |n| n.min(SWEEP_FRAME)));
                            let period = period.as_millis().max(1);
                            (now.saturating_duration_since(*since).as_millis() % period) as f32 / period as f32 * 100.0
                        }
                        None => 0.0,
                    };
                    c.set_meter_value(node.clone(), at);
                }
                for (node, on) in &item.interactive {
                    c.set_interactive(node.clone(), *on);
                }
                ListItem::new(item.id.clone(), item.describe_content(), item.node_selector(), c)
            })
            .collect();
        let list = RenderCustomization::ListCustomization(ListCustomization::new(items));
        match (moving, next_flip) {
            (true, _) => CustomizerOutput::new_update_next_frame(list),
            (false, Some(wait)) => CustomizerOutput::new_update_in(list, wait),
            (false, None) => CustomizerOutput::new_finished(list),
        }
    }
}

/// A component that can sit in a slot: its component set and what it sets on its parts.
pub trait Slotted: Component {
    /// The component set in the Figma library.
    const SET: &'static str;

    /// Sets the parts on the item (texts, meters, opacities).
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        item
    }
}

/// The slot field's value for `component` in the slot frame `slot`.
pub fn slot<C: Slotted>(slot: &str, component: &C) -> Slot {
    let item = ComponentItem::new(slot, C::SET, component.variant())
        .animation(animation(component.transition()));
    List::new(vec![component.parts(item)])
}

/// The slot field's value for `component` in the slot frame `slot`, shown or hidden as a whole
/// (`ComponentItem::shown`): a card's face, a hint that comes and goes.
pub fn slot_shown<C: Slotted>(slot: &str, component: &C, shown: bool) -> Slot {
    let item = ComponentItem::new(slot, C::SET, component.variant())
        .animation(animation(component.transition()))
        .shown(shown);
    List::new(vec![component.parts(item)])
}

/// The slot field's value for a grid of `components` in the slot frame `slot`: `columns` per
/// row, left to right and top to bottom, `pitch` (x, y) apart (the component's size plus the
/// gap). One field in the UI model for the whole grid; taps come as "<slot>/<index>"
/// (`row_of`).
pub fn grid<'a, C: Slotted + 'a>(slot: &str, components: impl IntoIterator<Item = &'a C>, columns: usize, pitch: (f32, f32)) -> Slot {
    cells(slot, components.into_iter().map(|c| (c, None)), columns, pitch)
}

/// A grid (`grid`) whose cells are each shown or hidden as a whole (`ComponentItem::shown`):
/// the faces of a card game over the cards' backs.
pub fn grid_shown<'a, C: Slotted + 'a>(slot: &str, components: impl IntoIterator<Item = (&'a C, bool)>, columns: usize, pitch: (f32, f32)) -> Slot {
    cells(slot, components.into_iter().map(|(c, on)| (c, Some(on))), columns, pitch)
}

fn cells<'a, C: Slotted + 'a>(slot: &str, components: impl Iterator<Item = (&'a C, Option<bool>)>, columns: usize, pitch: (f32, f32)) -> Slot {
    let columns = columns.max(1);
    List::new(
        components
            .enumerate()
            .map(|(i, (component, shown))| {
                let (col, row) = (i % columns, i / columns);
                let item = ComponentItem::new(slot, C::SET, component.variant())
                    .cell(i, col as f32 * pitch.0, row as f32 * pitch.1)
                    .animation(animation(component.transition()));
                let item = component.parts(item);
                ComponentItem { shown, ..item }
            })
            .collect(),
    )
}

/// The slot field's value for a list of `components` in the slot frame `slot`, one row under
/// the other, `pitch` apart (the component's row height plus the gap). Taps come as
/// "<slot>/<row>".
pub fn rows<'a, C: Slotted + 'a>(slot: &str, components: impl IntoIterator<Item = &'a C>, pitch: f32) -> Slot {
    List::new(
        components
            .into_iter()
            .enumerate()
            .map(|(row, component)| {
                let item = ComponentItem::new(slot, C::SET, component.variant())
                    .row(row, row as f32 * pitch)
                    .animation(animation(component.transition()));
                component.parts(item)
            })
            .collect(),
    )
}

/// The slot field's value for a scrolling list (`ListView`) in the slot frame `slot`, which
/// clips: every row at its y minus the scroll of the frame; a fling moves the rows frame by
/// frame from here. Taps come as "<slot>/<row>".
pub fn list<R: Row + Slotted>(slot: &str, view: &ListView<R>) -> Slot {
    let (scroll, view_h) = (view.scroll(), view.view_h());
    // where each row starts: the rows before it, each with its extent (ListView::offset)
    let mut at = 0.0;
    List::new(
        view.rows()
            .iter()
            .enumerate()
            .map(|(row, component)| {
                let (offset, extent) = (at, view.extent(row));
                at += extent;
                let item = ComponentItem::new(slot, R::SET, component.variant());
                // a ribbon lifts its chosen card (harry has no scale: the card moves up instead)
                let item = if view.is_horizontal() {
                    item.row_x(row, offset).lifted(if component.is_selected() { RIBBON_LIFT } else { 0.0 })
                } else {
                    item.row(row, offset)
                };
                let item = item
                    .list_row(row)
                    .scrolled(scroll, view_h, extent)
                    .animation(animation(component.transition()));
                // a row that takes no tap (a subheader): the touch goes to the list slot below it
                let item = if component.takes_taps() { item } else { item.passive() };
                component.parts(item)
            })
            .collect(),
    )
}

/// The slot field's value for the scrollbar of `view` in the slot frame `slot`: the
/// library's `#scrollbar` with its bottom at the list's bottom, the track as high as the list,
/// its thumb at the scroll, shown while the content moves and SCROLLBAR_DELAY after.
pub fn scrollbar(slot: &str, view: &ListView) -> Slot {
    let view_h = view.view_h().min(SCROLLBAR_MAX);
    // passive: the clone reaches far above its slot (harry hits by bounds, without clipping)
    let item = ComponentItem::new(slot, "#scrollbar", "state=default".to_string())
        .passive()
        .meter("#scrollbar/track", Meter::new(view_h / SCROLLBAR_MAX * 100.0, None))
        .scrolled(view.scroll(), view.view_h(), 0.0);
    List::new(vec![ComponentItem {
        thumb: Some(("#scrollbar/thumb".to_string(), view.max_scroll())),
        position: (0.0, view_h - SCROLLBAR_MAX),
        ..item
    }])
}

/// The pressed look of a component: its pressed layer fades in and out (display-only), the
/// variant stays.
fn pressed(item: ComponentItem, node: &str, press: &Pressable, kind: ComponentKind) -> ComponentItem {
    item.opacity(node, Meter::new(press.pressed_target(), press.press_transition(kind)))
}

impl Slotted for Button {
    const SET: &'static str = "#button";
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        pressed(item.text("#button/label", self.label.clone()), "#button/pressed", &self.press, Self::KIND)
    }
}

impl Slotted for IconButton {
    const SET: &'static str = "#icon-button";
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        pressed(item, "#icon-button/pressed", &self.press, Self::KIND)
    }
}

impl Slotted for Toggle {
    const SET: &'static str = "#toggle";
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        let to = Meter::new(self.target(), self.value_transition());
        item.meter("#toggle/thumb", to).opacity("#toggle/on", to)
    }
}

impl Slotted for Radio {
    const SET: &'static str = "#radio";
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        item.text("#radio/label", self.label.clone())
    }
}

impl Slotted for ListRow {
    const SET: &'static str = "#list-item";
    /// Title, subtitle, the pressed layer, the two icon slots, with an image the picture (a
    /// registered image; an unknown key keeps the placeholder) and the trailing text, with a
    /// leading text its two lines and the bar's colour (a picture of the colour). A row
    /// names only the parts its kind and lines have: every part costs on every frame (a text,
    /// a meter, a name), in every row, also in the rows nobody sees.
    ///
    /// An icon slot is a stack of every glyph at opacity 0 (display-only; a variant cannot
    /// change inside an item): with a glyph, that one is lit; with none, the whole stack is
    /// off by one part (its frame), so harry skips it.
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        let mut item = item.text("#list-item/title", self.title.clone());
        if self.lines != ListItemLines::One {
            item = item.text("#list-item/subtitle", self.subtitle.clone()).optional("#list-item/subtitle");
        }
        if matches!(self.kind, ListItemKind::Image | ListItemKind::Text | ListItemKind::Subheader) {
            item = item.text("#list-item/trailing-text", self.trailing_text.clone()).optional("#list-item/trailing-text");
        }
        if self.kind == ListItemKind::LeadingText {
            item = item
                .text("#list-item/leading-title", self.leading_title.clone())
                .text("#list-item/leading-subtitle", self.leading_subtitle.clone())
                .optional("#list-item/leading-title")
                .optional("#list-item/leading-subtitle")
                .optional("#list-item/accent");
            // the bar's colour: a picture of it on the plain frame (harry sets no fill)
            if let Some(color) = self.accent.and_then(crate::images::color) {
                item = item.image("#list-item/accent", color);
            }
        }
        if self.kind == ListItemKind::Image {
            item = item.optional("#list-item/image");
            if let Some(image) = self.image.as_deref().and_then(crate::images::get) {
                item = item.image("#list-item/image", image);
            }
        }
        let stacks = [
            ("#list-item/leading", self.leading, self.kind == ListItemKind::Icon),
            ("#list-item/trailing", self.trailing, matches!(self.kind, ListItemKind::Plain | ListItemKind::Icon)),
        ];
        for (stack, chosen, has) in stacks {
            if !has {
                continue;
            }
            match chosen.filter(|glyph| GLYPHS.contains(glyph)) {
                None => item = item.opacity(stack, Meter::new(0.0, None)).optional(stack),
                Some(chosen) => {
                    for glyph in GLYPHS {
                        let node = format!("{stack}/{glyph}");
                        let on = if glyph == chosen { 100.0 } else { 0.0 };
                        item = item.opacity(node.clone(), Meter::new(on, None)).optional(node);
                    }
                }
            }
        }
        pressed(item, "#list-item/pressed", &self.press, Self::KIND)
    }
}

impl Slotted for Slider {
    const SET: &'static str = "#slider";
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        let to = Meter::new(self.value, self.value_transition());
        item.meter("#slider/fill", to).meter("#slider/thumb", to)
    }
}

impl Slotted for Dropdown {
    const SET: &'static str = "#dropdown";
    /// The list's items take taps by their names (`#dropdown/item/N`) while it is open; only
    /// one list is open at a time, so the names are not ambiguous.
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        let select = transitions::transition(Self::KIND, Change::Select);
        let mut item = item
            .text("#dropdown/value", self.value())
            .opacity("#dropdown/list", Meter::new(self.list_target(), self.list_transition()));
        for (i, label) in self.items.iter().enumerate() {
            let n = format!("#dropdown/item/{}", i + 1);
            item = item
                .text(format!("{n}/label"), label.clone())
                .opacity(format!("{n}/selected"), Meter::new(self.mark_target(i), select))
                .interactive(n, self.open);
        }
        item
    }
}

impl Slotted for Segmented {
    const SET: &'static str = "#segmented";
    /// Per segment its labels and its light: the chosen one's highlight and white label at 100,
    /// the ink2 label at the rest. The segments take taps by their names
    /// (`#segmented/segment-<i>`), unambiguous while one segmented button is on the screen.
    /// Segments 3 and 4 are not in every variant (segments=2 has two).
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        let move_to = self.value_transition();
        self.labels.iter().enumerate().fold(item, |item, (i, label)| {
            let n = format!("#segmented/segment-{}", i + 1);
            let on = self.target(i);
            let names = [n.clone(), format!("{n}/selected"), format!("{n}/label"), format!("{n}/label-selected")];
            let item = item
                .text(format!("{n}/label"), label.clone())
                .text(format!("{n}/label-selected"), label.clone())
                .opacity(format!("{n}/selected"), Meter::new(on, move_to))
                .opacity(format!("{n}/label-selected"), Meter::new(on, move_to))
                .opacity(format!("{n}/label"), Meter::new(100.0 - on, move_to))
                .interactive(n, self.press.enabled);
            if i >= 2 { names.into_iter().fold(item, |item, name| item.optional(name)) } else { item }
        })
    }
}

/// Per tab its labels and its light: the chosen one's pill and accent label at 100, the ink2
/// label at the rest. The tabs take taps by their names (`#tabs/tab-<i>`); `Input` reports
/// them with the slot (`#home/tabs/#tabs/tab-2`), so two tab bars on one screen stay apart.
/// Tabs 3 and 4 are not in every variant (tabs=2 has two).
impl Slotted for Tabs {
    const SET: &'static str = "#tabs";
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        let move_to = self.value_transition();
        self.labels.iter().enumerate().fold(item, |item, (i, label)| {
            let n = format!("#tabs/tab-{}", i + 1);
            let on = self.target(i);
            let names = [n.clone(), format!("{n}/selected"), format!("{n}/label"), format!("{n}/label-selected")];
            let item = item
                .text(format!("{n}/label"), label.clone())
                .text(format!("{n}/label-selected"), label.clone())
                .opacity(format!("{n}/selected"), Meter::new(on, move_to))
                .opacity(format!("{n}/label-selected"), Meter::new(on, move_to))
                .opacity(format!("{n}/label"), Meter::new(100.0 - on, move_to))
                .interactive(n, self.press.enabled);
            if i >= 2 { names.into_iter().fold(item, |item, name| item.optional(name)) } else { item }
        })
    }
}

/// The card's texts, its symbol (one glyph of the stack lit), pressed, the picture, and the
/// control inside it as its own binding sets it (the same part names). The control takes taps
/// of its own: its node is interactive in the clone, harry reports it first and the card (the
/// clone root with its row) next, and `Input` pairs the two (tile.rs). Parts of other kinds'
/// controls are not in every variant.
impl Slotted for Tile {
    const SET: &'static str = "#tile";
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        let press = transitions::transition(Self::KIND, if self.press.pressed { Change::Press } else { Change::Release });
        let mut item = item
            .text("#tile/title", self.title.clone())
            .text("#tile/subtitle", self.subtitle.clone())
            .optional("#tile/value")
            .opacity("#tile/pressed", Meter::new(if self.press.pressed { 100.0 } else { 0.0 }, press));
        if self.kind != TileKind::Segmented && self.kind != TileKind::Image {
            item = item.text("#tile/value", self.value.clone());
        }
        for icon in SymbolIcon::ALL {
            let n = format!("#tile/symbol/{}", icon.name());
            item = item.opacity(n.clone(), Meter::new(if *icon == self.symbol { 100.0 } else { 0.0 }, None)).optional(n);
        }
        // the control of this kind, its parts in this kind's variants only; the control takes
        // taps while the card is enabled
        let taps = self.press.enabled;
        match self.kind {
            TileKind::Toggle => self.toggle.parts(item).interactive(crate::tile::PART_TOGGLE, taps).optional(crate::tile::PART_TOGGLE).optional_under("#toggle/"),
            TileKind::Slider => self.slider.parts(item).interactive(crate::tile::PART_SLIDER, taps).optional(crate::tile::PART_SLIDER).optional_under("#slider/"),
            TileKind::Segmented => (1..=self.segmented.labels.len())
                .fold(self.segmented.parts(item), |item, i| item.interactive(format!("#segmented/segment-{i}"), taps))
                .optional_under("#segmented/"),
            TileKind::Stepper => item.interactive(crate::tile::PART_MINUS, taps).interactive(crate::tile::PART_PLUS, taps).optional(crate::tile::PART_MINUS).optional(crate::tile::PART_PLUS),
            TileKind::Image => match self.image.as_deref().and_then(crate::images::get) {
                Some(image) => item.image("#tile/image", image),
                None => item,
            },
        }
    }
}

impl Slotted for Symbol {
    const SET: &'static str = "#symbol";
}

/// The shimmer's three markers (a bright core between two softer edges) share one travel and
/// sweep together; the clone takes no taps (it lies over the content it stands for).
impl Slotted for Skeleton {
    const SET: &'static str = "#skeleton";
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        let period = Skeleton::period();
        (1..=3).fold(item.passive(), |item, i| item.sweep(format!("#skeleton/shimmer/{i}"), self.since, period))
    }
}

impl Slotted for BarChart {
    const SET: &'static str = "#bar-chart";
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        self.bars.iter().enumerate().fold(item, |item, (i, bar)| {
            let name = format!("#bar-chart/bar/{}", i + 1);
            let accent = Meter::new(if bar.accent { 100.0 } else { 0.0 }, None);
            item.meter(name.clone(), Meter::new(bar.level, None))
                .meter(format!("{name}/accent"), Meter::new(bar.level, None))
                .opacity(format!("{name}/accent"), accent)
        })
    }
}

impl Slotted for Keyboard {
    const SET: &'static str = "#keyboard";
    /// Every key: its label for the mode, its pressed layer, a tap target by its own name
    /// (`#kbd/r1/3`; one keyboard on a screen, so the names are not ambiguous).
    fn parts(&self, mut item: ComponentItem) -> ComponentItem {
        for node in Keyboard::key_nodes() {
            if let Some(label) = self.label_of(&node) {
                item = item.text(format!("{node}/label"), label);
            }
            let pressed = Meter::new(self.pressed_target(&node), self.press_transition(&node));
            item = item.opacity(format!("{node}/pressed"), pressed).interactive(node, true);
        }
        item
    }
}

impl Slotted for InputField {
    const SET: &'static str = "#input-field";
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        item.text("#input-field/text", self.shown())
            .text("#input-field/placeholder", self.placeholder.clone())
            .opacity("#input-field/placeholder", Meter::new(self.placeholder_target(), None))
            .text("#input-field/helper", self.helper_line())
            .opacity("#input-field/focus", Meter::new(self.focus_target(), self.focus_transition()))
            .blink("#input-field/caret", self.caret_since())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use customizations::Customized;
    use customizations::UpdatePolicy;
    use dc_bundle::definition::NodeQuery;
    use dc_bundle::definition_file::load_design_def;
    use dc_bundle::view::view_data::View_data_type;
    use dc_bundle::view::View;
    use everyware_ui_core::button::{ButtonKind, Icon, IconButtonKind};
    use everyware_ui_core::list_item::{ListItemKind, ListItemLines, ListItemSize};
    use everyware_ui_core::radio::RadioGroup;
    use everyware_ui_core::skeleton::{SkeletonKind, SkeletonSize};
    use everyware_ui_core::symbol::SymbolIcon;
    use std::path::PathBuf;
    use std::thread::sleep;

    /// The customizations of the one item of a formatted slot.
    fn item_of(output: &CustomizerOutput) -> &RenderCustomizations {
        let RenderCustomization::ListCustomization(list) = &output.result else {
            panic!("a slot formats to a list customization")
        };
        list.items()[0].render_customization()
    }

    #[test]
    fn a_row_and_a_part_are_read_off_the_node_name() {
        assert_eq!(row_of("#tv/cards", "#tv/cards/3"), Some(3));
        assert_eq!(row_of("#tv/cards", "#tv/cards/3/#tile/toggle"), Some(3));
        assert_eq!(row_of("#tv/cards", "#tv/cards"), None);
        assert_eq!(row_of("#tv/cards", "#tv/nav"), None);
        assert_eq!(part_of("#tv/cards", "#tv/cards/3/#tile/toggle"), Some("#tile/toggle"));
        assert_eq!(part_of("#tv/cards", "#tv/cards/3/#segmented/segment-2"), Some("#segmented/segment-2"));
        assert_eq!(part_of("#tv/cards", "#tv/cards/3"), None);
        assert_eq!(part_of("#tv/cards", "#tv/cards"), None);
        // a component in a plain slot: no row, the part right after the slot
        assert_eq!(part_of("#hub/fan", "#hub/fan/#segmented/segment-2"), Some("#segmented/segment-2"));
        assert_eq!(part_of("#rooms/tile-1", "#rooms/tile-1/#tile/toggle"), Some("#tile/toggle"));
        assert_eq!(row_of("#rooms/tile-1", "#rooms/tile-1/#tile/toggle"), None);
        assert_eq!(part_of("#hub/fan", "#hub/fan"), None);
    }

    #[test]
    fn two_slots_with_one_component_are_customized_on_their_own() {
        let (mut a, mut b) = (SlotCustomizer::default(), SlotCustomizer::default());
        let on = a.format(slot("#a", &Toggle::new(true)));
        let off = b.format(slot("#b", &Toggle::new(false)));
        let thumb = |o: &CustomizerOutput| item_of(o).get("#toggle/thumb").and_then(|c| c.get_meter_value());
        assert_eq!((thumb(&on), thumb(&off)), (Some(100.0), Some(0.0)));
        assert!(item_of(&on).get(&clone_name("#a")).and_then(|c| c.is_interactive()).unwrap());
    }

    #[test]
    fn next_frame_while_moving_then_finished() {
        let mut c = SlotCustomizer::default();
        let mut toggle = Toggle::new(false);
        assert_eq!(c.format(slot("#t", &toggle)).update_policy, UpdatePolicy::Finished, "first value at once");
        toggle.set(true);
        assert_eq!(c.format(slot("#t", &toggle)).update_policy, UpdatePolicy::NextFrame);
        // the value transition is 200 ms: frame by frame, as a screen (a frame that comes
        // late holds the animation, Follow's LATE)
        for _ in 0..13 {
            sleep(Duration::from_millis(20));
            c.format(slot("#t", &toggle));
        }
        let done = c.format(slot("#t", &toggle));
        assert_eq!(done.update_policy, UpdatePolicy::Finished);
        assert_eq!(item_of(&done).get("#toggle/thumb").and_then(|c| c.get_meter_value()), Some(100.0));
    }

    #[test]
    fn a_loading_skeleton_sweeps_at_30_fps_and_takes_no_taps() {
        let mut skeleton = Skeleton::new(SkeletonKind::Text, SkeletonSize::Wide);
        let out = SlotCustomizer::default().format(slot("#k", &skeleton));
        assert_eq!(out.update_policy, UpdatePolicy::Finished, "still: nothing to wake up for");
        assert_eq!(item_of(&out).get("#skeleton/shimmer/2").and_then(|c| c.get_meter_value()), Some(0.0));
        assert_eq!(item_of(&out).get(&clone_name("#k")).and_then(|c| c.is_interactive()), Some(false));
        skeleton.start(Instant::now() - Duration::from_millis(700));
        let out = SlotCustomizer::default().format(slot("#k", &skeleton));
        let UpdatePolicy::UpdateInMillis(wait) = out.update_policy else { panic!("{:?}", out.update_policy) };
        assert!(wait <= SWEEP_FRAME);
        let at = item_of(&out).get("#skeleton/shimmer/1").and_then(|c| c.get_meter_value()).unwrap();
        assert!((49.0..60.0).contains(&at), "half a pass after 700 of 1400 ms: {at}");
    }

    #[test]
    fn a_focused_field_wakes_up_for_the_next_caret_flip() {
        let mut field = InputField::new("Password", "hint");
        field.focus(Instant::now());
        let out = SlotCustomizer::default().format(slot("#f", &field));
        let UpdatePolicy::UpdateInMillis(wait) = out.update_policy else { panic!("{:?}", out.update_policy) };
        assert!(wait <= Duration::from_millis(BLINK_MS));
        assert_eq!(item_of(&out).get("#input-field/caret").and_then(|c| c.get_opacity()), Some(1.0));
        field.blur();
        let out = SlotCustomizer::default().format(slot("#f", &field));
        assert_eq!(out.update_policy, UpdatePolicy::Finished, "no caret, nothing to wake up for");
    }

    #[test]
    fn a_hidden_slot_is_transparent_and_lets_taps_through() {
        let icon = Symbol::new(SymbolIcon::House);
        let out = SlotCustomizer::default().format(slot_shown("#f", &icon, false));
        let root = item_of(&out).get(&clone_name("#f")).unwrap();
        assert_eq!((root.get_opacity(), root.is_interactive()), (Some(0.0), Some(false)));
        let out = SlotCustomizer::default().format(slot_shown("#f", &icon, true));
        let root = item_of(&out).get(&clone_name("#f")).unwrap();
        assert_eq!((root.get_opacity(), root.is_interactive()), (Some(1.0), Some(true)));
        let out = SlotCustomizer::default().format(slot("#f", &icon));
        assert_eq!(item_of(&out).get(&clone_name("#f")).unwrap().get_opacity(), None, "a plain slot keeps the design's opacity");
    }

    /// The opacity the code sets on row `row` of a formatted list (None: the design's).
    fn row_opacity(output: &CustomizerOutput, slot: &str, row: usize) -> Option<f32> {
        let RenderCustomization::ListCustomization(list) = &output.result else {
            panic!("a list formats to a list customization")
        };
        list.items()[row].render_customization().get(&list_row_name(slot, row)).and_then(|c| c.get_opacity())
    }

    #[test]
    fn a_row_out_of_its_slot_is_not_drawn_and_a_row_partly_in_is() {
        // 10 rows 80 apart in a slot 200 high
        let rows = (0..10).map(|i| ListRow::new(ListItemKind::Plain, format!("Row {i}"), ""));
        let mut view = ListView::new(rows, 200.0, 80.0);
        let opacities = |view: &ListView<ListRow>| {
            let out = SlotCustomizer::default().format(list("#l", view));
            (0..10).map(|row| row_opacity(&out, "#l", row)).collect::<Vec<_>>()
        };
        // rows 0 and 1 in, row 2 at 160 partly in (its bottom at 240), row 3 at 240 out
        let at_top = opacities(&view);
        assert_eq!(at_top[..3], [None, None, None], "rows in the slot keep the design's opacity");
        assert!(at_top[3..].iter().all(|o| *o == Some(0.0)), "rows below the slot: {at_top:?}");
        // scrolled by one row: row 0 at -80 ends where the slot starts, out
        view.scroll_to(1);
        let one_down = opacities(&view);
        assert_eq!((one_down[0], one_down[1], one_down[3], one_down[4]), (Some(0.0), None, None, Some(0.0)));
        // at the end (scroll 600): row 7 at -40 is partly in, row 6 at -120 out
        view.scroll_to(9);
        let at_end = opacities(&view);
        assert_eq!((at_end[6], at_end[7], at_end[9]), (Some(0.0), None, None));
    }

    /// Every part name the code sets on a list row, optional or not.
    fn row_parts(row: &ListRow) -> Vec<String> {
        let item = slot("#r", row).items()[0].clone();
        item.part_names().into_iter().chain(item.optional_part_names()).map(String::from).collect()
    }

    #[test]
    fn a_row_names_only_the_parts_its_kind_and_lines_have() {
        let glyphs_of = |parts: &[String], stack: &str| parts.iter().filter(|p| p.starts_with(&format!("{stack}/"))).count();
        let has = |parts: &[String], part: &str| parts.iter().any(|p| p == part);

        let plain = row_parts(&ListRow::new(ListItemKind::Plain, "Amsterdam", "").lines(ListItemLines::One));
        assert!(has(&plain, "#list-item/title") && has(&plain, "#list-item/pressed"));
        for part in ["#list-item/subtitle", "#list-item/trailing-text", "#list-item/image", "#list-item/leading"] {
            assert!(!has(&plain, part), "plain, one line: {part} in {plain:?}");
        }
        assert!(has(&plain, "#list-item/trailing"), "no glyph: the trailing stack is off as a whole");
        assert_eq!((glyphs_of(&plain, "#list-item/leading"), glyphs_of(&plain, "#list-item/trailing")), (0, 0));

        let two = row_parts(&ListRow::new(ListItemKind::Plain, "Amsterdam", "GMT+01:00 · Europe").lines(ListItemLines::Two));
        assert!(has(&two, "#list-item/subtitle"));

        let icon = row_parts(&ListRow::new(ListItemKind::Icon, "Home", "").leading("wifi").trailing(Some("lock")));
        assert_eq!((glyphs_of(&icon, "#list-item/leading"), glyphs_of(&icon, "#list-item/trailing")), (GLYPHS.len(), GLYPHS.len()));
        assert!(!has(&icon, "#list-item/leading") && !has(&icon, "#list-item/trailing"), "a stack with a glyph is not switched off");
        assert!(!has(&icon, "#list-item/trailing-text") && !has(&icon, "#list-item/image"));

        let text = row_parts(&ListRow::new(ListItemKind::Text, "Firmware", "").lines(ListItemLines::One).trailing_text("0.1.0"));
        assert!(has(&text, "#list-item/trailing-text") && !has(&text, "#list-item/image"));
        assert_eq!(glyphs_of(&text, "#list-item/leading") + glyphs_of(&text, "#list-item/trailing"), 0);
        assert!(!has(&text, "#list-item/leading") && !has(&text, "#list-item/trailing"));

        let image = row_parts(&ListRow::new(ListItemKind::Image, "Headline", "Supporting").lines(ListItemLines::Three).image("k").trailing_text("1 h"));
        for part in ["#list-item/image", "#list-item/trailing-text", "#list-item/subtitle"] {
            assert!(has(&image, part), "image, three lines: {part} missing in {image:?}");
        }
        assert!(!has(&image, "#list-item/leading") && !has(&image, "#list-item/trailing"));

        for kind in [ListItemKind::Toggle, ListItemKind::Chevron] {
            let parts = row_parts(&ListRow::new(kind, "Title", "").lines(ListItemLines::One));
            assert!(parts.iter().all(|p| !p.starts_with("#list-item/leading") && !p.starts_with("#list-item/trailing") && p != "#list-item/image"), "{kind:?}: {parts:?}");
        }
    }

    #[test]
    fn a_leading_text_sets_its_lines_and_its_accent_and_a_subheader_its_trailing_text() {
        let _one = crate::images::TEST_STORE.lock().unwrap_or_else(|e| e.into_inner());
        let has = |parts: &[String], part: &str| parts.iter().any(|p| p == part);
        let lead = ListRow::new(ListItemKind::LeadingText, "Stand-up", "Room 2").leading_text("8:30", "30 min").accent(0x0b8043);
        let parts = row_parts(&lead);
        for part in ["#list-item/leading-title", "#list-item/leading-subtitle", "#list-item/accent", "#list-item/title", "#list-item/subtitle"] {
            assert!(has(&parts, part), "leading text: {part} missing in {parts:?}");
        }
        assert!(!has(&parts, "#list-item/trailing-text") && !has(&parts, "#list-item/leading"));
        let out = SlotCustomizer::default().format(slot("#r", &lead));
        let c = item_of(&out);
        assert_eq!(c.get("#list-item/leading-title").and_then(|c| c.get_text()).map(String::as_str), Some("8:30"));
        let accent = c.get("#list-item/accent").and_then(|c| c.get_image_content()).expect("the bar's colour");
        assert_eq!(accent.id, crate::images::color(0x0b8043).unwrap().id);
        // no colour: the design's accent, no picture
        let plain = SlotCustomizer::default().format(slot("#r", &ListRow::new(ListItemKind::LeadingText, "A", "")));
        assert!(item_of(&plain).get("#list-item/accent").and_then(|c| c.get_image_content()).is_none());

        let head = row_parts(&ListRow::new(ListItemKind::Subheader, "Section", "").trailing_text("3 items"));
        assert!(has(&head, "#list-item/title") && has(&head, "#list-item/trailing-text") && has(&head, "#list-item/pressed"));
        assert!(!has(&head, "#list-item/subtitle") && !has(&head, "#list-item/trailing"));
    }

    #[test]
    fn a_row_with_its_own_pitch_moves_the_rows_after_it() {
        let rows = [
            ListRow::new(ListItemKind::Subheader, "A", "").pitch(48.0),
            ListRow::new(ListItemKind::Plain, "Row 1", ""),
            ListRow::new(ListItemKind::Plain, "Row 2", ""),
        ];
        let view = ListView::new(rows, 400.0, 80.0);
        let out = SlotCustomizer::default().format(list("#l", &view));
        let RenderCustomization::ListCustomization(l) = &out.result else { panic!() };
        let y = |row: usize| l.items()[row].render_customization().get(&list_row_name("#l", row)).unwrap().get_node_position_override().unwrap().y();
        assert_eq!((y(0), y(1), y(2)), (Some(0.0), Some(48.0), Some(128.0)));
    }

    #[test]
    fn a_stack_without_a_glyph_is_off_as_a_whole() {
        let row = ListRow::new(ListItemKind::Icon, "Home", "").leading("wifi");
        let out = SlotCustomizer::default().format(slot("#r", &row));
        let opacity = |node: &str| item_of(&out).get(node).and_then(|c| c.get_opacity());
        assert_eq!(opacity("#list-item/trailing"), Some(0.0), "no trailing glyph: the stack is off");
        assert_eq!(opacity("#list-item/trailing/lock"), None, "and its glyphs are not named");
        assert_eq!(opacity("#list-item/leading/wifi"), Some(1.0));
        assert_eq!(opacity("#list-item/leading/lock"), Some(0.0));
        assert_eq!(opacity("#list-item/leading"), None, "a stack with a glyph keeps the design's opacity");
    }

    #[test]
    fn a_grid_lays_its_cells_out_in_rows_and_taps_come_by_index() {
        let buttons: Vec<Button> = (0..6).map(|_| Button::new(ButtonKind::Secondary, "")).collect();
        let out = SlotCustomizer::default().format(grid("#g", &buttons, 4, (264.0, 128.0)));
        let RenderCustomization::ListCustomization(list) = &out.result else { panic!() };
        let at = |i: usize| {
            let c = list.items()[i].render_customization().get(&row_name("#g", i)).unwrap();
            let p = c.get_node_position_override().unwrap();
            (p.x(), p.y())
        };
        assert_eq!(list.items().len(), 6);
        assert_eq!(at(0), (Some(0.0), Some(0.0)));
        assert_eq!(at(3), (Some(792.0), Some(0.0)));
        assert_eq!(at(5), (Some(264.0), Some(128.0)), "the second row starts at the left again");
        assert_eq!(row_of("#g", &slot_of(&row_name("#g", 5))), Some(5));
        let faces = [Symbol::new(SymbolIcon::House), Symbol::new(SymbolIcon::Droplets)];
        let out = SlotCustomizer::default().format(grid_shown("#h", faces.iter().zip([true, false]), 4, (264.0, 128.0)));
        let RenderCustomization::ListCustomization(list) = &out.result else { panic!() };
        let opacity = |i: usize| list.items()[i].render_customization().get(&row_name("#h", i)).unwrap().get_opacity();
        assert_eq!((opacity(0), opacity(1)), (Some(1.0), Some(0.0)));
    }

    #[test]
    fn taps_on_the_clone_come_with_the_slot_name() {
        assert_eq!(slot_of(&clone_name("#wifi/connect")), "#wifi/connect");
        assert_eq!(slot_of("#kbd/r1/1"), "#kbd/r1/1");
        assert_eq!(slot_of(&row_name("#wifi/networks", 3)), "#wifi/networks/3");
        assert_eq!(row_of("#wifi/networks", &slot_of(&row_name("#wifi/networks", 3))), Some(3));
        // a row of a scrolling list: the same name for the app, and its slot for Input
        assert_eq!(slot_of(&list_row_name("#wifi/networks", 3)), "#wifi/networks/3");
        assert_eq!(list_row_slot(&list_row_name("#wifi/networks", 3)), Some("#wifi/networks"));
        assert_eq!(list_row_slot(&row_name("#wifi/networks", 3)), None, "a fixed list's row is in its slot");
        assert_eq!((list_row_slot(&clone_name("#wifi/connect")), list_row_slot("#wifi/networks")), (None, None));
        assert_eq!(row_of("#wifi/networks", "#wifi/networks"), None);
        assert_eq!(row_of("#wifi/networks", "#wifi/networks-scrollbar"), None);
    }

    /// One item per component of the library, as an app would make it.
    fn library() -> Vec<ComponentItem> {
        let mut dropdown = Dropdown::new(["Living room", "Bedroom", "Office"]);
        dropdown.open = true;
        [
            slot("#s", &Button::new(ButtonKind::Primary, "OK")),
            slot("#s", &IconButton::new(IconButtonKind::Tonal, Icon::Minus)),
            slot("#s", &Toggle::new(false)),
            slot("#s", &RadioGroup::new(["One"]).options[0].clone()),
            slot("#s", &ListRow::new(ListItemKind::Icon, "Title", "Subtitle").leading("wifi").trailing(Some("lock"))),
            slot("#s", &ListRow::new(ListItemKind::Text, "Firmware", "").lines(ListItemLines::One).size(ListItemSize::Column).trailing_text("0.1.0")),
            // a row without a glyph: its trailing stack is off as a whole, by its frame
            slot("#s", &ListRow::new(ListItemKind::Plain, "Amsterdam", "GMT+01:00 · Europe").lines(ListItemLines::Two).size(ListItemSize::Panel)),
            slot("#s", &ListRow::new(ListItemKind::Image, "Headline", "Supporting").lines(ListItemLines::Three).size(ListItemSize::Column).image("k").trailing_text("1 h")),
            slot("#s", &ListRow::new(ListItemKind::LeadingText, "Headline", "Supporting").leading_text("8:30", "30 min").size(ListItemSize::Column)),
            slot("#s", &ListRow::new(ListItemKind::LeadingText, "A headline of three lines", "Supporting").lines(ListItemLines::Four).size(ListItemSize::Regular)),
            slot("#s", &ListRow::new(ListItemKind::Subheader, "Section", "").trailing_text("3 items").size(ListItemSize::Column)),
            slot("#s", &IconButton::new(IconButtonKind::Tonal, Icon::Close)),
            slot("#s", &IconButton::new(IconButtonKind::Tonal, Icon::House)),
            slot("#s", &Segmented::new(["Low", "Med", "High"])),
            slot("#s", &Segmented::new(["Off", "On"])),
            slot("#s", &Tabs::new(["Overview", "Rooms", "Automations"])),
            slot("#s", &Tabs::new(["Living room", "Entryway", "Bedroom", "Office"])),
            slot("#s", &Symbol::new(SymbolIcon::DoorOpen)),
            slot("#s", &Skeleton::new(SkeletonKind::Media, SkeletonSize::Wide)),
            slot("#s", &Slider::new(50.0, 0.0, 300.0)),
            slot("#s", &dropdown),
            slot("#s", &InputField::new("Password", "hint")),
            slot("#s", &Keyboard::default()),
            slot("#s", &BarChart::new([(40.0, true); 24])),
            scrollbar("#s", &ListView::new([], 472.0, 80.0)),
            slot("#s", &Tile::new(TileKind::Toggle, SymbolIcon::House, "Living room", "Lights")),
            slot("#s", &Tile::new(TileKind::Slider, SymbolIcon::BedDouble, "Bedroom", "Lights").value("65%")),
            slot("#s", &Tile { segmented: Segmented::new(["Low", "Med", "High"]), ..Tile::new(TileKind::Segmented, SymbolIcon::Armchair, "Office", "Fan") }),
            slot("#s", &Tile::new(TileKind::Stepper, SymbolIcon::Droplets, "Climate", "Target").value("22°")),
            slot("#s", &Tile::new(TileKind::Image, SymbolIcon::DoorOpen, "Front door", "Camera")),
        ]
        .into_iter()
        // every glyph of the icon button in every kind, size and state: each a variant of the set
        .chain(Icon::ALL.iter().flat_map(|icon| {
            [IconButtonKind::Tonal, IconButtonKind::Glass, IconButtonKind::Scrim].into_iter().flat_map(move |kind| {
                [false, true].into_iter().flat_map(move |compact| {
                    [true, false].into_iter().map(move |enabled| {
                        let mut b = IconButton::new(kind, *icon);
                        if compact {
                            b = b.compact();
                        }
                        b.press.set_enabled(enabled);
                        slot("#s", &b)
                    })
                })
            })
        }))
        .map(|s| s.items()[0].clone())
        .collect()
    }

    fn names_below(view: &View, names: &mut Vec<String>) {
        names.push(view.name.clone());
        if let Some(View_data_type::Container(container)) =
            view.data.as_ref().and_then(|d| d.view_data_type.as_ref())
        {
            container.children.iter().for_each(|child| names_below(child, names));
        }
    }

    /// The component set `name`: a top-level view when an instance on a screen uses it, else
    /// inside the fetched `Library` frame.
    fn find_set<'a>(view: &'a View, name: &str) -> Option<&'a View> {
        if view.name == name {
            return Some(view);
        }
        match view.data.as_ref().and_then(|d| d.view_data_type.as_ref()) {
            Some(View_data_type::Container(c)) => c.children.iter().find_map(|v| find_set(v, name)),
            _ => None,
        }
    }

    /// The Library's own .dcf (data/library.dcf, design/build-library.sh --dcf): LIBRARY_DCF, else
    /// the copy next to the test (Soong data).
    fn dcf() -> PathBuf {
        if let Ok(path) = std::env::var("LIBRARY_DCF") {
            return path.into();
        }
        let dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let path = dir.join("data/library.dcf");
        assert!(path.exists(), "{} missing; set LIBRARY_DCF", path.display());
        path
    }

    #[test]
    fn a_row_is_as_high_as_the_code_takes_it() {
        let sizes = crate::design::Sizes::load(dcf()).expect("library.dcf");
        let mut rows = vec![
            ListRow::new(ListItemKind::Subheader, "Section", "").size(ListItemSize::Column),
            ListRow::new(ListItemKind::Plain, "A", "B"),
            ListRow::new(ListItemKind::Image, "A", "B").lines(ListItemLines::Three),
        ];
        rows.extend((1..=6).map(|n| ListRow::new(ListItemKind::LeadingText, "A", "B").lines(ListItemLines::with_headline(n)).size(ListItemSize::Column)));
        for row in rows {
            assert_eq!(sizes.height("#list-item", &row.variant()), Some(row.height()), "{}", row.variant());
        }
    }

    #[test]
    fn a_subheader_in_a_list_takes_no_tap() {
        let rows = [ListRow::new(ListItemKind::Subheader, "A", "").pitch(56.0), ListRow::new(ListItemKind::Plain, "Row", "")];
        let out = SlotCustomizer::default().format(list("#l", &ListView::new(rows, 400.0, 80.0)));
        let RenderCustomization::ListCustomization(l) = &out.result else { panic!() };
        let taps = |row: usize| l.items()[row].render_customization().get(&list_row_name("#l", row)).and_then(|c| c.is_interactive());
        assert_eq!((taps(0), taps(1)), (Some(false), Some(true)));
        // a passive row neither
        let rows = [ListRow::new(ListItemKind::Plain, "Nothing found", "").passive(), ListRow::new(ListItemKind::Plain, "Row", "")];
        let out = SlotCustomizer::default().format(list("#l", &ListView::new(rows, 400.0, 80.0)));
        let RenderCustomization::ListCustomization(l) = &out.result else { panic!() };
        let taps = |row: usize| l.items()[row].render_customization().get(&list_row_name("#l", row)).and_then(|c| c.is_interactive());
        assert_eq!((taps(0), taps(1)), (Some(false), Some(true)));
    }

    #[test]
    fn a_row_takes_taps_while_some_of_it_is_in_the_slot_however_high_it_is() {
        use everyware_ui_core::gesture::Gesture;
        use everyware_ui_core::scroller::TOUCH_SLOP;
        // rows of 182 (a headline of six lines) and 72, 8 between them, in a slot 400 high
        let tall = || ListRow::new(ListItemKind::LeadingText, "A long title", "Place").lines(ListItemLines::Seven).size(ListItemSize::Column);
        let short = || ListRow::new(ListItemKind::Plain, "Row", "");
        let rows = [tall().pitch(190.0), short(), tall().pitch(190.0), short(), short(), short()];
        let mut view = ListView::new(rows, 400.0, 80.0);
        let taps = |view: &ListView<ListRow>| {
            let out = SlotCustomizer::default().format(list("#l", view));
            let RenderCustomization::ListCustomization(l) = &out.result else { panic!() };
            (0..6).map(|row| l.items()[row].render_customization().get(&list_row_name("#l", row)).and_then(|c| c.is_interactive())).collect::<Vec<_>>()
        };
        // the first row's top `by` under the slot's top edge, by a drag
        let scrolled = |view: &mut ListView<ListRow>, by: f32| {
            let now = Instant::now();
            view.scroll_to(0);
            view.on(None, Gesture::down(0.0, 300.0), now);
            view.on(None, Gesture::Move { x: 0.0, y: 300.0 - by - TOUCH_SLOP, inside: false }, now);
            view.on(None, Gesture::Up { x: 0.0, y: 300.0 - by - TOUCH_SLOP, inside: false, velocity: 0.0, velocity_x: 0.0 }, now);
            assert_eq!(view.scroll_at(now), by);
        };
        // at the top: the third row (at 270, 182 high) reaches 52 under the slot's bottom and
        // takes taps as the rows above it; the fourth (at 460) is out
        assert_eq!(taps(&view)[..4], [Some(true), Some(true), Some(true), Some(false)]);
        // a fling stopped with the first row 30 under the top: 152 of it show, it takes taps
        scrolled(&mut view, 30.0);
        assert_eq!(taps(&view)[0], Some(true));
        // 170 under the top: 12 of it show, still a row to tap
        scrolled(&mut view, 170.0);
        assert_eq!(taps(&view)[0], Some(true));
        // scrolled by its whole pitch: out of the slot, no taps
        scrolled(&mut view, 190.0);
        assert_eq!(taps(&view)[..2], [Some(false), Some(true)]);
    }

    #[test]
    fn the_scrollbar_is_as_high_as_the_code_takes_it() {
        let sizes = crate::design::Sizes::load(dcf()).expect("library.dcf");
        assert_eq!(sizes.height("#scrollbar", "state=default"), Some(SCROLLBAR_MAX));
    }

    #[test]
    fn the_thumb_runs_along_the_list_s_part_of_the_scrollbar() {
        let at = |y: f32| y / (SCROLLBAR_MAX - SCROLLBAR_THUMB) * 100.0;
        // a list 472 high: the track is the lowest 472 px, the thumb at its top, then its bottom
        assert_eq!(thumb_meter(0.0, 400.0, 472.0), at(SCROLLBAR_MAX - 472.0));
        assert_eq!(thumb_meter(400.0, 400.0, 472.0), at(SCROLLBAR_MAX - SCROLLBAR_THUMB));
        // nothing to scroll: the thumb stays at the top
        assert_eq!(thumb_meter(0.0, 0.0, 472.0), at(SCROLLBAR_MAX - 472.0));
        // a list as high as the screen: the whole component
        assert_eq!(thumb_meter(0.0, 100.0, SCROLLBAR_MAX), 0.0);
        assert_eq!(scrollbar("#s", &ListView::new([], 472.0, 80.0)).items()[0].position, (0.0, 472.0 - SCROLLBAR_MAX));
        // no list (a design without the slot): the thumb stays on the component
        assert_eq!(thumb_meter(0.0, 0.0, 0.0), 100.0);
        // the clone reaches above its slot: it takes no taps
        assert!(!scrollbar("#s", &ListView::new([], 472.0, 80.0)).items()[0].taps);
    }

    #[test]
    fn every_part_the_code_sets_is_in_every_variant_of_the_figma_set() {
        let (_, doc) = load_design_def(dcf()).expect("library.dcf");
        let mut missing = Vec::new();
        for item in library() {
            let set = doc
                .views
                .get(&NodeQuery::component_set(item.component()).encode())
                .or_else(|| doc.views.values().find_map(|v| find_set(v, item.component())))
                .unwrap_or_else(|| panic!("component set {} is not in the .dcf: the Library of the design has no such set. Add it in ui-lib's design/components-patch.js (ui-component skill, step 2), build and import the .fig, fetch, then run again; or the code names a set the design calls differently.", item.component()));
            let Some(View_data_type::Container(variants)) =
                set.data.as_ref().and_then(|d| d.view_data_type.as_ref())
            else {
                panic!("{} has no variants", item.component())
            };
            if !variants.children.iter().any(|v| v.name == item.variant()) {
                missing.push(format!("{}: the code produces the variant {:?} and the design has no such variant. Either add it to the set in components.json and components-patch.js, or stop producing it in the component's variant() (the property values and their order must match the design).", item.component(), item.variant()));
            }
            let mut anywhere = Vec::new();
            for variant in &variants.children {
                let mut names = Vec::new();
                names_below(variant, &mut names);
                for part in item.part_names() {
                    if !names.iter().any(|n| n == part) {
                        missing.push(format!("{} / {}: the code sets the part {part} and the variant has no layer of that name. Name the layer {part} in this variant too (every variant of a set carries the same part names), or drop it from the binding in harry/src/slot.rs.", item.component(), variant.name));
                    }
                }
                anywhere.extend(names);
            }
            for part in item.optional_part_names() {
                if !anywhere.iter().any(|n| n == part) {
                    missing.push(format!("{}: the optional part {part} is in no variant of the design: the code could never set it. Add the layer to the variants that show it, or drop it from the binding.", item.component()));
                }
            }
        }
        assert!(missing.is_empty(), "the code and the Figma library disagree ({} problems); the .dcf is {} (LIBRARY_DCF):\n{}", missing.len(), dcf().display(), missing.join("\n"));
    }
}

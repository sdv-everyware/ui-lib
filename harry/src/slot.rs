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
use everyware_ui_core::list_item::{ListItem as ListRow, GLYPHS};
use everyware_ui_core::list_view::Row;
use everyware_ui_core::list_view::{ListView, Scroll};
use everyware_ui_core::radio::Radio;
use everyware_ui_core::segmented::Segmented;
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
/// The id prefix of the rows of a list slot: "r0", "r1", ...
const ROW_ID: &str = "r";
/// A row partly scrolled under the slot's top edge still takes taps up to this much (px).
const ROW_SLACK: f32 = 24.0;
/// How far a ribbon (a horizontal list) lifts its chosen card, in pixels; its slot is that
/// much higher than a card.
pub const RIBBON_LIFT: f32 = 20.0;

/// What harry puts between the list name and the item id in the clone's name.
const CLONE_MARK: &str = "@$List-";
/// A sweeping part (a skeleton's shimmer) moves this often: 30 frames a second, not every
/// frame, because each frame redraws the whole screen.
const SWEEP_FRAME: Duration = Duration::from_millis(33);

/// A UI model field for a slot: a list with the slot's component.
pub type Slot = List<ComponentItem>;

/// The node name of the clone in `slot`.
pub fn clone_name(slot: &str) -> String {
    format!("{slot}{CLONE_MARK}{ITEM_ID}")
}

/// The node name of row `row` of the list in `slot`.
pub fn row_name(slot: &str, row: usize) -> String {
    format!("{slot}{CLONE_MARK}{ROW_ID}{row}")
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
        Some((slot, id)) => match id.strip_prefix(ROW_ID).and_then(|r| r.parse::<usize>().ok()) {
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

    /// A row of a scrolling list: `scroll` moves it, `view_h` is the slot's height.
    fn scrolled(mut self, scroll: Scroll, view_h: f32) -> Self {
        self.scroll = Some((scroll, view_h));
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
                // the row's place along the list, for the hit test below
                let y = if item.horizontal { x } else { y };
                // harry hits by bounds without clipping: a row scrolled out of the slot
                // takes no taps.
                let shown = item.thumb.is_some() || scroll.map_or(true, |(_, view_h)| y > -ROW_SLACK && y < view_h);
                c.set_interactive(root.clone(), shown && item.taps && item.shown != Some(false));
                if let Some(on) = item.shown {
                    c.set_opacity(root.clone(), if on { 1.0 } else { 0.0 });
                }
                if let Some((node, max, shown)) = &scrollbar {
                    let y = scroll.map_or(0.0, |(s, _)| s);
                    c.set_meter_value(node.clone(), if *max > 0.0 { (y / max * 100.0).clamp(0.0, 100.0) } else { 0.0 });
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
    let (pitch, scroll, view_h) = (view.pitch(), view.scroll(), view.view_h());
    List::new(
        view.rows()
            .iter()
            .enumerate()
            .map(|(row, component)| {
                let item = ComponentItem::new(slot, R::SET, component.variant());
                // a ribbon lifts its chosen card (harry has no scale: the card moves up instead)
                let item = if view.is_horizontal() {
                    item.row_x(row, row as f32 * pitch).lifted(if component.is_selected() { RIBBON_LIFT } else { 0.0 })
                } else {
                    item.row(row, row as f32 * pitch)
                };
                let item = item
                    .scrolled(scroll, view_h)
                    .animation(animation(component.transition()));
                component.parts(item)
            })
            .collect(),
    )
}

/// The slot field's value for the scrollbar of `view` in the slot frame `slot`: the
/// library's `#scrollbar` of the list's length, its thumb at the scroll, shown while the
/// content moves and SCROLLBAR_DELAY after.
pub fn scrollbar(slot: &str, view: &ListView) -> Slot {
    let item = ComponentItem::new(slot, "#scrollbar", format!("length={}", view.view_h() as i32))
        .scrolled(view.scroll(), view.view_h());
    List::new(vec![ComponentItem { thumb: Some(("#scrollbar/thumb".to_string(), view.max_scroll())), ..item }])
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
    /// Title, subtitle, the pressed layer, and the two icon slots: every glyph of the stack
    /// at opacity 0 but the chosen one (display-only; a variant cannot change inside an item).
    /// With an image: the picture (a registered image; an unknown key keeps the placeholder) and
    /// the trailing text. The subtitle is not in lines one, the image parts only in kind image.
    fn parts(&self, item: ComponentItem) -> ComponentItem {
        let mut item = item
            .text("#list-item/title", self.title.clone())
            .text("#list-item/subtitle", self.subtitle.clone())
            .optional("#list-item/subtitle")
            .text("#list-item/trailing-text", self.trailing_text.clone())
            .optional("#list-item/trailing-text")
            .optional("#list-item/image");
        if let Some(image) = self.image.as_deref().and_then(crate::images::get) {
            item = item.image("#list-item/image", image);
        }
        for (slot, chosen) in [("leading", self.leading), ("trailing", self.trailing)] {
            for glyph in GLYPHS {
                let node = format!("#list-item/{slot}/{glyph}");
                let on = if chosen == Some(glyph) { 100.0 } else { 0.0 };
                item = item.opacity(node.clone(), Meter::new(on, None)).optional(node);
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
        sleep(Duration::from_millis(260)); // the value transition is 200 ms
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
            slot("#s", &IconButton::new(IconButtonKind::Tonal, Icon::Close)),
            slot("#s", &IconButton::new(IconButtonKind::Tonal, Icon::House)),
            slot("#s", &Segmented::new(["Low", "Med", "High"])),
            slot("#s", &Segmented::new(["Off", "On"])),
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

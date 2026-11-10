//! Tile (#tile): a device's card. A tap on the card opens the device (its sheet); the control
//! inside it (a toggle, a slider, a segmented button, a stepper's minus and plus) takes its own
//! taps and changes the device right there. The variant is the kind of control the card shows
//! and the state (default, disabled); pressed is #tile/pressed faded in. The card's texts
//! (#tile/title, /subtitle, /value), its symbol (#tile/symbol/<icon>, one of a stack lit by
//! opacity) and the control are parts the binding sets.
//!
//! The control's layers carry the same names in every tile of a screen (#tile/toggle,
//! #tile/slider, #segmented/segment-<i>, #tile/minus, #tile/plus). harry hands a touch to every
//! interactive node under the finger, top-most first, until one is consumed: the control first,
//! then the card (a list's clone root, with its row). `everyware_ui::Input` pairs the two into
//! one gesture on "<slot>/<row>/<part>", so a tile knows which of its parts was touched
//! (`on_part`).

use crate::gesture::Gesture;
use crate::list_view::Row;
use crate::pressable::Pressable;
use crate::segmented::{Segmented, SegmentedEvent};
use crate::slider::{Slider, SliderEvent};
use crate::symbol::SymbolIcon;
use crate::toggle::{Toggle, ToggleEvent};
use crate::transitions::{Change, ComponentKind};
use crate::Component;

/// The kind property of #tile: the control the card shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TileKind {
    /// `toggle`: an on/off device (a light).
    #[default]
    Toggle,
    /// `slider`: a level (a dimmer).
    Slider,
    /// `segmented`: one of 2 to 4 modes (a fan).
    Segmented,
    /// `stepper`: a value with minus and plus (a thermostat), the value in #tile/value.
    Stepper,
    /// `image`: a picture (a camera's last frame) in #tile/image.
    Image,
}

impl TileKind {
    /// The value of the kind property.
    pub fn name(self) -> &'static str {
        match self {
            TileKind::Toggle => "toggle",
            TileKind::Slider => "slider",
            TileKind::Segmented => "segmented",
            TileKind::Stepper => "stepper",
            TileKind::Image => "image",
        }
    }
}

/// A device's card.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tile {
    pub kind: TileKind,
    /// The device's symbol: the glyph of the stack that is lit.
    pub symbol: SymbolIcon,
    pub title: String,
    pub subtitle: String,
    /// The value text (a slider's level, a stepper's value; empty for the others).
    pub value: String,
    /// The toggle's state (kind toggle).
    pub toggle: Toggle,
    /// The slider's level (kind slider; its track is not touched, any x will do).
    pub slider: Slider,
    /// The modes and the chosen one (kind segmented).
    pub segmented: Segmented,
    /// The key of a registered picture (kind image, everyware_ui::images).
    pub image: Option<String>,
    /// Pressed and disabled.
    pub press: Pressable,
    /// The stepper's minus and plus buttons (kind stepper).
    pub minus: Pressable,
    pub plus: Pressable,
    /// The chosen card of a ribbon (a horizontal ListView lifts it).
    pub selected: bool,
}

/// What a tile reports.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileEvent {
    /// The card was tapped: open the device.
    Opened,
    /// The toggle was flipped (kind toggle): the new value.
    Toggled(bool),
    /// The slider is being dragged (kind slider): the level under the finger.
    Moving(f32),
    /// The slider was released (kind slider): the level.
    Level(f32),
    /// A segment was chosen (kind segmented): its index.
    Mode(usize),
    /// Minus (-1) or plus (+1) was tapped (kind stepper).
    Step(i8),
}

/// Where the parts sit inside the tile (the Library's design, components-patch.js): the
/// padding, and the slider instance's width. The slider's track is inset TRACK_INSET in it.
pub const TILE_PAD: f32 = 24.0;
pub const TILE_SLIDER_W: f32 = 243.0;
/// The names of the parts a finger can touch inside a tile, as harry reports them.
pub const PART_TOGGLE: &str = "#tile/toggle";
pub const PART_SLIDER: &str = "#tile/slider";
pub const PART_MINUS: &str = "#tile/minus";
pub const PART_PLUS: &str = "#tile/plus";
const PART_SEGMENT: &str = "#segmented/segment-";

impl Tile {
    /// A card of `kind` for a device with `symbol`, `title` (the device) and `subtitle` (its room
    /// or state).
    pub fn new(kind: TileKind, symbol: SymbolIcon, title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        // the slider's track where the design draws it, in the tile's own pixels
        let slider = Slider::in_slot(0.0, TILE_PAD, TILE_SLIDER_W);
        Tile { kind, symbol, title: title.into(), subtitle: subtitle.into(), slider, ..Default::default() }
    }

    /// The value text.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// A tap on the card: Opened when the finger goes up over it.
    pub fn on(&mut self, gesture: Gesture) -> Option<TileEvent> {
        self.press.on(gesture).then_some(TileEvent::Opened)
    }

    /// A gesture on the card (`part` None) or on a part inside it, by the name harry reports
    /// (`PART_TOGGLE`, `PART_SLIDER`, `#segmented/segment-<i>`, `PART_MINUS`, `PART_PLUS`).
    /// `x`, `y` is where the tile's top left is on the display (its row's place in the list
    /// plus the slot's), so the slider can read the finger against its track.
    pub fn on_part(&mut self, part: Option<&str>, gesture: Gesture, x: f32, y: f32) -> Option<TileEvent> {
        let Some(part) = part else { return self.on(gesture) };
        if !self.press.enabled {
            return None;
        }
        match part {
            PART_TOGGLE if self.kind == TileKind::Toggle => self.toggle.on(gesture).map(|ToggleEvent::Toggled(v)| TileEvent::Toggled(v)),
            PART_SLIDER if self.kind == TileKind::Slider => match self.slider.on(gesture.shifted(-x, -y))? {
                SliderEvent::Moving(v) => Some(TileEvent::Moving(v)),
                SliderEvent::Changed(v) => Some(TileEvent::Level(v)),
            },
            PART_MINUS if self.kind == TileKind::Stepper => self.minus.on(gesture).then_some(TileEvent::Step(-1)),
            PART_PLUS if self.kind == TileKind::Stepper => self.plus.on(gesture).then_some(TileEvent::Step(1)),
            _ if self.kind == TileKind::Segmented => {
                let i = part.strip_prefix(PART_SEGMENT)?.parse::<usize>().ok()?.checked_sub(1)?;
                self.segmented.on(i, gesture).map(|SegmentedEvent::Selected(i)| TileEvent::Mode(i))
            }
            _ => None,
        }
    }
}

/// A tile is a row of a list: a ribbon of cards (ListView::horizontal) scrolls them, a tap
/// chooses one.
impl Row for Tile {
    fn key(&self) -> &str {
        &self.title
    }
    fn tap(&mut self, gesture: Gesture) -> bool {
        self.on(gesture) == Some(TileEvent::Opened)
    }
    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
    fn is_selected(&self) -> bool {
        self.selected
    }
    fn keep_state_of(&mut self, before: &Self) {
        self.press = before.press;
        self.selected = before.selected;
    }
}

impl Component for Tile {
    const KIND: ComponentKind = ComponentKind::Tile;

    fn variant(&self) -> String {
        format!("kind={}, state={}", self.kind.name(), if self.press.enabled { "default" } else { "disabled" })
    }

    fn last_change(&self) -> Option<Change> {
        self.press.last_change
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tap_opens_the_device_and_the_variant_is_the_kind() {
        let mut tile = Tile::new(TileKind::Slider, SymbolIcon::BedDouble, "Bedroom", "Lights").value("65%");
        assert_eq!(tile.variant(), "kind=slider, state=default");
        assert_eq!(tile.on(Gesture::DOWN), None);
        assert_eq!(tile.on(Gesture::UP), Some(TileEvent::Opened));
        tile.press.enabled = false;
        assert_eq!(tile.variant(), "kind=slider, state=disabled");
        assert_eq!((tile.on(Gesture::DOWN), tile.on(Gesture::UP)), (None, None));
    }

    #[test]
    fn a_part_takes_its_own_taps_and_the_card_only_opens() {
        use crate::slider::TRACK_INSET;
        // the card: a tap opens, nothing else changes
        let mut light = Tile::new(TileKind::Toggle, SymbolIcon::House, "Hall", "On");
        assert_eq!(light.on_part(None, Gesture::DOWN, 0.0, 0.0), None);
        assert_eq!(light.on_part(None, Gesture::UP, 0.0, 0.0), Some(TileEvent::Opened));
        assert!(!light.toggle.value);
        // the toggle inside it flips
        light.on_part(Some(PART_TOGGLE), Gesture::DOWN, 0.0, 0.0);
        assert_eq!(light.on_part(Some(PART_TOGGLE), Gesture::UP, 0.0, 0.0), Some(TileEvent::Toggled(true)));
        // a part of another kind does nothing
        assert_eq!(light.on_part(Some(PART_SLIDER), Gesture::DOWN, 0.0, 0.0), None);
        // the slider reads the finger against its track in the tile's place: a tile at x 558
        // (row 1 of a ribbon at 96, pitch 462) has its track from 558 + 24 + 20 on
        let mut dimmer = Tile::new(TileKind::Slider, SymbolIcon::BedDouble, "Bedroom", "Lights");
        let tile_x = 558.0;
        let track_x = tile_x + TILE_PAD + TRACK_INSET;
        let track_w = TILE_SLIDER_W - 2.0 * TRACK_INSET;
        assert_eq!(dimmer.on_part(Some(PART_SLIDER), Gesture::down(track_x + track_w / 2.0, 0.0), tile_x, 0.0), Some(TileEvent::Moving(50.0)));
        assert_eq!(dimmer.on_part(Some(PART_SLIDER), Gesture::up(track_x + track_w, 0.0), tile_x, 0.0), Some(TileEvent::Level(100.0)));
        // the segments by their number, the stepper's buttons
        let mut fan = Tile::new(TileKind::Segmented, SymbolIcon::Armchair, "Fan", "Office");
        fan.segmented = Segmented::new(["Low", "Med", "High"]);
        fan.on_part(Some("#segmented/segment-3"), Gesture::DOWN, 0.0, 0.0);
        assert_eq!(fan.on_part(Some("#segmented/segment-3"), Gesture::UP, 0.0, 0.0), Some(TileEvent::Mode(2)));
        assert_eq!(fan.on_part(Some("#segmented/segment-9"), Gesture::UP, 0.0, 0.0), None);
        let mut climate = Tile::new(TileKind::Stepper, SymbolIcon::Droplets, "Climate", "Whole house");
        climate.on_part(Some(PART_PLUS), Gesture::DOWN, 0.0, 0.0);
        assert_eq!(climate.on_part(Some(PART_PLUS), Gesture::UP, 0.0, 0.0), Some(TileEvent::Step(1)));
        climate.on_part(Some(PART_MINUS), Gesture::DOWN, 0.0, 0.0);
        assert_eq!(climate.on_part(Some(PART_MINUS), Gesture::UP, 0.0, 0.0), Some(TileEvent::Step(-1)));
        // disabled: no part answers
        climate.press.enabled = false;
        assert_eq!(climate.on_part(Some(PART_PLUS), Gesture::UP, 0.0, 0.0), None);
    }

    #[test]
    fn every_kind_is_a_variant_of_the_set() {
        for (kind, name) in [
            (TileKind::Toggle, "toggle"),
            (TileKind::Slider, "slider"),
            (TileKind::Segmented, "segmented"),
            (TileKind::Stepper, "stepper"),
            (TileKind::Image, "image"),
        ] {
            assert_eq!(kind.name(), name);
            assert_eq!(Tile::new(kind, SymbolIcon::House, "t", "s").variant(), format!("kind={name}, state=default"));
        }
        assert_eq!(TileKind::default(), TileKind::Toggle);
    }

    #[test]
    fn a_press_and_its_release_are_the_changes_a_transition_plays() {
        let mut tile = Tile::new(TileKind::Toggle, SymbolIcon::House, "Hall", "On");
        assert_eq!(tile.last_change(), None);
        tile.on(Gesture::DOWN);
        assert_eq!(tile.last_change(), Some(Change::Press));
        tile.on(Gesture::UP);
        assert_eq!(tile.last_change(), Some(Change::Release));
    }

    #[test]
    fn a_ribbon_of_tiles_chooses_by_tap_and_keeps_the_choice_over_a_refresh() {
        use crate::list_view::{ListEvent, ListView};
        use std::time::Instant;
        let cards = || ["Hall", "Bedroom", "Office"].map(|t| Tile::new(TileKind::Toggle, SymbolIcon::House, t, "On"));
        let mut ribbon = ListView::new(cards(), 1728.0, 462.0).horizontal();
        let now = Instant::now();
        // a tap on the second card: the row is chosen, the app marks it (it lifts)
        assert_eq!(ribbon.on(Some(1), Gesture::down(600.0, 50.0), now), None);
        assert_eq!(ribbon.on(Some(1), Gesture::up(601.0, 50.0), now), Some(ListEvent::Chosen(1)));
        ribbon.select(Some(1));
        assert_eq!(ribbon.selected(), Some(1));
        assert!(ribbon.rows()[1].is_selected() && !ribbon.rows()[0].is_selected());
        // new data for the same devices (keyed by title): the chosen card stays chosen
        ribbon.set_rows(cards());
        assert_eq!(ribbon.selected(), Some(1));
        assert_eq!(ribbon.rows()[1].key(), "Bedroom");
        // a press survives the refresh as well
        ribbon.on(Some(2), Gesture::down(1000.0, 50.0), now);
        ribbon.set_rows(cards());
        assert!(ribbon.rows()[2].press.pressed);
    }
}

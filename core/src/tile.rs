//! Tile (#tile): a device's card, the whole card a touch target that opens the device (its
//! sheet). The variant is the kind of control the card shows and the state (default,
//! disabled); pressed is #tile/pressed faded in. The card's texts (#tile/title, /subtitle,
//! /value), its symbol (#tile/symbol/<icon>, one of a stack lit by opacity) and the control
//! inside it (a toggle, a slider, a segmented button, a stepper's minus and plus, a picture) are
//! parts the binding sets. The control shows the device's state and takes no taps of its own:
//! its layers carry the same names in every tile of a screen, so a tap could not tell the tiles
//! apart; the device's sheet changes it.

use crate::gesture::Gesture;
use crate::list_view::Row;
use crate::pressable::Pressable;
use crate::segmented::Segmented;
use crate::slider::Slider;
use crate::symbol::SymbolIcon;
use crate::toggle::Toggle;
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
    /// The chosen card of a ribbon (a horizontal ListView lifts it).
    pub selected: bool,
}

/// What a tile reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileEvent {
    /// The card was tapped: open the device.
    Opened,
}

impl Tile {
    /// A card of `kind` for a device with `symbol`, `title` (the device) and `subtitle` (its room
    /// or state).
    pub fn new(kind: TileKind, symbol: SymbolIcon, title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Tile { kind, symbol, title: title.into(), subtitle: subtitle.into(), ..Default::default() }
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

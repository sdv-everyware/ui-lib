//! Symbol (#symbol): the tile with a room's or a device's symbol (48 x 48, the soft accent, the
//! glyph in ink). The variant is the symbol; display only.

use crate::transitions::{Change, ComponentKind};
use crate::Component;

/// The icon property of #symbol: the design's icons (lucide names).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SymbolIcon {
    /// `house`
    #[default]
    House,
    /// `door-open`
    DoorOpen,
    /// `bed-double`
    BedDouble,
    /// `armchair`
    Armchair,
    /// `droplets`
    Droplets,
}

impl SymbolIcon {
    /// Every icon, in the order of the set's property (the stacks of glyphs in #tile, #symbol).
    pub const ALL: &'static [SymbolIcon] = &[SymbolIcon::House, SymbolIcon::DoorOpen, SymbolIcon::BedDouble, SymbolIcon::Armchair, SymbolIcon::Droplets];

    /// The value of the set's `icon` property.
    pub fn name(self) -> &'static str {
        match self {
            SymbolIcon::House => "house",
            SymbolIcon::DoorOpen => "door-open",
            SymbolIcon::BedDouble => "bed-double",
            SymbolIcon::Armchair => "armchair",
            SymbolIcon::Droplets => "droplets",
        }
    }
}

/// A symbol tile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Symbol {
    /// Which symbol.
    pub icon: SymbolIcon,
}

impl Symbol {
    /// The tile of `icon`.
    pub fn new(icon: SymbolIcon) -> Self {
        Symbol { icon }
    }
}

impl Component for Symbol {
    const KIND: ComponentKind = ComponentKind::Symbol;

    fn variant(&self) -> String {
        format!("icon={}", self.icon.name())
    }

    fn last_change(&self) -> Option<Change> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_icons_are_the_symbol_set_of_the_spec() {
        let spec: serde_json::Value = serde_json::from_str(include_str!("../../components.json")).unwrap();
        let icons: Vec<&str> = spec["components"]["symbol"]["props"]["icon"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
        let ours = [SymbolIcon::House, SymbolIcon::DoorOpen, SymbolIcon::BedDouble, SymbolIcon::Armchair, SymbolIcon::Droplets].map(SymbolIcon::name);
        assert_eq!(icons, ours);
        assert_eq!(Symbol::new(SymbolIcon::BedDouble).variant(), "icon=bed-double");
    }
}

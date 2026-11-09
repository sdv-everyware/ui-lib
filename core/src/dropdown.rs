//! Dropdown (#dropdown). The variant is only the state (default, disabled); the list
//! (#dropdown/list) is always in the component and fades by opacity, the selection mark of
//! an item (#dropdown/item/N/selected) likewise.

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::transitions::{self, Change, ComponentKind, Transition};
use crate::Component;

/// A dropdown with its items.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dropdown {
    /// The items of the list.
    pub items: Vec<String>,
    /// Index of the selected item.
    pub selected: usize,
    /// The list is shown.
    pub open: bool,
    /// Disabled (the field opens on tap, pressed is not shown).
    pub press: Pressable,
}

/// What a dropdown reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropdownEvent {
    /// The user picked the item with this index.
    Selected(usize),
}

impl Dropdown {
    /// A closed dropdown with these items, the first selected.
    pub fn new<S: Into<String>>(items: impl IntoIterator<Item = S>) -> Self {
        Dropdown { items: items.into_iter().map(Into::into).collect(), ..Default::default() }
    }

    /// The text shown in the field.
    pub fn value(&self) -> &str {
        self.items.get(self.selected).map(String::as_str).unwrap_or("")
    }

    /// Applies a gesture on the field: a tap opens or closes the list.
    pub fn on_field(&mut self, gesture: Gesture) {
        if self.press.on(gesture) {
            self.open = !self.open;
        }
    }

    /// Applies a gesture on item `index`: a tap on the open list selects it and closes.
    /// Taps on the closed (transparent) list are ignored.
    pub fn on_item(&mut self, index: usize, gesture: Gesture) -> Option<DropdownEvent> {
        if !self.open || !matches!(gesture, Gesture::Up { inside: true, .. }) || index >= self.items.len() {
            return None;
        }
        self.open = false;
        let changed = self.selected != index;
        self.selected = index;
        changed.then_some(DropdownEvent::Selected(index))
    }

    /// Closes the list without a choice (a tap outside).
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Opacity target of the list: 100 open, 0 closed.
    pub fn list_target(&self) -> f32 {
        if self.open {
            100.0
        } else {
            0.0
        }
    }

    /// Opacity target of the selection mark of item `index`.
    pub fn mark_target(&self, index: usize) -> f32 {
        if index == self.selected {
            100.0
        } else {
            0.0
        }
    }

    /// How the list fades: open and close have their own curves.
    pub fn list_transition(&self) -> Option<Transition> {
        transitions::transition(Self::KIND, if self.open { Change::Open } else { Change::Close })
    }
}

impl Component for Dropdown {
    const KIND: ComponentKind = ComponentKind::Dropdown;

    fn variant(&self) -> String {
        format!("state={}", if self.press.enabled { "default" } else { "disabled" })
    }

    fn last_change(&self) -> Option<Change> {
        self.press.last_change
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_pick_close_without_variant_change() {
        let mut d = Dropdown::new(["Living room", "Bedroom", "Office"]);
        d.on_field(Gesture::DOWN);
        d.on_field(Gesture::UP);
        assert_eq!((d.variant().as_str(), d.list_target()), ("state=default", 100.0));
        assert_eq!(d.list_transition().unwrap().duration.as_millis(), 200);
        assert_eq!(d.on_item(1, Gesture::UP), Some(DropdownEvent::Selected(1)));
        assert_eq!((d.list_target(), d.value(), d.mark_target(1)), (0.0, "Bedroom", 100.0));
        assert_eq!(d.on_item(0, Gesture::UP), None, "closed list ignores taps");
    }
}

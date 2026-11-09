//! A form: a sheet (sheet.rs) with input fields. A tap on a field moves the focus there; the
//! screen's keyboard (dock.rs) opens while a field has the focus and types into it through the
//! form. The sheet's variant and movement are the form's; what a submitted text means (connect,
//! search, rename) is the app's.
//!
//! The fields are named by their slots on the screen (`#wifi/password`): the form routes a
//! gesture by the node it came from, as harry reports it.

use crate::gesture::Gesture;
use crate::input_field::InputField;
use crate::keyboard::{key_of_node, Keyboard, Outcome};
use crate::sheet::{Layout, Sheet};
use crate::transitions::{Change, ComponentKind};
use crate::Component;
use std::time::Instant;

/// A field of the form: the slot it sits in and the input field itself.
#[derive(Clone, Debug)]
struct Field {
    node: String,
    input: InputField,
}

/// A sheet with input fields.
#[derive(Clone, Debug)]
pub struct Form {
    /// The sheet: closed or open over the screen, its layout, the card's place.
    pub sheet: Sheet,
    fields: Vec<Field>,
}

/// What the form reports; the index is the field's, in the order given to `new`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormEvent {
    /// A tap gave a field the focus (the keyboard opens).
    Focused(usize),
    /// A key changed a field's text.
    Edited(usize),
    /// Done on the keyboard: the field's text is submitted.
    Submitted(usize),
}

impl Form {
    /// A closed form in a sheet of this layout, with these fields, each named by its slot on
    /// the screen.
    pub fn new(layout: Layout, fields: impl IntoIterator<Item = (impl Into<String>, InputField)>) -> Self {
        Form {
            sheet: Sheet::new(layout),
            fields: fields.into_iter().map(|(node, input)| Field { node: node.into(), input }).collect(),
        }
    }

    /// Opens it, nothing in focus; the change animates with the open transition.
    pub fn open(&mut self) {
        self.sheet.open();
    }

    /// Closes it and takes the focus away (the keyboard closes with it); the change animates
    /// with the close transition.
    pub fn close(&mut self) {
        self.sheet.close();
        self.blur();
    }

    /// Whether the sheet is open.
    pub fn is_open(&self) -> bool {
        self.sheet.is_open()
    }

    /// Empties the fields (texts and errors); placeholders and helpers stay.
    pub fn clear(&mut self) {
        for field in &mut self.fields {
            field.input.text.clear();
            field.input.error = None;
        }
    }

    /// A field, by its index in `new`.
    pub fn field(&self, index: usize) -> &InputField {
        &self.fields[index].input
    }

    /// A field to change, by its index in `new`.
    pub fn field_mut(&mut self, index: usize) -> &mut InputField {
        &mut self.fields[index].input
    }

    /// The field with the focus; the keyboard dock follows it.
    pub fn focused(&self) -> Option<usize> {
        self.fields.iter().position(|f| f.input.focused)
    }

    /// Gives a field the focus; a disabled field does not take it.
    pub fn focus(&mut self, index: usize, now: Instant) {
        for (i, field) in self.fields.iter_mut().enumerate() {
            if i == index {
                field.input.focus(now);
            } else {
                field.input.blur();
            }
        }
    }

    /// Takes the focus away from every field.
    pub fn blur(&mut self) {
        for field in &mut self.fields {
            field.input.blur();
        }
    }

    /// Applies a gesture from a node of the form: a tap on a field moves the focus there, a
    /// key (of the screen's keyboard) goes to the field in focus. None for a node that is not
    /// a field or a key, while a key is held, for Shift / mode, and for a key without a field
    /// in focus.
    pub fn on(&mut self, node: &str, gesture: Gesture, now: Instant, keyboard: &mut Keyboard) -> Option<FormEvent> {
        if key_of_node(node).is_some() {
            let index = self.focused()?;
            let outcome = keyboard.on(node, gesture, &mut self.fields[index].input.text)?;
            return Some(match outcome {
                Outcome::Edited => {
                    self.fields[index].input.edited(now);
                    FormEvent::Edited(index)
                }
                Outcome::Done => FormEvent::Submitted(index),
            });
        }
        let index = self.fields.iter().position(|f| f.node == node)?;
        self.fields[index].input.on(gesture, now)?;
        for (i, field) in self.fields.iter_mut().enumerate() {
            if i != index {
                field.input.blur();
            }
        }
        Some(FormEvent::Focused(index))
    }
}

impl Component for Form {
    const KIND: ComponentKind = ComponentKind::Form;

    fn variant(&self) -> String {
        self.sheet.variant()
    }

    fn last_change(&self) -> Option<Change> {
        self.sheet.last_change()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::KeyboardDock;
    use crate::keyboard::Key;

    fn form() -> Form {
        Form::new(Layout::Dialog, [("#f/name", InputField::new("Name", "")), ("#f/city", InputField::new("City", ""))])
    }

    fn tap(form: &mut Form, dock: &mut KeyboardDock, node: &str, now: Instant) -> Option<FormEvent> {
        form.on(node, Gesture::DOWN, now, &mut dock.keyboard);
        let event = form.on(node, Gesture::UP, now, &mut dock.keyboard);
        // the screen's rule, after every action
        dock.follow(form.focused().is_some());
        event
    }

    fn key_node(key: Key) -> String {
        Keyboard::key_nodes().into_iter().find(|n| key_of_node(n) == Some(key)).expect("key on the keyboard")
    }

    #[test]
    fn the_keyboard_dock_follows_the_focus() {
        let now = Instant::now();
        let (mut f, mut d) = (form(), KeyboardDock::new());
        assert_eq!((f.variant().as_str(), d.variant().as_str()), ("layout=dialog, state=closed", "state=closed"));
        f.open();
        assert_eq!((f.variant().as_str(), f.last_change()), ("layout=dialog, state=open", Some(Change::Open)));
        assert_eq!(tap(&mut f, &mut d, "#f/city", now), Some(FormEvent::Focused(1)));
        assert_eq!((d.variant().as_str(), f.focused()), ("state=open", Some(1)));
        // the other field takes the focus over, the dock stays
        assert_eq!(tap(&mut f, &mut d, "#f/name", now), Some(FormEvent::Focused(0)));
        assert_eq!((f.focused(), f.field(1).focused, d.open()), (Some(0), false, true));
        f.close();
        d.follow(f.focused().is_some());
        assert_eq!((f.variant().as_str(), f.focused(), d.variant().as_str()), ("layout=dialog, state=closed", None, "state=closed"));
    }

    #[test]
    fn keys_type_into_the_field_in_focus_and_done_submits() {
        let now = Instant::now();
        let (mut f, mut d) = (form(), KeyboardDock::new());
        f.open();
        let letter = Keyboard::key_nodes()
            .into_iter()
            .find(|n| matches!(key_of_node(n), Some(Key::Char(..))))
            .expect("a character key");
        // no focus: keys go nowhere
        assert_eq!(tap(&mut f, &mut d, &letter, now), None);
        tap(&mut f, &mut d, "#f/name", now);
        assert_eq!(tap(&mut f, &mut d, &letter, now), Some(FormEvent::Edited(0)));
        assert_eq!(f.field(0).text.chars().count(), 1);
        assert_eq!(tap(&mut f, &mut d, &key_node(Key::Done), now), Some(FormEvent::Submitted(0)));
        f.clear();
        assert_eq!(f.field(0).text, "");
    }

    #[test]
    fn nodes_of_the_screen_around_are_not_its() {
        let now = Instant::now();
        let (mut f, mut d) = (form(), KeyboardDock::new());
        f.open();
        assert_eq!(tap(&mut f, &mut d, "#f/connect", now), None);
    }

    #[test]
    fn transitions_from_the_spec() {
        let mut f = form();
        f.open();
        assert_eq!(f.transition().map(|t| t.duration.as_millis()), Some(550));
        f.close();
        assert_eq!(f.transition().map(|t| t.duration.as_millis()), Some(400));
    }
}

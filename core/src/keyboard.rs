/*
 * The on-screen keyboard: modes, key labels, what a key does to
 * the text. The Figma component "#keyboard" has one fixed layout (rows of 10, 9 and 7 keys
 * plus Shift, Delete, mode, space, Done); the labels come from here, per mode.
 */

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::transitions::{Change, ComponentKind, Transition};
use crate::Component;
use std::collections::HashMap;

/// The key rows with a character on them, top to bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// 10 keys.
    One,
    /// 9 keys.
    Two,
    /// 7 keys, between Shift and Delete.
    Three,
}

impl Row {
    /// Keys in the row.
    pub const fn len(self) -> usize {
        match self {
            Row::One => 10,
            Row::Two => 9,
            Row::Three => 7,
        }
    }
}

/// A key of the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// A character key: row and index from 0.
    Char(Row, usize),
    /// Upper case for one letter, or the other symbol page.
    Shift,
    /// Removes the last character.
    Delete,
    /// 123 / abc.
    Mode,
    /// A space.
    Space,
    /// Finished with the field.
    Done,
}

/// What the keys show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// Lower-case letters.
    #[default]
    Lower,
    /// After Shift, for one character.
    Upper,
    /// Digits and the common symbols.
    Symbols,
    /// The other symbols (Shift in Symbols).
    MoreSymbols,
}

const LOWER: [&str; 3] = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
const SYMBOLS: [&str; 3] = ["1234567890", "-/:;()$&@", ".,?!'\"_"];
const MORE_SYMBOLS: [&str; 3] = ["[]{}#%^*+=", "\\|~<>`€£¥", ".,?!'\"_"];

/// WPA2/WPA3 passphrases are 8 to 63 characters.
pub const MAX_LEN: usize = 63;

/// Keyboard state: the mode and which keys are pressed; the text belongs to the field being
/// edited.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Keyboard {
    /// What the keys show.
    pub mode: Mode,
    /// Pressed state per key node ("#kbd/r1/3"): each key fades its #kbd/<key>/pressed layer.
    presses: HashMap<String, Pressable>,
}

impl Component for Keyboard {
    const KIND: ComponentKind = ComponentKind::Keyboard;

    /// One layout; the labels change per mode, not the variant.
    fn variant(&self) -> String {
        "state=default".into()
    }

    fn last_change(&self) -> Option<Change> {
        None
    }
}

/// What a key press means for the screen around the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The text or the mode changed.
    Edited,
    /// Done: the user is finished with the field.
    Done,
}

impl Keyboard {
    /// Applies a gesture on a key node: the key shows pressed while the finger is on it and
    /// is returned when it is released over it.
    pub fn on_key(&mut self, node: &str, gesture: Gesture) -> Option<Key> {
        let key = key_of_node(node)?;
        self.presses.entry(node.to_string()).or_default().on(gesture).then_some(key)
    }

    /// Applies a gesture on a key node and, on a tap, the key to `text` (the field in focus):
    /// what the app does with the result is its own (Done: submit). None while the key is
    /// held, for a node that is not a key, and for Shift / mode (they only change labels).
    pub fn on(&mut self, node: &str, gesture: Gesture, text: &mut String) -> Option<Outcome> {
        let key = self.on_key(node, gesture)?;
        Some(self.press(key, text)).filter(|_| !matches!(key, Key::Shift | Key::Mode))
    }

    /// The key nodes of the component, in the order of the layout.
    pub fn key_nodes() -> Vec<String> {
        let mut nodes = Vec::new();
        for (row, r) in [(Row::One, 1), (Row::Two, 2), (Row::Three, 3)] {
            nodes.extend((1..=row.len()).map(|i| format!("#kbd/r{r}/{i}")));
        }
        nodes.extend(["shift", "delete", "mode", "space", "done"].map(|k| format!("#kbd/{k}")));
        nodes
    }

    /// The label the code sets on a key node (the character keys, Shift and mode); None for
    /// keys whose label is fixed in the design (Delete, space, Done).
    pub fn label_of(&self, node: &str) -> Option<String> {
        match key_of_node(node)? {
            Key::Char(row, index) => Some(self.label(row, index)),
            Key::Shift => Some(self.shift_label().into()),
            Key::Mode => Some(self.mode_label().into()),
            _ => None,
        }
    }

    /// Opacity target of the pressed layer of a key node: 100 pressed, 0 not.
    pub fn pressed_target(&self, node: &str) -> f32 {
        self.presses.get(node).map_or(0.0, Pressable::pressed_target)
    }

    /// How the pressed layer of a key node fades.
    pub fn press_transition(&self, node: &str) -> Option<Transition> {
        let press = self.presses.get(node).copied().unwrap_or_default();
        press.press_transition(ComponentKind::Keyboard)
    }

    /// The character on a key in the current mode.
    pub fn char_at(&self, row: Row, index: usize) -> Option<char> {
        let rows = match self.mode {
            Mode::Lower | Mode::Upper => &LOWER,
            Mode::Symbols => &SYMBOLS,
            Mode::MoreSymbols => &MORE_SYMBOLS,
        };
        let c = rows[row as usize].chars().nth(index)?;
        Some(if self.mode == Mode::Upper { c.to_ascii_uppercase() } else { c })
    }

    /// The label of a character key.
    pub fn label(&self, row: Row, index: usize) -> String {
        self.char_at(row, index).map(String::from).unwrap_or_default()
    }

    /// The label of Shift: what pressing it switches to.
    pub fn shift_label(&self) -> &'static str {
        match self.mode {
            Mode::Lower | Mode::Upper => "Shift",
            Mode::Symbols => "#+=",
            Mode::MoreSymbols => "123",
        }
    }

    /// The label of the mode key: what pressing it switches to.
    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            Mode::Lower | Mode::Upper => "123",
            Mode::Symbols | Mode::MoreSymbols => "abc",
        }
    }

    /// Applies a key to `text`.
    pub fn press(&mut self, key: Key, text: &mut String) -> Outcome {
        match key {
            Key::Char(row, index) => {
                if let Some(c) = self.char_at(row, index) {
                    if text.chars().count() < MAX_LEN {
                        text.push(c);
                    }
                }
                // Shift holds for one letter, as on a phone.
                if self.mode == Mode::Upper {
                    self.mode = Mode::Lower;
                }
            }
            Key::Space => {
                if text.chars().count() < MAX_LEN {
                    text.push(' ');
                }
            }
            Key::Delete => {
                text.pop();
            }
            Key::Shift => {
                self.mode = match self.mode {
                    Mode::Lower => Mode::Upper,
                    Mode::Upper => Mode::Lower,
                    Mode::Symbols => Mode::MoreSymbols,
                    Mode::MoreSymbols => Mode::Symbols,
                }
            }
            Key::Mode => {
                self.mode = match self.mode {
                    Mode::Lower | Mode::Upper => Mode::Symbols,
                    Mode::Symbols | Mode::MoreSymbols => Mode::Lower,
                }
            }
            Key::Done => return Outcome::Done,
        }
        Outcome::Edited
    }
}

/// The key of a Figma node name: "#kbd/r1/3" is row one, index 2.
pub fn key_of_node(node: &str) -> Option<Key> {
    let name = node.strip_prefix("#kbd/")?;
    match name {
        "shift" => return Some(Key::Shift),
        "delete" => return Some(Key::Delete),
        "mode" => return Some(Key::Mode),
        "space" => return Some(Key::Space),
        "done" => return Some(Key::Done),
        _ => {}
    }
    let (row, index) = name.split_once('/')?;
    let row = match row {
        "r1" => Row::One,
        "r2" => Row::Two,
        "r3" => Row::Three,
        _ => return None,
    };
    let index: usize = index.parse().ok()?;
    (1..=row.len()).contains(&index).then_some(Key::Char(row, index - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(keys: &[Key]) -> (String, Keyboard) {
        let mut kbd = Keyboard::default();
        let mut text = String::new();
        for key in keys {
            kbd.press(*key, &mut text);
        }
        (text, kbd)
    }

    #[test]
    fn shift_holds_for_one_letter() {
        let (text, kbd) =
            typed(&[Key::Shift, Key::Char(Row::One, 0), Key::Char(Row::One, 0)]);
        assert_eq!(text, "Qq");
        assert_eq!(kbd.mode, Mode::Lower);
    }

    #[test]
    fn symbols_and_back() {
        let (text, kbd) = typed(&[
            Key::Mode,
            Key::Char(Row::One, 0),
            Key::Shift,
            Key::Char(Row::Two, 0),
            Key::Mode,
            Key::Char(Row::Three, 6),
        ]);
        assert_eq!(text, "1\\m");
        assert_eq!(kbd.mode, Mode::Lower);
    }

    #[test]
    fn delete_space_limit_done() {
        let mut kbd = Keyboard::default();
        let mut text = String::from("ab");
        kbd.press(Key::Delete, &mut text);
        kbd.press(Key::Space, &mut text);
        assert_eq!(text, "a ");
        let mut long = "x".repeat(MAX_LEN);
        kbd.press(Key::Char(Row::One, 0), &mut long);
        assert_eq!(long.len(), MAX_LEN);
        assert_eq!(kbd.press(Key::Done, &mut text), Outcome::Done);
    }

    #[test]
    fn labels_follow_the_mode() {
        let mut kbd = Keyboard::default();
        assert_eq!((kbd.label(Row::Two, 0), kbd.shift_label(), kbd.mode_label()), ("a".into(), "Shift", "123"));
        kbd.mode = Mode::Symbols;
        assert_eq!((kbd.label(Row::Two, 0), kbd.shift_label(), kbd.mode_label()), ("-".into(), "#+=", "abc"));
    }

    #[test]
    fn node_names() {
        assert_eq!(key_of_node("#kbd/r1/3"), Some(Key::Char(Row::One, 2)));
        assert_eq!(key_of_node("#kbd/r3/7"), Some(Key::Char(Row::Three, 6)));
        assert_eq!(key_of_node("#kbd/r3/8"), None);
        assert_eq!(key_of_node("#kbd/delete"), Some(Key::Delete));
        assert_eq!(key_of_node("#wifi/connect"), None);
    }

    #[test]
    fn a_key_shows_pressed_and_acts_on_release() {
        let mut kbd = Keyboard::default();
        assert_eq!(kbd.on_key("#kbd/r1/1", Gesture::DOWN), None);
        assert_eq!(kbd.pressed_target("#kbd/r1/1"), 100.0);
        assert_eq!(kbd.pressed_target("#kbd/r1/2"), 0.0);
        assert_eq!(kbd.press_transition("#kbd/r1/1").unwrap().duration.as_millis(), 80);
        assert_eq!(kbd.on_key("#kbd/r1/1", Gesture::UP), Some(Key::Char(Row::One, 0)));
        assert_eq!(kbd.pressed_target("#kbd/r1/1"), 0.0);
        assert_eq!(kbd.on_key("#wifi/connect", Gesture::DOWN), None, "not a key");
    }

    #[test]
    fn on_types_into_the_text_and_reports_done() {
        let mut kbd = Keyboard::default();
        let mut text = String::new();
        assert_eq!(kbd.on("#kbd/r1/1", Gesture::DOWN, &mut text), None, "held");
        assert_eq!(kbd.on("#kbd/r1/1", Gesture::UP, &mut text), Some(Outcome::Edited));
        kbd.on("#kbd/shift", Gesture::DOWN, &mut text);
        assert_eq!(kbd.on("#kbd/shift", Gesture::UP, &mut text), None, "Shift only changes labels");
        assert_eq!(kbd.label_of("#kbd/r1/2").as_deref(), Some("W"));
        kbd.on("#kbd/done", Gesture::DOWN, &mut text);
        assert_eq!((kbd.on("#kbd/done", Gesture::UP, &mut text), text.as_str()), (Some(Outcome::Done), "q"));
        assert_eq!(Keyboard::key_nodes().len(), 31);
        assert_eq!(kbd.label_of("#kbd/done"), None, "fixed in the design");
    }
}

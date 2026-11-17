//! List item (#list-item): a row of a list with the roles of Material 3's list item: a
//! leading slot (an icon `#list-item/leading`, or an image `#list-item/image`), the headline
//! (`#list-item/title`), the supporting text (`#list-item/subtitle`) and the trailing end (an
//! icon `#list-item/trailing`, a toggle, a chevron, with an image a trailing text
//! `#list-item/trailing-text`, or with kind text the trailing text alone: a value on the right
//! of its label). Kind leading text has a short text of two lines at the start
//! (`#list-item/leading-title` over `#list-item/leading-subtitle`) and a thin bar
//! (`#list-item/accent`) in a colour the code sets; kind subheader is the label of a group of
//! rows with a trailing text, on the list's background. `lines` sets the height: one (headline), two (one line of
//! supporting text), three (a tall row, headline and supporting text up to two lines each). An icon slot holds every glyph of the library's `#icon` as an
//! instance `#list-item/<slot>/<glyph>` at opacity 0; the code lights one. (harry does not
//! swap the variant of an instance inside a list item: overrides.rs reads the global
//! customizations only; opacity is per item and display-only.)

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::transitions::{Change, ComponentKind};
use crate::Component;

/// The glyphs of `#icon`, as components.json lists them (checked by a test): the instances
/// of an icon slot are named `#list-item/<slot>/<glyph>`.
pub const GLYPHS: [&str; 11] = [
    "wifi", "wifi-high", "wifi-low", "wifi-zero", "lock", "chevron-right", "minus", "plus", "settings-2", "sliders-horizontal", "maximize-2",
];

/// The kind property of #list-item: what is at the ends of the row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListItemKind {
    /// Text only.
    #[default]
    Plain,
    /// An icon before the text.
    Icon,
    /// A toggle at the end.
    Toggle,
    /// A chevron at the end: opens something.
    Chevron,
    /// An image before the text (a registered picture, everyware_ui::images) and a trailing
    /// text at the end.
    Image,
    /// A label and its value (the trailing supporting text of Material 3) at the end, in one
    /// line: "Firmware · 0.1.0".
    Text,
    /// A short text of two lines before the headline (a strong line over a dim one: "8:30" over
    /// "30 min") and a thin bar between them in a colour the code sets (`accent`); lines two to
    /// seven: the headline takes all lines but the last, which is the supporting text's, so a
    /// long headline is shown whole (the app counts its lines).
    LeadingText,
    /// The label of the group of rows below it, with a trailing text ("Section · 3 items"):
    /// lower than a row, no card, lines one (Material's list subheader). In a list of taller
    /// rows it takes its own pitch (`pitch`). It takes no tap: a finger on it only drags the
    /// list.
    Subheader,
}

/// The lines property of #list-item: the height of the row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListItemLines {
    /// The headline only (72, the touch target).
    One,
    /// Headline and one line of supporting text (72).
    #[default]
    Two,
    /// Headline and supporting text up to two lines each (128) with an image; with a leading
    /// text, a headline of two lines and a line of supporting text (94).
    Three,
    /// With a leading text only: a headline of three lines and a line of supporting text (116).
    Four,
    /// With a leading text only: a headline of four lines (138).
    Five,
    /// With a leading text only: a headline of five lines (160).
    Six,
    /// With a leading text only: a headline of six lines (182).
    Seven,
}

impl ListItemLines {
    /// The lines of a row whose headline takes `headline` lines and its supporting text one
    /// (kind leading text): two to seven; a longer headline ends in an ellipsis after six.
    pub fn with_headline(headline: usize) -> Self {
        match headline {
            0 | 1 => ListItemLines::Two,
            2 => ListItemLines::Three,
            3 => ListItemLines::Four,
            4 => ListItemLines::Five,
            5 => ListItemLines::Six,
            _ => ListItemLines::Seven,
        }
    }

    /// How many lines: 1 to 7.
    pub fn count(self) -> usize {
        match self {
            ListItemLines::One => 1,
            ListItemLines::Two => 2,
            ListItemLines::Three => 3,
            ListItemLines::Four => 4,
            ListItemLines::Five => 5,
            ListItemLines::Six => 6,
            ListItemLines::Seven => 7,
        }
    }
}

/// A row of kind leading text: 72 for a headline of one line, 22 more for each further line
/// (the design's, components-patch.js; everyware_ui_test checks it against the Library).
const LEADING_TEXT_H: f32 = 72.0;
const HEADLINE_LINE_H: f32 = 22.0;

/// The size property of #list-item: the width of the row (a variant has a fixed size).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListItemSize {
    /// 440 wide: a column of a card.
    #[default]
    Regular,
    /// 490 wide: a column of a dashboard.
    Column,
    /// 856 wide: the content of a left or right sheet (880) beside a scrollbar.
    Panel,
    /// 960 wide: a dialog sheet's list.
    Wide,
}

/// A row of a list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListItem {
    /// What is at the ends of the row.
    pub kind: ListItemKind,
    /// First line.
    pub title: String,
    /// Second line.
    pub subtitle: String,
    /// Marked as the current one.
    pub selected: bool,
    /// Pressed and disabled.
    pub press: Pressable,
    /// The glyph of `#icon` in the leading slot (kind icon), by its `icon` value.
    pub leading: Option<&'static str>,
    /// The glyph of `#icon` in the trailing slot (kinds plain and icon); None hides it.
    pub trailing: Option<&'static str>,
    /// The width.
    pub size: ListItemSize,
    /// The height.
    pub lines: ListItemLines,
    /// The picture of kind image: the key of a registered image; None keeps the placeholder.
    pub image: Option<String>,
    /// The trailing text of kinds image (a source, an age), text (the value) and subheader.
    pub trailing_text: String,
    /// The strong line before the headline (kind leading text).
    pub leading_title: String,
    /// The dim line under it.
    pub leading_subtitle: String,
    /// The colour of the bar of kind leading text, 0xRRGGBB; None keeps the design's accent.
    pub accent: Option<u32>,
    /// What the row takes of its list (its height plus the gap) when that is not the list's
    /// pitch: a subheader between taller rows. None: the list's pitch.
    pub pitch: Option<f32>,
    /// What identifies the row across a refresh of its list (`ListView::set_rows`) when its
    /// headline does not: the id of the item it shows. None: the headline.
    pub id: Option<String>,
    /// The row takes no tap (a line that only tells something, "Nothing found"): never pressed or
    /// chosen, a finger on it drags the list, as on a subheader.
    pub passive: bool,
}

/// What a list item reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListItemEvent {
    /// The row was tapped.
    Clicked,
}

impl ListItem {
    /// A new, enabled row. A subheader has one line, every other kind two until `lines` says
    /// otherwise.
    pub fn new(kind: ListItemKind, title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        let lines = if kind == ListItemKind::Subheader { ListItemLines::One } else { ListItemLines::Two };
        ListItem { kind, title: title.into(), subtitle: subtitle.into(), lines, ..Default::default() }
    }

    /// The leading icon (kind icon): a glyph of `#icon`.
    pub fn leading(mut self, icon: &'static str) -> Self {
        self.leading = Some(icon);
        self
    }

    /// The trailing icon: a glyph of `#icon`, or none.
    pub fn trailing(mut self, icon: Option<&'static str>) -> Self {
        self.trailing = icon;
        self
    }

    /// The wide row (960).
    pub fn wide(mut self) -> Self {
        self.size = ListItemSize::Wide;
        self
    }

    /// The width.
    pub fn size(mut self, size: ListItemSize) -> Self {
        self.size = size;
        self
    }

    /// The height.
    pub fn lines(mut self, lines: ListItemLines) -> Self {
        self.lines = lines;
        self
    }

    /// The picture (kind image): the key of a registered image.
    pub fn image(mut self, key: impl Into<String>) -> Self {
        self.image = Some(key.into());
        self
    }

    /// How high the row is drawn (the design's variant): a list's pitch is that plus the gap.
    pub fn height(&self) -> f32 {
        match (self.kind, self.lines) {
            (ListItemKind::Subheader, _) => 48.0,
            (ListItemKind::LeadingText, lines) => LEADING_TEXT_H + HEADLINE_LINE_H * lines.count().saturating_sub(2) as f32,
            (_, ListItemLines::Three) => 128.0,
            _ => 72.0,
        }
    }

    /// Whether a finger on the row taps it; a subheader and a passive row only let the list drag.
    pub fn takes_taps(&self) -> bool {
        self.kind != ListItemKind::Subheader && !self.passive
    }

    /// What identifies the row across a refresh of its list when its headline does not (two
    /// events of the same title): the id of the item it shows.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// The row takes no tap: it tells something and opens nothing ("Nothing found"). A finger on
    /// it drags the list and shows no pressed layer.
    pub fn passive(mut self) -> Self {
        self.passive = true;
        self
    }

    /// The trailing text (kinds image, text and subheader).
    pub fn trailing_text(mut self, text: impl Into<String>) -> Self {
        self.trailing_text = text.into();
        self
    }

    /// The two lines before the headline (kind leading text): the strong one over the dim one.
    pub fn leading_text(mut self, title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        self.leading_title = title.into();
        self.leading_subtitle = subtitle.into();
        self
    }

    /// The colour of the bar (kind leading text), 0xRRGGBB. Each colour becomes a picture
    /// that stays for the life of the process (harry has no colour of its own to set), so the
    /// colours come from a palette, not from free data.
    pub fn accent(mut self, rgb: u32) -> Self {
        self.accent = Some(rgb & 0xff_ffff);
        self
    }

    /// What the row takes of its list, its height plus the gap, when that is not the list's
    /// pitch (`ListView::new`): a subheader between taller rows.
    pub fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = Some(pitch);
        self
    }

    /// Applies a gesture on the row.
    pub fn on(&mut self, gesture: Gesture) -> Option<ListItemEvent> {
        self.press.on(gesture).then_some(ListItemEvent::Clicked)
    }

    /// Marks or unmarks the row as the selected one.
    pub fn set_selected(&mut self, selected: bool) {
        if self.selected != selected {
            self.selected = selected;
            self.press.last_change = Some(Change::Select);
        }
    }
}

impl Component for ListItem {
    const KIND: ComponentKind = ComponentKind::ListItem;

    fn variant(&self) -> String {
        // pressed is a layer (#list-item/pressed, Pressable::pressed_target)
        let state = match (self.press.enabled, self.selected) {
            (false, _) => "disabled",
            (true, true) => "selected",
            (true, false) => "default",
        };
        let kind = match self.kind {
            ListItemKind::Plain => "plain",
            ListItemKind::Icon => "icon",
            ListItemKind::Toggle => "toggle",
            ListItemKind::Chevron => "chevron",
            ListItemKind::Image => "image",
            ListItemKind::Text => "text",
            ListItemKind::LeadingText => "leading-text",
            ListItemKind::Subheader => "subheader",
        };
        let lines = match self.lines {
            ListItemLines::One => "one",
            ListItemLines::Two => "two",
            ListItemLines::Three => "three",
            ListItemLines::Four => "four",
            ListItemLines::Five => "five",
            ListItemLines::Six => "six",
            ListItemLines::Seven => "seven",
        };
        let size = match self.size {
            ListItemSize::Regular => "regular",
            ListItemSize::Column => "column",
            ListItemSize::Panel => "panel",
            ListItemSize::Wide => "wide",
        };
        format!("state={state}, kind={kind}, lines={lines}, size={size}")
    }

    fn last_change(&self) -> Option<Change> {
        self.press.last_change
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_glyphs_are_the_icon_set_of_the_spec() {
        let spec: serde_json::Value = serde_json::from_str(include_str!("../../components.json")).unwrap();
        let icons: Vec<&str> = spec["components"]["icon"]["props"]["icon"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(icons, GLYPHS);
    }

    #[test]
    fn a_tall_row_with_an_image() {
        let item = ListItem::new(ListItemKind::Image, "A headline", "Supporting text").lines(ListItemLines::Three).size(ListItemSize::Column).image("pic/1").trailing_text("2 h");
        assert_eq!(item.variant(), "state=default, kind=image, lines=three, size=column");
        assert_eq!(item.image.as_deref(), Some("pic/1"));
    }

    #[test]
    fn a_label_and_its_value() {
        let item = ListItem::new(ListItemKind::Text, "Firmware", "").lines(ListItemLines::One).size(ListItemSize::Column).trailing_text("0.1.0");
        assert_eq!(item.variant(), "state=default, kind=text, lines=one, size=column");
        assert_eq!(item.trailing_text, "0.1.0");
    }

    #[test]
    fn a_row_with_a_leading_text_and_an_accent() {
        let item = ListItem::new(ListItemKind::LeadingText, "Stand-up", "Room 2").leading_text("8:30", "30 min").accent(0xff33_b679).size(ListItemSize::Column);
        assert_eq!(item.variant(), "state=default, kind=leading-text, lines=two, size=column");
        assert_eq!((item.leading_title.as_str(), item.leading_subtitle.as_str(), item.accent), ("8:30", "30 min", Some(0x33b679)));
        assert_eq!(ListItem::new(ListItemKind::LeadingText, "A", "").accent, None, "the design's accent");
    }

    #[test]
    fn a_long_headline_takes_more_lines() {
        let row = |n| ListItem::new(ListItemKind::LeadingText, "A long headline", "Place").lines(ListItemLines::with_headline(n)).size(ListItemSize::Column);
        assert_eq!(row(1).variant(), "state=default, kind=leading-text, lines=two, size=column");
        assert_eq!(row(3).variant(), "state=default, kind=leading-text, lines=four, size=column");
        assert_eq!(row(6).variant(), "state=default, kind=leading-text, lines=seven, size=column");
        assert_eq!(row(9).lines, ListItemLines::Seven, "at most six lines of headline");
        assert_eq!((row(0).height(), row(1).height(), row(2).height(), row(6).height()), (72.0, 72.0, 94.0, 182.0));
        assert_eq!(ListItemLines::with_headline(4).count(), 5);
        assert_eq!(ListItem::new(ListItemKind::Image, "A", "").lines(ListItemLines::Three).height(), 128.0);
        assert_eq!(ListItem::new(ListItemKind::Plain, "A", "").height(), 72.0);
    }

    #[test]
    fn a_subheader_takes_no_tap() {
        assert!(!ListItem::new(ListItemKind::Subheader, "Section", "").takes_taps());
        assert!(ListItem::new(ListItemKind::LeadingText, "A", "").takes_taps());
        // a passive row of any kind neither
        let note = ListItem::new(ListItemKind::Plain, "Nothing found", "").passive();
        assert!(!note.takes_taps());
        assert_eq!(note.variant(), "state=default, kind=plain, lines=two, size=regular", "it looks as any row");
        assert_eq!(ListItem::new(ListItemKind::Subheader, "Section", "").height(), 48.0);
    }

    #[test]
    fn a_subheader_has_one_line_and_its_own_pitch() {
        let item = ListItem::new(ListItemKind::Subheader, "Section", "").trailing_text("3 items").pitch(56.0);
        assert_eq!(item.variant(), "state=default, kind=subheader, lines=one, size=regular");
        assert_eq!((item.trailing_text.as_str(), item.pitch), ("3 items", Some(56.0)));
        assert_eq!(ListItem::new(ListItemKind::Plain, "A", "").pitch, None);
    }

    #[test]
    fn a_row_of_a_side_sheet() {
        let item = ListItem::new(ListItemKind::Plain, "Amsterdam", "GMT+01:00").lines(ListItemLines::Two).size(ListItemSize::Panel);
        assert_eq!(item.variant(), "state=default, kind=plain, lines=two, size=panel");
    }

    #[test]
    fn selected_after_release() {
        let mut item = ListItem::new(ListItemKind::Chevron, "Bedroom", "21°");
        item.on(Gesture::DOWN);
        assert_eq!((item.variant().as_str(), item.press.pressed_target()), ("state=default, kind=chevron, lines=two, size=regular", 100.0));
        assert_eq!(item.on(Gesture::UP), Some(ListItemEvent::Clicked));
        item.set_selected(true);
        assert_eq!(item.variant(), "state=selected, kind=chevron, lines=two, size=regular");
        assert_eq!(item.transition().unwrap().duration.as_millis(), 160);
    }
}

//! #list-item in an app: rows of the app's data in a list that scrolls with the finger and takes
//! a tap: stories with their pictures, or rows in groups under a subheader each. The docs show the parts between the doc marks
//! (docs/build-docs.js); the tests at the end compile and run them (everyware_ui_recipes_test).
//! Edit here: this file is the example of the guide docs/content/list_item.md.

// doc:note Rows never stand alone: a `ListView` holds them, scrolls them and reports the tapped row; its slot is `interactive`, so a drag may start between the rows.

use crate::prelude::*;
// doc:use
use everyware_ui::images;
use everyware_ui::list_item::{ListItem, ListItemKind, ListItemLines, ListItemSize};
use everyware_ui::list_view::{ListEvent, ListView};
// doc:end

/// The list's slot is 744 high; a row of three lines is 128 high, 8 between two rows.
const LIST_H: f32 = 744.0;
const ROW_PITCH: f32 = 128.0 + 8.0;

/// A story as the app has it. Yours comes from a service.
#[derive(Clone, Debug)]
pub struct Story {
    pub id: u64,
    pub title: String,
    pub summary: String,
    pub age: String,
}

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    /// The rows and where they are scrolled to: one `ListItem` per item of the app's data.
    pub my_list_item: ListView,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            // empty until the first data comes; the slot's height and the row pitch
            my_list_item: ListView::new([], LIST_H, ROW_PITCH),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_list_item: Slot = "#doc/list_item" interactive ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_list_item = list("#doc/list_item", &state.my_list_item);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        // a row ("#doc/list_item/<row>") or the list between its rows (None): a drag scrolls,
        // a fast lift flings, a tap chooses the row
        _ if node == "#doc/list_item" || node.starts_with("#doc/list_item/") => {
            let row = row_of("#doc/list_item", node);
            if let Some(ListEvent::Chosen(row)) = state.my_list_item.on(row, gesture, Instant::now()) {
                state.my_list_item.select(Some(row));
                log::info!("row {row} chosen"); // yours: open it
            }
        }
        // doc:end
        _ => {}
    }
}

// doc:change
/// New stories from the service: one row each, the headline, the summary, the age at the end and
/// the picture by its key. A row keeps its pressed and selected state by its key (the headline),
/// so a tap in progress still lands; the scroll stays inside the new content.
pub fn show_stories(state: &mut State, stories: &[Story]) {
    state.my_list_item.set_rows(stories.iter().map(|story| {
        ListItem::new(ListItemKind::Image, story.title.clone(), story.summary.clone())
            .lines(ListItemLines::Three)
            .size(ListItemSize::Column)
            .image(format!("news/{}", story.id))
            .trailing_text(story.age.clone())
    }));
}

/// A story's picture came: registered under its row's key, it shows at the next frame. Until
/// then the row shows the design's placeholder.
pub fn show_picture(id: u64, rgb: &[u8], width: u32, height: u32) {
    if let Err(err) = images::register_rgb(format!("news/{id}"), rgb, width, height) {
        log::warn!("picture of story {id}: {err}");
    }
}

/// A subheader is 48 high, a row of two lines 72; 8 between them.
const SUBHEADER_PITCH: f32 = 48.0 + 8.0;
const TWO_LINES_PITCH: f32 = 72.0 + 8.0;

/// Rows in groups: a subheader over each group (its label and a trailing text), then rows that
/// lead with a short text of two lines and a bar in the group's colour (from a palette: each
/// colour stays a picture). Each row takes its own pitch, so the rows after a subheader move up.
pub fn show_groups(state: &mut State, groups: &[Group]) {
    state.my_list_item.set_rows(groups.iter().flat_map(|group| {
        let head = ListItem::new(ListItemKind::Subheader, group.label.clone(), "")
            .trailing_text(group.count.clone())
            .size(ListItemSize::Column)
            .pitch(SUBHEADER_PITCH);
        let rows = group.stories.iter().map(|story| {
            ListItem::new(ListItemKind::LeadingText, story.title.clone(), story.summary.clone())
                .leading_text(story.age.clone(), "ago")
                .accent(group.color)
                .size(ListItemSize::Column)
                .pitch(TWO_LINES_PITCH)
        });
        std::iter::once(head).chain(rows)
    }));
}
// doc:end

/// Stories under a label ("TODAY"), with their count and a colour for the bars.
#[derive(Clone, Debug)]
pub struct Group {
    pub label: String,
    pub count: String,
    pub color: u32,
    pub stories: Vec<Story>,
}

#[cfg(test)]
fn stories(n: u64) -> Vec<Story> {
    (0..n).map(|id| Story { id, title: format!("Story {id}"), summary: "A summary".into(), age: "2 h".into() }).collect()
}

/// A finger down and up at `y` in the list, on row `row` (harry's hit test names the row).
#[cfg(test)]
fn tap_row(state: &mut State, row: usize, y: f32) {
    let node = format!("#doc/list_item/{row}");
    on_gesture(state, &node, Gesture::down(0.0, y));
    on_gesture(state, &node, Gesture::up(0.0, y + 2.0));
}

#[test]
fn list_item_shows_the_stories_and_a_tap_selects_one() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    show_stories(&mut state, &stories(12));
    present(&mut model, &state);
    let rows = state.my_list_item.rows();
    assert_eq!(rows.len(), 12);
    assert_eq!(rows[0].variant(), "state=default, kind=image, lines=three, size=column");
    assert_eq!((rows[3].image.as_deref(), rows[3].trailing_text.as_str()), (Some("news/3"), "2 h"));
    // a tap on the third row selects it
    tap_row(&mut state, 2, 300.0);
    assert_eq!(state.my_list_item.selected(), Some(2));
    assert_eq!(state.my_list_item.rows()[2].variant(), "state=selected, kind=image, lines=three, size=column");
    present(&mut model, &state);
}

#[test]
fn list_item_a_drag_scrolls_and_chooses_nothing() {
    let mut state = State::new(&Sizes::none());
    show_stories(&mut state, &stories(12));
    on_gesture(&mut state, "#doc/list_item/1", Gesture::down(0.0, 400.0));
    // the finger moves up 200: past the touch slop, the content follows it
    on_gesture(&mut state, "#doc/list_item", Gesture::Move { x: 0.0, y: 200.0, inside: false });
    on_gesture(&mut state, "#doc/list_item", Gesture::Up { x: 0.0, y: 200.0, inside: false, velocity: 0.0, velocity_x: 0.0 });
    assert!(state.my_list_item.scroll_at(Instant::now()) > 150.0);
    assert_eq!(state.my_list_item.selected(), None);
}

#[test]
fn list_item_new_stories_keep_the_selected_row() {
    let mut state = State::new(&Sizes::none());
    show_stories(&mut state, &stories(4));
    tap_row(&mut state, 1, 150.0);
    // the feed comes again with a new story on top: "Story 1" is now the third row
    let mut next = vec![Story { id: 9, title: "Story 9".into(), summary: String::new(), age: "1 min".into() }];
    next.extend(stories(4));
    show_stories(&mut state, &next);
    assert_eq!(state.my_list_item.selected(), Some(2));
    assert_eq!(state.my_list_item.rows()[2].title, "Story 1");
}

#[test]
fn list_item_groups_have_a_subheader_each_and_rows_with_a_leading_text() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    let group = |label: &str, color: u32, n: u64| Group { label: label.into(), count: format!("{n} STORIES"), color, stories: stories(n) };
    show_groups(&mut state, &[group("TODAY", 0x0b8043, 2), group("YESTERDAY", 0x039be5, 3)]);
    present(&mut model, &state);
    let list = &state.my_list_item;
    let rows = list.rows();
    assert_eq!(rows.len(), 7);
    assert_eq!(rows[0].variant(), "state=default, kind=subheader, lines=one, size=column");
    assert_eq!(rows[1].variant(), "state=default, kind=leading-text, lines=two, size=column");
    assert_eq!((rows[1].leading_title.as_str(), rows[1].accent), ("2 h", Some(0x0b8043)));
    // the second subheader comes after one subheader and two rows
    assert_eq!(list.offset(3), SUBHEADER_PITCH + 2.0 * TWO_LINES_PITCH);
    assert_eq!(list.content_h(), 2.0 * SUBHEADER_PITCH + 5.0 * TWO_LINES_PITCH);
}

#[test]
fn list_item_a_picture_is_registered_under_its_row_s_key() {
    show_picture(4242, &[255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255], 2, 2);
    assert!(images::get("news/4242").is_some());
    images::unregister("news/4242");
}

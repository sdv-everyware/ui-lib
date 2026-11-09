// The lines an app writes to put a component on a screen, per component: what the docs show
// (docs/ai/<component>.md "The app", the site's right side) and what the host test
// everyware_ui_recipes_test compiles (docs/recipes/recipes.rs, written by build-docs.js). One
// table for both, so a recipe that does not compile fails the build, not an agent.
//
// In the strings SLOT is the slot's #name and F the field's name; LIST (scrollbar) is the field
// of the list it follows. `act: null` is a display-only component: no gestures.
//   use:     the imports (paths under everyware_ui)
//   state:   the field of State; init: its first value in State::new
//   model:   the field of the ui_model! (one per node)
//   present: the presenter's line(s)
//   act:     the arm(s) of the match on the gesture's node name (in fn on_gesture(state, node, gesture))
//   note:    one sentence the docs add under the code

const R = {
  button: {
    use: ['button::{Button, ButtonEvent, ButtonKind}'],
    state: 'pub F: Button,', init: 'F: Button::new(ButtonKind::Primary, "Talk").compact(),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: ['"SLOT" => {', '    if let Some(ButtonEvent::Clicked) = state.F.on(gesture) {', '        // yours', '    }', '}'],
  },
  icon_button: {
    use: ['button::{ButtonEvent, Icon, IconButton, IconButtonKind}'],
    state: 'pub F: IconButton,', init: 'F: IconButton::new(IconButtonKind::Tonal, Icon::Plus),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: ['"SLOT" => {', '    if let Some(ButtonEvent::Clicked) = state.F.on(gesture) {', '        // yours', '    }', '}'],
  },
  toggle: {
    use: ['toggle::{Toggle, ToggleEvent}'],
    state: 'pub F: Toggle,', init: 'F: Toggle::new(false),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: ['"SLOT" => {', '    if let Some(ToggleEvent::Toggled(on)) = state.F.on(gesture) {', '        let _ = on; // yours: state.F.value is the same', '    }', '}'],
    note: 'Read the value any time as `state.F.value` (a pub field); `state.F.set(on)` follows the device without a gesture.',
  },
  radio: {
    use: ['radio::{RadioEvent, RadioGroup}'],
    state: 'pub F: RadioGroup,', init: 'F: RadioGroup::new(["Off", "Low", "High"]),',
    model: ['F_1: Slot = "SLOT/1" ["SlotCustomizer"];', 'F_2: Slot = "SLOT/2" ["SlotCustomizer"];', 'F_3: Slot = "SLOT/3" ["SlotCustomizer"];'],
    present: ['model.F_1 = slot("SLOT/1", &state.F.options[0]);', 'model.F_2 = slot("SLOT/2", &state.F.options[1]);', 'model.F_3 = slot("SLOT/3", &state.F.options[2]);'],
    act: ['_ if node.starts_with("SLOT/") => {', '    // the option from its slot\'s name: SLOT/1 is index 0', '    if let Some(i) = row_of("SLOT", node).and_then(|n| n.checked_sub(1)) {', '        if let Some(RadioEvent::Selected(i)) = state.F.on(i, gesture) {', '            let _ = i; // yours', '        }', '    }', '}'],
    note: 'One slot per option, `SLOT/1`, `SLOT/2` ...; the group decides which is on.',
  },
  dropdown: {
    use: ['dropdown::{Dropdown, DropdownEvent}'],
    state: 'pub F: Dropdown,', init: 'F: Dropdown::new(["At home", "Away", "Asleep"]),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: ['"SLOT" => state.F.on_field(gesture),', '// the open list\'s items report by their own names, #dropdown/item/1 ...', '_ if node.starts_with("#dropdown/item/") => {', '    let n = node["#dropdown/item/".len()..].parse::<usize>().ok().and_then(|n| n.checked_sub(1));', '    if let Some(DropdownEvent::Selected(i)) = n.and_then(|n| state.F.on_item(n, gesture)) {', '        let _ = i; // yours', '    }', '}'],
    note: 'The field opens and closes the list (`on_field`); an item reports by its own name, one open list at a time.',
  },
  list_item: {
    use: ['list_item::{ListItem, ListItemKind}', 'list_view::{ListEvent, ListView}'],
    state: 'pub F: ListView,', init: 'F: ListView::new([ListItem::new(ListItemKind::Plain, "Title", "Supporting text")], 744.0, 72.0 + 8.0),',
    model: 'F: Slot = "SLOT" interactive ["SlotCustomizer"];',
    present: 'model.F = list("SLOT", &state.F);',
    act: ['// the list itself (a drag between rows) or a row, "SLOT/<row>"', '_ if node == "SLOT" || node.starts_with("SLOT/") => {', '    if let Some(ListEvent::Chosen(row)) = state.F.on(row_of("SLOT", node), gesture, Instant::now()) {', '        let _ = row; // yours', '    }', '}'],
    note: 'A list is a `ListView` of rows (`ListItem`), its slot `interactive`; new rows with `state.F.set_rows(...)` keep a tap in progress.',
  },
  scrollbar: {
    use: ['list_view::ListView'],
    state: null, init: null,
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = scrollbar("SLOT", &state.LIST);',
    act: null,
    note: 'Display only: it follows the `ListView` of its list (`LIST`) and fades out when the list stops.',
  },
  segmented: {
    use: ['segmented::{Segmented, SegmentedEvent}'],
    state: 'pub F: Segmented,', init: 'F: Segmented::new(["Low", "Med", "High"]),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: ['// the segments report by their own names, #segmented/segment-1 ...', '_ if node.starts_with("#segmented/segment-") => {', '    let n = node["#segmented/segment-".len()..].parse::<usize>().ok().and_then(|n| n.checked_sub(1));', '    if let Some(SegmentedEvent::Selected(i)) = n.and_then(|n| state.F.on(n, gesture)) {', '        let _ = i; // yours: state.F.value() is the label', '    }', '}'],
  },
  slider: {
    use: ['slider::{Slider, SliderEvent}', 'design::Sizes'],
    state: 'pub F: Slider,', init: 'F: Slider::in_slot(65.0, frame.x, frame.width),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: ['"SLOT" => {', '    if let Some(SliderEvent::Changed(value)) = state.F.on(gesture) {', '        let _ = value; // yours: the finger is up; Moving(value) while it drags', '    }', '}'],
    note: '`frame` is where the design puts the slot: `let frame = sizes.frame("SLOT")` with `sizes = Sizes::load(<the .dcf>)` at the start; the touch area follows the design.',
  },
  symbol: {
    use: ['symbol::{Symbol, SymbolIcon}'],
    state: 'pub F: Symbol,', init: 'F: Symbol::new(SymbolIcon::House),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: null,
  },
  keyboard: {
    use: ['keyboard::Keyboard'],
    state: 'pub F: Keyboard,', init: 'F: Keyboard::default(),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: ['// the keys report by their own names, #kbd/<row>/<i>', '_ if node.starts_with("#kbd/") => {', '    let mut text = String::new(); // yours: the text of the field being edited', '    if let Some(outcome) = state.F.on(node, gesture, &mut text) {', '        let _ = outcome;', '    }', '}'],
    note: 'Usually inside the keyboard dock with a form (`Form`), which routes the keys to the focused field.',
  },
  bar_chart: {
    use: ['bar_chart::BarChart'],
    state: 'pub F: BarChart,', init: 'F: BarChart::new([(40.0, false); 24]),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: null,
    note: 'Display only; `BarChart::new(bars)` takes up to 24 (level 0..100, accent) pairs. The app scales its data to levels and decides which bars are accented.',
  },
  input_field: {
    use: ['input_field::{InputEvent, InputField}'],
    state: 'pub F: InputField,', init: 'F: InputField::new("Password", ""),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: ['"SLOT" => {', '    if let Some(InputEvent::Focused) = state.F.on(gesture, Instant::now()) {', '        // yours: open the keyboard (a Form does it for its fields)', '    }', '}'],
  },
  tile: {
    use: ['tile::{Tile, TileEvent, TileKind}', 'symbol::SymbolIcon'],
    state: 'pub F: Tile,', init: 'F: Tile::new(TileKind::Slider, SymbolIcon::BedDouble, "Bedroom", "Lights").value("65%"),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: ['"SLOT" => {', '    if let Some(TileEvent::Opened) = state.F.on(gesture) {', '        // yours: open the device\'s sheet', '    }', '}'],
    note: 'The whole card is one touch target; the control inside shows the device (`state.F.toggle`, `.slider`, `.segmented`, `.value`) and takes no taps: the device\'s sheet changes it.',
  },
  skeleton: {
    use: ['skeleton::{Skeleton, SkeletonKind, SkeletonSize}'],
    state: 'pub F: Skeleton,', init: 'F: Skeleton::new(SkeletonKind::Text, SkeletonSize::Regular),',
    model: 'F: Slot = "SLOT" ["SlotCustomizer"];',
    present: 'model.F = slot("SLOT", &state.F);',
    act: null,
    note: 'Display only; `state.F.start(Instant::now())` while the content loads.',
  },
};

const lines = v => v == null ? [] : Array.isArray(v) ? v : [v];
// a recipe with its slot name and field name filled in
function recipe(key, slot, field, list) {
  const r = R[key];
  if (!r) return null;
  const sub = s => s.replace(/SLOT/g, slot).replace(/\bF(?=_\d|\b)/g, field).replace(/\bLIST\b/g, list || 'list');
  return {
    use: r.use.map(u => `use everyware_ui::${u};`),
    state: lines(r.state).map(sub), init: lines(r.init).map(sub), model: lines(r.model).map(sub),
    present: lines(r.present).map(sub), act: r.act ? r.act.map(sub) : null, note: r.note ? sub(r.note) : null,
  };
}

module.exports = { R, recipe };

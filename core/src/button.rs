//! Button with text (#button) and button with an icon (#icon-button).

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::tone::Tone;
use crate::transitions::{Change, ComponentKind};
use crate::Component;

/// The kind property of #button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonKind {
    /// Filled with the accent colour: the main action.
    #[default]
    Primary,
    /// Soft fill.
    Secondary,
    /// Text only.
    Ghost,
}

/// The size property of #button and #icon-button: the size is the design's, as Google's components are drawn
/// (a fixed size per variant, the label centred in it), not the label's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// #button 200 wide (a sheet's Connect, Cancel); #icon-button 56.
    #[default]
    Regular,
    /// #button 116 wide (Talk in a widget's row); #icon-button 48.
    Compact,
}

/// A button with text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Button {
    /// Which look.
    pub kind: ButtonKind,
    /// Which width.
    pub size: ButtonSize,
    /// The text on it.
    pub label: String,
    /// Its colour role: the default accent, or success, warning, error (a deletion).
    pub tone: Tone,
    /// Pressed and disabled.
    pub press: Pressable,
}

/// What a button reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonEvent {
    /// The button was tapped.
    Clicked,
}

impl Button {
    /// A new, enabled button of the regular size.
    pub fn new(kind: ButtonKind, label: impl Into<String>) -> Self {
        Button { kind, size: ButtonSize::Regular, label: label.into(), tone: Tone::Default, press: Pressable::default() }
    }

    /// The same button in a tone (`Tone::Error` for an action that cannot be undone).
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// The same button in the compact size.
    pub fn compact(mut self) -> Self {
        self.size = ButtonSize::Compact;
        self
    }

    /// Applies a gesture on the button.
    pub fn on(&mut self, gesture: Gesture) -> Option<ButtonEvent> {
        self.press.on(gesture).then_some(ButtonEvent::Clicked)
    }
}

impl Component for Button {
    const KIND: ComponentKind = ComponentKind::Button;

    fn variant(&self) -> String {
        let kind = match self.kind {
            ButtonKind::Primary => "primary",
            ButtonKind::Secondary => "secondary",
            ButtonKind::Ghost => "ghost",
        };
        let size = match self.size {
            ButtonSize::Regular => "regular",
            ButtonSize::Compact => "compact",
        };
        // a tone is a kind of its own in the set (kind=primary-error), so the names of the
        // default kinds stay as they were
        let tone = match self.tone {
            Tone::Default => String::new(),
            t => format!("-{}", t.name()),
        };
        format!("kind={kind}{tone}, state={}, size={size}", self.press.enabled_state())
    }

    fn last_change(&self) -> Option<Change> {
        // pressed is a layer (Pressable::pressed_target), the variant only changes on disable
        None
    }
}

/// The kind property of #icon-button: the circle, as the dashboard draws it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonKind {
    /// Soft accent circle: the widgets' buttons.
    #[default]
    Tonal,
    /// White 44 % with a white line: the status row.
    Glass,
    /// White 15 %: over the camera.
    Scrim,
}

/// The icon property of #icon-button: the design's icons (lucide names).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Icon {
    /// `minus`
    #[default]
    Minus,
    /// `plus`
    Plus,
    /// `sliders-horizontal`
    Sliders,
    /// `maximize-2`
    Maximize,
    /// `chevron-right`
    ChevronRight,
    /// `settings-2`
    Settings,
    /// `wifi`
    Wifi,
    /// `x`: closes (a sheet)
    Close,
    /// `house`: home (a hub's rail)
    House,
    /// `door-open`: a door, the entrance
    DoorOpen,
    /// `bed-double`: a bedroom
    BedDouble,
    /// `armchair`: a living room
    Armchair,
    /// `droplets`: humidity, water
    Droplets,
    /// `chevron-left`: back, or a direction (a direction pad)
    ChevronLeft,
    /// `chevron-up`: up, or a direction
    ChevronUp,
    /// `chevron-down`: down, or a direction
    ChevronDown,
    /// `gamepad`: a game
    Gamepad,
    /// `pencil`: edits (a name)
    Pencil,
}

impl Icon {
    /// Every icon, in the order of the set's property (components.json).
    pub const ALL: &'static [Icon] = &[
        Icon::Minus,
        Icon::Plus,
        Icon::Sliders,
        Icon::Maximize,
        Icon::ChevronRight,
        Icon::Settings,
        Icon::Wifi,
        Icon::Close,
        Icon::House,
        Icon::DoorOpen,
        Icon::BedDouble,
        Icon::Armchair,
        Icon::Droplets,
        Icon::ChevronLeft,
        Icon::ChevronUp,
        Icon::ChevronDown,
        Icon::Gamepad,
        Icon::Pencil,
    ];

    /// The value of the set's `icon` property.
    pub fn name(self) -> &'static str {
        match self {
            Icon::Minus => "minus",
            Icon::Plus => "plus",
            Icon::Sliders => "sliders-horizontal",
            Icon::Maximize => "maximize-2",
            Icon::ChevronRight => "chevron-right",
            Icon::Settings => "settings-2",
            Icon::Wifi => "wifi",
            Icon::Close => "x",
            Icon::House => "house",
            Icon::DoorOpen => "door-open",
            Icon::BedDouble => "bed-double",
            Icon::Armchair => "armchair",
            Icon::Droplets => "droplets",
            Icon::ChevronLeft => "chevron-left",
            Icon::ChevronUp => "chevron-up",
            Icon::ChevronDown => "chevron-down",
            Icon::Gamepad => "gamepad",
            Icon::Pencil => "pencil",
        }
    }
}

/// A button with an icon. Its size is the design's (regular 56, compact 48).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IconButton {
    /// Which look.
    pub kind: IconButtonKind,
    /// Which icon.
    pub icon: Icon,
    /// Which size.
    pub size: ButtonSize,
    /// Pressed and disabled.
    pub press: Pressable,
}

impl IconButton {
    /// A new, enabled icon button of the regular size.
    pub fn new(kind: IconButtonKind, icon: Icon) -> Self {
        IconButton { kind, icon, size: ButtonSize::Regular, press: Pressable::default() }
    }

    /// The same button in the compact size.
    pub fn compact(mut self) -> Self {
        self.size = ButtonSize::Compact;
        self
    }

    /// Applies a gesture on the button.
    pub fn on(&mut self, gesture: Gesture) -> Option<ButtonEvent> {
        self.press.on(gesture).then_some(ButtonEvent::Clicked)
    }
}

impl Component for IconButton {
    const KIND: ComponentKind = ComponentKind::IconButton;

    fn variant(&self) -> String {
        let kind = match self.kind {
            IconButtonKind::Tonal => "tonal",
            IconButtonKind::Glass => "glass",
            IconButtonKind::Scrim => "scrim",
        };
        let size = match self.size {
            ButtonSize::Regular => "regular",
            ButtonSize::Compact => "compact",
        };
        format!("kind={kind}, state={}, icon={}, size={size}", self.press.enabled_state(), self.icon.name())
    }

    fn last_change(&self) -> Option<Change> {
        // pressed is a layer (Pressable::pressed_target), the variant only changes on disable
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_release_click() {
        let mut b = Button::new(ButtonKind::Primary, "Connect");
        assert_eq!(b.on(Gesture::DOWN), None);
        // pressed is a layer: the variant stays, the layer fades in
        assert_eq!((b.variant().as_str(), b.press.pressed_target()), ("kind=primary, state=default, size=regular", 100.0));
        assert_eq!(b.press.press_transition(Button::KIND).unwrap().duration.as_millis(), 80);
        assert_eq!(b.on(Gesture::UP), Some(ButtonEvent::Clicked));
        assert_eq!(b.press.pressed_target(), 0.0);
        assert_eq!(b.press.press_transition(Button::KIND).unwrap().duration.as_millis(), 160);
    }

    #[test]
    fn the_icons_are_the_icon_button_set_of_the_spec() {
        let spec: serde_json::Value = serde_json::from_str(include_str!("../../components.json")).unwrap();
        let icons: Vec<&str> = spec["components"]["icon_button"]["props"]["icon"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
        let ours: Vec<&str> = Icon::ALL.iter().map(|i| i.name()).collect();
        assert_eq!(icons, ours);
        let pad = IconButton::new(IconButtonKind::Tonal, Icon::ChevronUp);
        assert_eq!(pad.variant(), "kind=tonal, state=default, icon=chevron-up, size=regular");
        assert_eq!(IconButton::new(IconButtonKind::Glass, Icon::Gamepad).compact().variant(), "kind=glass, state=default, icon=gamepad, size=compact");
    }

    #[test]
    fn a_held_button_is_pressed_until_the_finger_lifts_or_leaves() {
        // a direction pad acts while its button is held: press.pressed from the down to the up
        let mut b = IconButton::new(IconButtonKind::Tonal, Icon::ChevronLeft);
        b.on(Gesture::DOWN);
        assert!(b.press.pressed);
        b.on(Gesture::Move { x: 1.0, y: 1.0, inside: true });
        assert!(b.press.pressed, "a finger that moves on the button still holds it");
        assert_eq!(b.on(Gesture::UP), Some(ButtonEvent::Clicked));
        assert!(!b.press.pressed);
        b.on(Gesture::DOWN);
        b.on(Gesture::Move { x: 90.0, y: 1.0, inside: false });
        assert!(!b.press.pressed, "a finger that slides off lets go");
    }

    #[test]
    fn cancel_and_disabled_do_not_click() {
        let mut b = Button::new(ButtonKind::Ghost, "Cancel");
        b.on(Gesture::DOWN);
        assert_eq!(b.on(Gesture::Cancel), None);
        b.press.set_enabled(false);
        assert_eq!(b.variant(), "kind=ghost, state=disabled, size=regular");
        assert_eq!(b.transition(), None);
        b.on(Gesture::DOWN);
        assert_eq!(b.on(Gesture::UP), None);
    }

    #[test]
    fn a_tone_is_a_kind_of_its_own_and_the_default_names_stay() {
        let b = Button::new(ButtonKind::Primary, "OK");
        assert_eq!((b.tone, b.variant().as_str()), (Tone::Default, "kind=primary, state=default, size=regular"));
        let mut red = Button::new(ButtonKind::Primary, "Remove").tone(Tone::Error);
        assert_eq!(red.variant(), "kind=primary-error, state=default, size=regular");
        red.press.set_enabled(false);
        assert_eq!(red.variant(), "kind=primary-error, state=disabled, size=regular");
        assert_eq!(Button::new(ButtonKind::Secondary, "Yes").tone(Tone::Success).compact().variant(), "kind=secondary-success, state=default, size=compact");
        assert_eq!(Button::new(ButtonKind::Ghost, "Maybe").tone(Tone::Warning).variant(), "kind=ghost-warning, state=default, size=regular");
        // a toned button taps as any other
        let mut b = Button::new(ButtonKind::Secondary, "Remove").tone(Tone::Error);
        b.on(Gesture::DOWN);
        assert_eq!(b.on(Gesture::UP), Some(ButtonEvent::Clicked));
    }
}

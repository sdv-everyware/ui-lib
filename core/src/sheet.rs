//! #sheet: the library's container for anything shown over the screen. A scrim and an empty
//! card (#sheet/content is for the app's design: the content is a component of its own in
//! that slot); the card is a dialog in the middle or a panel along one edge, and the app
//! picks which (`Layout`). Open and close move the card from behind its edge and fade the
//! scrim, both display-only in harry (position override and opacity: no new tree, no
//! layout). The app never sets the instance's variant: harry takes a variant from the
//! Library, which loses the instance's own part names and its swapped content; the layout is
//! the instance's in the design and `Layout` here must match it. `variant()` names that
//! variant for tests and the contract check.
//!
//! The geometry here is the geometry of components-patch.js (the same numbers in both):
//! a 1920x1080 screen, the dialog 1120 wide in the middle and as high as its content plus the
//! card's padding (the card hugs it; 840 until the content's height is known), a side panel
//! half the width, a top or bottom panel half the height, the closed card 40 px past its edge.

use crate::overlay::Overlay;
use crate::transitions::{Change, ComponentKind, Transition};
use crate::Component;

/// The screen's edge a panel sits at and slides in from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

/// How a sheet shows its card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// A card in the middle of the screen, in from the bottom.
    Dialog,
    /// A panel along `Edge`, half the screen, in from behind that edge.
    Panel(Edge),
}

/// A rectangle in screen pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// How far behind its edge the closed card waits (its shadow stays off the screen too).
pub const CLOSED_GAP: f32 = 40.0;
/// The dialog card: its width, its height until the content's is known, its padding around
/// the content (the card hugs the content).
pub const DIALOG_W: f32 = 1120.0;
pub const DIALOG_H: f32 = 840.0;
pub const CARD_PAD: f32 = 40.0;

impl Layout {
    /// The value of the Figma property `layout`.
    pub fn name(self) -> &'static str {
        match self {
            Layout::Dialog => "dialog",
            Layout::Panel(Edge::Left) => "left",
            Layout::Panel(Edge::Right) => "right",
            Layout::Panel(Edge::Top) => "top",
            Layout::Panel(Edge::Bottom) => "bottom",
        }
    }

    /// Every layout, in the order of the Figma property.
    pub const ALL: [Layout; 5] =
        [Layout::Dialog, Layout::Panel(Edge::Left), Layout::Panel(Edge::Right), Layout::Panel(Edge::Top), Layout::Panel(Edge::Bottom)];

    /// The card open, on a screen of `(w, h)`, with the content `content_height` high (a
    /// dialog's card hugs it; None: the default height).
    pub fn card(self, screen: (f32, f32), content_height: Option<f32>) -> Rect {
        let (sw, sh) = screen;
        match self {
            Layout::Dialog => {
                let h = content_height.map_or(DIALOG_H, |c| c + 2.0 * CARD_PAD);
                Rect { x: (sw - DIALOG_W) / 2.0, y: (sh - h) / 2.0, w: DIALOG_W, h }
            }
            Layout::Panel(Edge::Left) => Rect { x: 0.0, y: 0.0, w: sw / 2.0, h: sh },
            Layout::Panel(Edge::Right) => Rect { x: sw / 2.0, y: 0.0, w: sw / 2.0, h: sh },
            Layout::Panel(Edge::Top) => Rect { x: 0.0, y: 0.0, w: sw, h: sh / 2.0 },
            Layout::Panel(Edge::Bottom) => Rect { x: 0.0, y: sh / 2.0, w: sw, h: sh / 2.0 },
        }
    }

    /// Where the card waits closed: behind the edge it comes from, `CLOSED_GAP` past the screen.
    pub fn closed(self, screen: (f32, f32)) -> (f32, f32) {
        let (sw, sh) = screen;
        let open = self.card(screen, None);
        match self {
            Layout::Dialog | Layout::Panel(Edge::Bottom) => (open.x, sh + CLOSED_GAP),
            Layout::Panel(Edge::Top) => (open.x, -open.h - CLOSED_GAP),
            Layout::Panel(Edge::Left) => (-open.w - CLOSED_GAP, open.y),
            Layout::Panel(Edge::Right) => (sw + CLOSED_GAP, open.y),
        }
    }
}

/// A sheet: an overlay with the library's card, in one of the layouts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sheet {
    pub overlay: Overlay,
    pub layout: Layout,
    /// The height of the content in the card (design.rs Sizes): the card hugs it, so the
    /// middle of the screen depends on it. None until known: the default dialog height.
    pub content_height: Option<f32>,
}

impl Default for Sheet {
    fn default() -> Self {
        Sheet::new(Layout::Dialog)
    }
}

impl Sheet {
    /// A closed sheet with this layout.
    pub fn new(layout: Layout) -> Self {
        Sheet { overlay: Overlay::new(), layout, content_height: None }
    }

    /// The same sheet with this content height (the card's height follows the content).
    pub fn with_content_height(mut self, height: Option<f32>) -> Self {
        self.content_height = height;
        self
    }

    pub fn open(&mut self) {
        self.overlay.open();
    }

    pub fn close(&mut self) {
        self.overlay.close();
    }

    pub fn is_open(&self) -> bool {
        self.overlay.open
    }

    /// Where the card is now: in place while open, behind its edge while closed; the app
    /// gives both coordinates to the card as a position override (FollowPositionCustomizer).
    /// `free_height`: the screen's height without what covers its bottom (the keyboard dock
    /// while it is open): a dialog is centred in that, or at the top when it is higher than
    /// that, as a phone moves a dialog above its keyboard. A panel keeps its edge.
    pub fn card_position(&self, screen: (f32, f32), free_height: f32) -> (f32, f32) {
        if self.overlay.open {
            let card = self.layout.card(screen, self.content_height);
            match self.layout {
                Layout::Dialog => (card.x, ((free_height - card.h) / 2.0).max(0.0)),
                Layout::Panel(_) => (card.x, card.y),
            }
        } else {
            self.layout.closed(screen)
        }
    }

    /// The scrim's opacity, 0..100.
    pub fn scrim(&self) -> f32 {
        self.overlay.slide(35.0, 0.0)
    }
}

impl Component for Sheet {
    const KIND: ComponentKind = ComponentKind::Sheet;

    fn variant(&self) -> String {
        format!("layout={}, {}", self.layout.name(), self.overlay.variant())
    }

    fn last_change(&self) -> Option<Change> {
        self.overlay.last_change()
    }
}

/// The transition of the sheet's last change, for the parts that move.
impl Sheet {
    pub fn transition(&self) -> Option<Transition> {
        Component::transition(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: (f32, f32) = (1920.0, 1080.0);

    #[test]
    fn the_variant_names_the_layout_and_the_state() {
        let mut s = Sheet::new(Layout::Panel(Edge::Right));
        assert_eq!(s.variant(), "layout=right, state=closed");
        s.open();
        assert_eq!(s.variant(), "layout=right, state=open");
        assert_eq!(Sheet::new(Layout::Dialog).variant(), "layout=dialog, state=closed");
    }

    #[test]
    fn a_panel_takes_half_the_screen_at_its_edge() {
        assert_eq!(Layout::Panel(Edge::Right).card(SCREEN, None), Rect { x: 960.0, y: 0.0, w: 960.0, h: 1080.0 });
        assert_eq!(Layout::Panel(Edge::Left).card(SCREEN, None), Rect { x: 0.0, y: 0.0, w: 960.0, h: 1080.0 });
        assert_eq!(Layout::Panel(Edge::Bottom).card(SCREEN, None), Rect { x: 0.0, y: 540.0, w: 1920.0, h: 540.0 });
        assert_eq!(Layout::Panel(Edge::Top).card(SCREEN, None), Rect { x: 0.0, y: 0.0, w: 1920.0, h: 540.0 });
        assert_eq!(Layout::Dialog.card(SCREEN, None), Rect { x: 400.0, y: 120.0, w: 1120.0, h: 840.0 });
        // the card hugs its content: 392 of content, 40 around it, in the middle
        assert_eq!(Layout::Dialog.card(SCREEN, Some(392.0)), Rect { x: 400.0, y: 304.0, w: 1120.0, h: 472.0 });
        assert_eq!(Layout::Panel(Edge::Right).card(SCREEN, Some(392.0)).h, 1080.0, "a panel ignores it");
    }

    #[test]
    fn the_closed_card_waits_behind_its_edge() {
        assert_eq!(Layout::Panel(Edge::Right).closed(SCREEN), (1960.0, 0.0));
        assert_eq!(Layout::Panel(Edge::Left).closed(SCREEN), (-1000.0, 0.0));
        assert_eq!(Layout::Panel(Edge::Top).closed(SCREEN), (0.0, -580.0));
        assert_eq!(Layout::Panel(Edge::Bottom).closed(SCREEN), (0.0, 1120.0));
        assert_eq!(Layout::Dialog.closed(SCREEN), (400.0, 1120.0));
    }

    #[test]
    fn open_moves_the_card_in_with_the_overlay_transition() {
        let mut s = Sheet::new(Layout::Panel(Edge::Right));
        assert_eq!(s.card_position(SCREEN, SCREEN.1), (1960.0, 0.0));
        assert_eq!((s.scrim(), s.transition()), (0.0, None));
        s.open();
        assert_eq!(s.card_position(SCREEN, SCREEN.1), (960.0, 0.0));
        assert_eq!(s.card_position(SCREEN, 660.0), (960.0, 0.0), "a panel keeps its edge over a keyboard");
        assert_eq!(s.scrim(), 35.0);
        assert_eq!(s.transition().expect("open transition").duration.as_millis(), 550);
        s.close();
        assert_eq!(s.transition().expect("close transition").duration.as_millis(), 400);
    }

    #[test]
    fn a_dialog_opens_in_the_middle_for_its_content() {
        let mut s = Sheet::new(Layout::Dialog).with_content_height(Some(620.0));
        s.open();
        assert_eq!(s.card_position(SCREEN, SCREEN.1), (400.0, 190.0), "620 + 80 = 700 high, centred");
        assert_eq!(s.with_content_height(None).card_position(SCREEN, SCREEN.1), (400.0, 120.0), "unknown: the default 840");
    }

    #[test]
    fn a_dialog_moves_above_the_keyboard() {
        let mut s = Sheet::new(Layout::Dialog).with_content_height(Some(312.0));
        s.open();
        assert_eq!(s.card_position(SCREEN, SCREEN.1), (400.0, 344.0), "392 high, centred in 1080");
        assert_eq!(s.card_position(SCREEN, 660.0), (400.0, 134.0), "centred in the 660 above the dock");
        let tall = s.with_content_height(Some(760.0));
        assert_eq!(tall.card_position(SCREEN, 660.0), (400.0, 0.0), "840 does not fit above the dock: at the top");
        s.close();
        assert_eq!(s.card_position(SCREEN, 660.0), (400.0, 1120.0), "closed: behind the screen's edge, not the dock's");
    }

    #[test]
    fn every_layout_has_a_name_for_the_figma_property() {
        assert_eq!(Layout::ALL.map(Layout::name), ["dialog", "left", "right", "top", "bottom"]);
    }
}

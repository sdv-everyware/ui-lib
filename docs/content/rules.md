## Rules (what every harry app with this library does)
1. One `Slot` field in the UI model per component on the screen, bound to the slot's node name; the presenter fills it with `slot("#node", &component)` (lists: `list(...)`, scrollbars: `scrollbar(...)`, a grid of one component: `grid("#node", &cells, columns, (pitch_x, pitch_y))`, one field for the whole grid, taps as `"#node/<index>"` through `row_of`). A whole component is hidden with `slot_shown("#node", &component, false)` or `grid_shown(...)`: opacity 0 and no taps, so a tap reaches what lies under it; never a hidden layer.
2. The variant string is exact and ordered as the set's properties: `kind=primary, state=default, size=regular`. `Component::variant()` produces it; never type it by hand in an app.
3. A variant change rebuilds the node (~50 ms on the RB3): the variant carries only what rarely changes (kind, disabled, selected). Pressed, fades, thumbs, carets are meters and opacities (`parts`), display-only, smooth.
4. Gestures: the app routes `HomeAction::Gesture(node, gesture)` by node name to the component's `on(gesture)`; the component answers with an event (`ButtonEvent::Clicked`, `ListEvent::Chosen(row)`, `FormEvent::Submitted`).
5. Transitions come from components.json through the component (`transition()`), never from Figma's prototype: harry does not read prototype animations.
6. Nothing hidden in Figma: a hidden layer is `display: none` forever. The app hides with opacity or visibility.
7. A sheet over the screen is the library's `#sheet` with a content component swapped into its slot; the app never sets the sheet's variant, it moves the card (`Sheet::card_position`) and fades the scrim.
8. Texts the code writes end with an ellipsis after their lines (`maxLines`, `textTruncation`), letter spacing 0, a preview text of at least a space.
9. Pictures only on a plain rectangle node (`#list-item/image`, `#tile/image`), registered by key (`everyware_ui::images`).
10. Check the contract against the design: `everyware_ui_test` (host) fails when a variant or a part the code uses is not in the app's .dcf.
11. Behaviour is code: what a tap does, which variant shows, the data. Figma's prototype (taps, overlays, transitions in Play) is for designers only and never reaches harry; the prototype is a picture of the code, the device is right when they differ.
12. The library knows no use case: a component is named by its role (pictogram, bar chart, list item), never by what an app shows with it; the mapping from an app's data to a component's values is the app's (`scripts/check-roles.py`).

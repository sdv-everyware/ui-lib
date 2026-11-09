---
title: Dropdown
role: A closed pill that opens a list of items below it; the open list and the selected item are opacities, the variant only the state.
rust: core/src/dropdown.rs
types: Dropdown, DropdownEvent
---
`state` is `default` or `disabled`. Open is `#dropdown/list` faded in, the chosen row `#dropdown/item/N/selected` lit, both display-only. The pill shows the selected item's text (`#dropdown/value`); the items report taps by their own names (`#dropdown/item/2`), unambiguous while one dropdown is open on the screen. `on_field(gesture)` opens and closes, `on_item(i, gesture)` answers `Selected(i)`.

## In Figma
The slot keeps the closed pill's height (48) and lets the list hang below it (counter-align MIN, no clipping on the slot), so a row of the dashboard does not grow by the open list. The component is drawn as tall as its open list.

## Traps
- A slot as tall as the component centres the pill half above the row (seen after a regenerate): slot height = the pill, `frameMaskDisabled` on the slot.
- Items are a fixed number in the set (the list is drawn); more items than rows are not shown. The app fills the texts, it cannot add rows.

// figma-tweaks.js: the code's changes on top of component sets the designers drew in Figma
// ("figma": true in components.json). The drawing stays Figma's (design/source/library.json,
// fetched); a tweak is applied to it at every build, before the set is checked against
// components.json. Use it when the look of a designer's component has to change now: a colour,
// a layer the code needs, a variant.
//
//   module.exports = {
//     '#stepper': t => {
//       for (const v of t.variants()) {
//         t.set(t.find(v, 'bg'), { cornerRadius: 12 });                      // a value
//         if (!t.find(v, '#stepper/pressed')) {                               // a layer, once
//           const p = t.A.rect(v, '#stepper/pressed', 0, 0, v.size.x, v.size.y, t.C.accentPressed, 12, { opacity: 0 });
//           t.moveBelow(p, t.find(v, '#stepper/value'));
//         }
//       }
//       t.addVariant('state=default', 'state=disabled', v => t.set(v, { opacity: 0.4 }));   // a variant, once
//     },
//   };
//
// t: node (the set's node), variants() and variant(name), find(variant, layer) and all(variant),
// set(node, props) (assigns only what differs), fill(node, rgb), moveBelow(node, sibling),
// addVariant(from, name, edit), A (fig-helpers' authoring: rect, text, frame, ...), C, FONT.
//
// Write a tweak so that applying it to its own result changes nothing: set values, add a layer
// or a variant only when it is not there (set, fill and addVariant do that). The build tells by
// it where the change is:
// - the tweak changes the set: the designers do not have the change; the build says to paste
//   the set from design/build/library.fig into their Figma file;
// - it changes nothing: Figma has it (pasted and fetched); the build says to delete the tweak.
module.exports = {
};

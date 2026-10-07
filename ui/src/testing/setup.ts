import "@testing-library/jest-dom/vitest";

// This file used to build a second JSDOM here and assign its window, document,
// navigator, HTMLElement, Node and NodeList over the globals. Vitest's `jsdom`
// environment (see vite.config.ts) already provides all of them, so those
// assignments only ever swapped in a *different* document than the one the
// environment had created.
//
// That was harmless until @testing-library/jest-dom 7, whose entry point pulls
// in @testing-library/dom. `screen` is bound to `document.body` when that module
// is first evaluated, and ESM hoists the import above any statement here — so
// `screen` bound to the environment's document while `render` wrote into the
// replacement. Every `screen.getByRole`/`getByText` query then searched an empty
// document: 109 tests across 14 files failed with "Unable to find an accessible
// element" while the components under test rendered perfectly.

export type GeneralKeys =
  | "f" // fullscreen
  | "/" // keyboard shortcuts dialog
  | "s" // manual hard save
  | "r" // reader dialog
  | "t" // transformer dialog
  | "w" // writer dialog
  | "p" // bottom panel preview
  | "c"; // left panel canvas navigator

export type EditorKeys =
  | "f" // fullscreen
  | "r" // reader dialog
  | "t" // transformer dialog
  | "w" // writer dialog
  | "p" // bottom panel preview
  | "a" // SHIFT + a = left panel actions list
  | "r" // SHIFT + r = left panel resources
  | "c" // SHIFT + c = left panel canvas navigator
  | "s"; // new subworkflow with select nodes

export type CanvasKeys =
  | "c" // w CMD = Copy, wout CMD = left panel canvas navigator
  | "x" // cut
  | "v" // paste
  | "z" // w CMD = undo, w CMD + SHIFT = redo
  | "e" // disable/enable node
  | "+" // zoom in
  | "=" // zoom in (alternative - depends on keyboard layout)
  | "-" // zoom out
  | "0"; // fit view

export type PossibleKeys = GeneralKeys | EditorKeys | CanvasKeys;

type PossibleActions =
  | "zoomIn"
  | "zoomOut"
  | "fitView"
  | "copy"
  | "cut"
  | "paste"
  | "undo"
  | "redo"
  | "disableNode"
  | "fullscreen"
  | "shortcutsDialog"
  | "save"
  | "readerDialog"
  | "transformerDialog"
  | "writerDialog"
  | "leftPanelCanvasNavigator"
  | "leftPanelActionsList"
  | "leftPanelResources"
  | "groupToSubWorkFlow";

export type KeyBinding<K extends PossibleKeys = PossibleKeys> = {
  key: K;
  commandKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
};

export type Shortcut<K extends PossibleKeys = PossibleKeys> = {
  keyBinding?: KeyBinding<K>;
  description: string;
};

export type Shortcuts<K extends PossibleKeys = PossibleKeys> = {
  title: string;
  shortcuts: Shortcut<K>[];
};

export const GeneralKeyBindings: Partial<
  Record<PossibleActions, KeyBinding<GeneralKeys>>
> = {
  shortcutsDialog: { key: "/", commandKey: true },
  save: { key: "s", commandKey: true },
};

export const EditorKeyBindings: Partial<
  Record<PossibleActions, KeyBinding<EditorKeys>>
> = {
  fullscreen: { key: "f", commandKey: true },
  readerDialog: { key: "r" },
  transformerDialog: { key: "t" },
  writerDialog: { key: "w" },
  leftPanelCanvasNavigator: { key: "c", shiftKey: true },
  leftPanelActionsList: { key: "a", shiftKey: true },
  leftPanelResources: { key: "r", shiftKey: true },
  groupToSubWorkFlow: { key: "s" },
};

export const CanvasKeyBindings: Partial<
  Record<PossibleActions, KeyBinding<CanvasKeys>>
> = {
  copy: { key: "c", commandKey: true },
  cut: { key: "x", commandKey: true },
  paste: { key: "v", commandKey: true },
  undo: { key: "z", commandKey: true },
  redo: { key: "z", commandKey: true, shiftKey: true },
  disableNode: { key: "e", commandKey: true },
  zoomIn: { key: "+" },
  zoomOut: { key: "-" },
  fitView: { key: "0", commandKey: true },
};

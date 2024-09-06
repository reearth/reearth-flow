export type PossibleKeys =
  | "+" // zoom in
  | "=" // zoom in
  | "-" // zoom out
  | "0" // fit view
  | "f" // fullscreen
  | "/" // keyboard shortcuts dialog
  | "r" // reader dialog
  | "t" // transformer dialog
  | "w" // writer dialog
  | "l" // bottom panel logs
  | "p" // bottom panel preview
  | "c"; // left panel canvas navigator

type PossibleActions =
  | "zoomIn"
  | "zoomOut"
  | "fitView"
  | "fullscreen"
  | "shortcutsDialog"
  | "readerDialog"
  | "transformerDialog"
  | "writerDialog"
  | "bottomPanelLogs"
  | "bottomPanelPreview"
  | "leftPanelCanvasNavigator";

export type KeyBinding = {
  key: Partial<PossibleKeys>;
  commandKey?: boolean;
};

export type Shortcut = {
  keyBinding?: KeyBinding;
  description: string;
};

export type Shortcuts = {
  title: string;
  shortcuts: Shortcut[];
};

export const GeneralKeyBindings: Partial<Record<PossibleActions, KeyBinding>> =
  {
    shortcutsDialog: { key: "/", commandKey: true },
  };

export const EditorKeyBindings: Partial<Record<PossibleActions, KeyBinding>> = {
  fullscreen: { key: "f", commandKey: true },
  readerDialog: { key: "r" },
  transformerDialog: { key: "t" },
  writerDialog: { key: "w" },
  bottomPanelLogs: { key: "l", commandKey: true },
  bottomPanelPreview: { key: "p", commandKey: true },
  leftPanelCanvasNavigator: { key: "c", commandKey: true },
};

export const CanvasKeyBindings: Partial<Record<PossibleActions, KeyBinding>> = {
  zoomIn: { key: "+" },
  zoomOut: { key: "-" },
  fitView: { key: "0", commandKey: true },
};

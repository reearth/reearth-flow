type EditorKeys = "⌘K" | "⌘P" | "⌘X" | "⌘⇧Y";

type CanvasKeys = "⌘N" | "⌘S" | "⌘Z" | "⌘⇧Z";

type Shortcut<T = "editor" | "canvas"> = {
  key: T extends "editor" ? EditorKeys : T extends "canvas" ? CanvasKeys : null;
  description: string;
};

export type Shortcuts<T = "editor" | "canvas"> = {
  title: string;
  shortcuts: Shortcut<T>[];
};

import { useT } from "@flow/lib/i18n";
import {
  Shortcuts,
  EditorKeyBindings,
  CanvasKeyBindings,
  GeneralKeyBindings,
  CanvasKeys,
  EditorKeys,
  GeneralKeys,
} from "@flow/types";

export default () => {
  const t = useT();

  const title = t("Keyboard shortcuts");

  const generalShortcuts: Shortcuts<GeneralKeys> = {
    title: t("General shortcuts"),
    shortcuts: [
      {
        keyBinding: GeneralKeyBindings["shortcutsDialog"],
        description: t("Open the keyboard shortcuts dialog"),
      },
    ],
  };

  const editorShortcuts: Shortcuts<EditorKeys> = {
    title: t("Editor shortcuts"),
    shortcuts: [
      {
        keyBinding: EditorKeyBindings["fullscreen"],
        description: t("Toggle fullscreen mode"),
      },
      {
        keyBinding: EditorKeyBindings["readerDialog"],
        description: t("Open the reader dialog"),
      },
      {
        keyBinding: EditorKeyBindings["transformerDialog"],
        description: t("Open the transformer dialog"),
      },
      {
        keyBinding: EditorKeyBindings["writerDialog"],
        description: t("Open the writer dialog"),
      },
      {
        keyBinding: EditorKeyBindings["bottomPanelLogs"],
        description: t("Toggle the logs panel"),
      },
      {
        keyBinding: EditorKeyBindings["bottomPanelPreview"],
        description: t("Toggle the preview panel"),
      },
      {
        keyBinding: EditorKeyBindings["leftPanelCanvasNavigator"],
        description: t("Toggle the canvas navigator panel"),
      },
    ],
  };

  const canvasShortcuts: Shortcuts<CanvasKeys> = {
    title: t("Canvas shortcuts"),
    shortcuts: [
      {
        keyBinding: CanvasKeyBindings["copy"],
        description: t("Copy the selected nodes"),
      },
      {
        keyBinding: CanvasKeyBindings["paste"],
        description: t("Paste the copied nodes"),
      },
      {
        keyBinding: CanvasKeyBindings["undo"],
        description: t("Undo the last action"),
      },
      {
        keyBinding: CanvasKeyBindings["redo"],
        description: t("Redo the last action"),
      },
      {
        keyBinding: CanvasKeyBindings["zoomIn"],
        description: t("Zoom in on the canvas"),
      },
      {
        keyBinding: CanvasKeyBindings["zoomOut"],
        description: t("Zoom out on the canvas"),
      },
      {
        keyBinding: CanvasKeyBindings["fitView"],
        description: t("Fit the canvas to the viewport"),
      },
    ],
  };

  return {
    title,
    generalShortcuts,
    editorShortcuts,
    canvasShortcuts,
  };
};

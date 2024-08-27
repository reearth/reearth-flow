import { useT } from "@flow/lib/i18n";
import { Shortcuts, EditorKeyBindings, CanvasKeyBindings } from "@flow/types";

export default () => {
  const t = useT();

  const title = t("Keyboard shortcuts");

  const editorShortcuts: Shortcuts = {
    title: t("Editor shortcuts"),
    shortcuts: [
      {
        keyBinding: EditorKeyBindings["fullscreen"],
        description: t("Toggle fullscreen mode"),
      },
      {
        keyBinding: EditorKeyBindings["shortcutsDialog"],
        description: t("Open the keyboard shortcuts dialog"),
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

  const canvasShortcuts: Shortcuts = {
    title: t("Canvas shortcuts"),
    shortcuts: [
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
    editorShortcuts,
    canvasShortcuts,
  };
};

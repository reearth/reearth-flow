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

  const title = t("Keyboard Shortcuts");

  const generalShortcuts: Shortcuts<GeneralKeys> = {
    title: t("General shortcuts"),
    shortcuts: [
      {
        keyBinding: GeneralKeyBindings["shortcutsDialog"],
        description: t("Open the Keyboard Shortcuts Dialog"),
      },
      {
        keyBinding: GeneralKeyBindings["save"],
        description: t("Manually Save the Project"),
      },
    ],
  };

  const editorShortcuts: Shortcuts<EditorKeys> = {
    title: t("Editor shortcuts"),
    shortcuts: [
      {
        keyBinding: EditorKeyBindings["fullscreen"],
        description: t("Toggle Fullscreen Mode"),
      },
      {
        keyBinding: EditorKeyBindings["writerDialog"],
        description: t("Open the Writer Dialog"),
      },
      {
        keyBinding: EditorKeyBindings["readerDialog"],
        description: t("Open the Reader Dialog"),
      },
      {
        keyBinding: EditorKeyBindings["transformerDialog"],
        description: t("Open the Transformer Dialog"),
      },
      {
        keyBinding: EditorKeyBindings["groupToSubWorkFlow"],
        description: t("Group Selected Nodes into a Sub-Workflow"),
      },
    ],
  };

  const canvasShortcuts: Shortcuts<CanvasKeys> = {
    title: t("Canvas Shortcuts"),
    shortcuts: [
      {
        keyBinding: CanvasKeyBindings["copy"],
        description: t("Copy the Selected Nodes"),
      },
      {
        keyBinding: CanvasKeyBindings["cut"],
        description: t("Cut the Selected Nodes"),
      },
      {
        keyBinding: CanvasKeyBindings["paste"],
        description: t("Paste the Copied Nodes"),
      },
      {
        keyBinding: CanvasKeyBindings["undo"],
        description: t("Undo the Last Action"),
      },
      {
        keyBinding: CanvasKeyBindings["redo"],
        description: t("Redo the Last Action"),
      },
      {
        keyBinding: CanvasKeyBindings["disableNode"],
        description: t("Disable/Enable the Selected Node"),
      },
      {
        keyBinding: CanvasKeyBindings["zoomIn"],
        description: t("Zoom in on the Canvas"),
      },
      {
        keyBinding: CanvasKeyBindings["zoomOut"],
        description: t("Zoom out on the Canvas"),
      },
      {
        keyBinding: CanvasKeyBindings["fitView"],
        description: t("Fit the Canvas to the Viewport"),
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

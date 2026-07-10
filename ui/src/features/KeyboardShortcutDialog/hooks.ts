import { useT } from "@flow/lib/i18n";
import {
  Shortcuts,
  EditorKeyBindings,
  CanvasKeyBindings,
  GeneralKeyBindings,
  CanvasKeys,
  EditorKeys,
  GeneralKeys,
  DebugKeys,
  DebugKeyBindings,
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
      {
        keyBinding: GeneralKeyBindings["lockProject"],
        description: t("Lock/Unlock the Project"),
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
        keyBinding: EditorKeyBindings["openSearch"],
        description: t("Open Canvas Search"),
      },
      {
        keyBinding: EditorKeyBindings["groupToSubWorkFlow"],
        description: t("Sub-Workflow from Selection"),
      },
    ],
  };

  const canvasShortcuts: Shortcuts<CanvasKeys> = {
    title: t("Canvas Shortcuts"),
    shortcuts: [
      {
        keyBinding: CanvasKeyBindings["selectAll"],
        description: t("Select All Actions"),
      },
      {
        keyBinding: CanvasKeyBindings["copy"],
        description: t("Copy the Selected Actions"),
      },
      {
        keyBinding: CanvasKeyBindings["cut"],
        description: t("Cut the Selected Actions"),
      },
      {
        keyBinding: CanvasKeyBindings["paste"],
        description: t("Paste the Copied Actions"),
      },
      {
        keyBinding: CanvasKeyBindings["spreadNodes"],
        description: t("Spread the Selected Actions"),
      },
      {
        keyBinding: CanvasKeyBindings["compressNodes"],
        description: t("Compress the Selected Actions"),
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
        description: t("Disable/Enable the Selected Actions"),
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

  const debugShortcuts: Shortcuts<DebugKeys> = {
    title: t("Debug Shortcuts"),
    shortcuts: [
      {
        keyBinding: DebugKeyBindings["startDebugRun"],
        description: t("Run Debug Workflow"),
      },
      {
        keyBinding: DebugKeyBindings["runDebugFromSelected"],
        description: t("Run Debug from Selected"),
      },
      {
        keyBinding: DebugKeyBindings["cancelDebugRun"],
        description: t("Stop Debug Workflow"),
      },
      {
        keyBinding: DebugKeyBindings["clearDebugResults"],
        description: t("Clear Debug Results"),
      },
    ],
  };

  return {
    title,
    generalShortcuts,
    editorShortcuts,
    canvasShortcuts,
    debugShortcuts,
  };
};

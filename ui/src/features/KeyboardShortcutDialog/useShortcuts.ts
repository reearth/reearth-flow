import { useT } from "@flow/lib/i18n";
import { Shortcuts } from "@flow/types/shortcuts";

export default () => {
  const t = useT();

  const title = t("Keyboard shortcuts");

  const editorShortcuts: Shortcuts<"editor"> = {
    title: t("Editor shortcuts"),
    shortcuts: [
      {
        key: "⌘K",
        description: t("Create a new document"),
      },
      {
        key: "⌘P",
        description: t("Save the current document"),
      },
      {
        key: "⌘X",
        description: t("Create a new document"),
      },
      {
        key: "⌘⇧Y",
        description: t("Save the current document"),
      },
    ],
  };

  const canvasShortcuts: Shortcuts<"canvas"> = {
    title: t("Canvas shortcuts"),
    shortcuts: [
      {
        key: "⌘N",
        description: t("Create a new document"),
      },
      {
        key: "⌘S",
        description: t("Save the current document"),
      },
      {
        key: "⌘Z",
        description: t("Undo the last action"),
      },
      {
        key: "⌘⇧Z",
        description: t("Redo the last action"),
      },
    ],
  };

  return {
    title,
    editorShortcuts,
    canvasShortcuts,
  };
};

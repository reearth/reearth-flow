import { useT } from "@flow/providers";

export default () => {
  const t = useT();

  const title = t("Keyboard shortcuts");

  const description = t(
    "This is a list of keyboard shortcuts that you can use to navigate the application.",
  );

  const editorShortcuts = {
    title: t("Editor shortcuts"),
    shortcuts: [
      {
        key: "⌘N",
        description: t("Create a new document"),
      },
      {
        key: "⌘S",
        description: t("Save the current document"),
      },
    ],
  };

  const canvasShortcuts = {
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
    description,
    editorShortcuts,
    canvasShortcuts,
  };
};

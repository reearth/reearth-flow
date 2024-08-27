import { useEffect, useState } from "react";

import type { KeyBinding, PossibleKeys } from "@flow/types";

type ShortcutProps = {
  keyBinding: KeyBinding;
  callback: () => void;
};

export default (shortcuts?: ShortcutProps[]) => {
  const [disableShortcuts, setDisableShortcuts] = useState(false);

  // Disable shortcuts when input is focused
  useEffect(() => {
    const handleFocusIn = (event: FocusEvent) => {
      const target = event.target as HTMLElement;
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable
      ) {
        setDisableShortcuts(true);
      }
    };

    const handleFocusOut = () => {
      setDisableShortcuts(false);
    };

    document.addEventListener("focusin", handleFocusIn);
    document.addEventListener("focusout", handleFocusOut);

    return () => {
      document.removeEventListener("focusin", handleFocusIn);
      document.removeEventListener("focusout", handleFocusOut);
    };
  }, []);

  useEffect(() => {
    if (!shortcuts || disableShortcuts) return;

    const handleShortcuts = (e: KeyboardEvent) => {
      for (const shortcut of shortcuts) {
        const { keyBinding } = shortcut;

        const eventKey = getKey(e.key as PossibleKeys);
        if (eventKey === keyBinding.key) {
          if (
            (keyBinding.commandKey && !e.metaKey && !e.ctrlKey) ||
            (!keyBinding.commandKey && (e.metaKey || e.ctrlKey))
          )
            return;
          e.preventDefault();
          shortcut.callback();
        }
      }
    };

    document.addEventListener("keydown", handleShortcuts);

    return () => {
      document.removeEventListener("keydown", handleShortcuts);
    };
  }, [shortcuts, disableShortcuts]);
};

const getKey = (key: PossibleKeys): PossibleKeys => {
  return key === "=" ? "+" : key;
};

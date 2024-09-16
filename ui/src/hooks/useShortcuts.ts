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
        const { keyBinding, callback } = shortcut;

        const eventKey = getKey(e.key as PossibleKeys).toLowerCase();
        if (eventKey === keyBinding.key) {
          const isCommandKeyPressed = e.metaKey || e.ctrlKey;
          const isShiftKeyPressed = e.shiftKey;
          const isAltKeyPressed = e.altKey;

          // 1. Check if the keybinding requires the command key, but it's not pressed
          if (keyBinding.commandKey && !isCommandKeyPressed) continue;

          // 2. Check if the keybinding requires the shift key, but it's not pressed
          if (keyBinding.shiftKey && !isShiftKeyPressed) continue;

          // 3. Check if the keybinding requires the alt key, but it's not pressed
          if (keyBinding.altKey && !isAltKeyPressed) continue;

          // 4. Prevent simple keybinding from triggering when modifier keys are pressed
          if (!keyBinding.commandKey && isCommandKeyPressed) continue;
          if (!keyBinding.shiftKey && isShiftKeyPressed) continue;
          if (!keyBinding.altKey && isAltKeyPressed) continue;

          // If all conditions match, trigger the shortcut callback
          e.preventDefault();
          callback();
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

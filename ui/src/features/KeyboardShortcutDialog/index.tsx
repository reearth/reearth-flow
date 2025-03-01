import { X } from "@phosphor-icons/react";
import { ReactNode, useCallback, useEffect, useState } from "react";
import ReactDOM from "react-dom";

import { Shortcuts } from "./components";
import useHooks from "./hooks";

type Props = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
};

const KeyboardShortcutDialog: React.FC<Props> = ({ isOpen, onOpenChange }) => {
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsReady(true);
    }, 100);
    return () => {
      clearTimeout(timer);
    };
  }, []);

  const handlePortalClose = useCallback(() => {
    setIsReady(false);
    setTimeout(() => {
      onOpenChange(false);
    }, 100);
  }, [onOpenChange]);

  const { title, generalShortcuts, editorShortcuts, canvasShortcuts } =
    useHooks();

  return (
    <Portal isVisible={isOpen} onClose={handlePortalClose}>
      <div
        className="h-[400px] w-full rounded-t-md bg-secondary transition-all"
        style={{
          transform: `translateY(${isReady ? "8px" : "100%"})`,
          transitionDuration: "300ms",
          transitionProperty: "transform",
          transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
        }}
        onClick={(e) => e.stopPropagation()}>
        <div className="relative flex h-[40px] items-center justify-center rounded-t-lg border-y border-b-primary">
          <p>{title}</p>
          <X
            className="absolute right-3 cursor-pointer"
            onClick={handlePortalClose}
          />
        </div>
        <div className="flex h-[352px] flex-wrap gap-4 p-4">
          <div className="flex h-[320px] flex-1 flex-col gap-1">
            <p className="font-light">{generalShortcuts.title}</p>
            <div className="overflow-auto">
              <Shortcuts shortcuts={generalShortcuts.shortcuts} />
            </div>
          </div>
          <div className="flex h-[320px] flex-1 flex-col gap-1">
            <p className="font-light">{editorShortcuts.title}</p>
            <div className="overflow-auto">
              <Shortcuts shortcuts={editorShortcuts.shortcuts} />
            </div>
          </div>
          <div className="flex h-[320px] flex-1 flex-col gap-1">
            <p className="font-light">{canvasShortcuts.title}</p>
            <div className="overflow-auto">
              <Shortcuts shortcuts={canvasShortcuts.shortcuts} />
            </div>
          </div>
        </div>
      </div>
    </Portal>
  );
};

export default KeyboardShortcutDialog;

type BottomPortalProps = {
  children: ReactNode;
  isVisible: boolean;
  onClose: () => void;
};

const Portal: React.FC<BottomPortalProps> = ({
  children,
  isVisible,
  onClose,
}) => {
  const portalContainer = document.getElementById("portal-root");

  useEffect(() => {
    const handleEscPress = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose(); // Call the onClose function passed in props
      }
    };

    if (isVisible) {
      document.addEventListener("keydown", handleEscPress);
    }

    return () => {
      document.removeEventListener("keydown", handleEscPress);
    };
  }, [isVisible, onClose]);

  if (!portalContainer || !isVisible) {
    return <>{children}</>;
  }

  return ReactDOM.createPortal(
    <div
      className="fixed inset-0 z-[1000] flex items-end justify-center"
      onClick={onClose}>
      {children}
    </div>,
    portalContainer,
  );
};

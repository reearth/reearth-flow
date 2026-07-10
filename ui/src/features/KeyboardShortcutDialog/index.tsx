import { KeyboardIcon, XIcon } from "@phosphor-icons/react";
import { ReactNode, useCallback, useEffect, useState } from "react";
import ReactDOM from "react-dom";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@flow/components";

import { Shortcuts } from "./components";
import useHooks from "./hooks";

type Props = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
};

const KeyboardShortcutDialog: React.FC<Props> = ({ isOpen, onOpenChange }) => {
  const [isReady, setIsReady] = useState(false);
  const [tabValue, setTabValue] = useState("general");
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

  const {
    title,
    generalShortcuts,
    editorShortcuts,
    canvasShortcuts,
    debugShortcuts,
  } = useHooks();
  return (
    <Portal isVisible={isOpen} onClose={handlePortalClose}>
      <div
        className="h-[350px] w-full rounded-t-2xl border-t bg-secondary p-4 transition-all"
        style={{
          transform: `translateY(${isReady ? "8px" : "100%"})`,
          transitionDuration: "300ms",
          transitionProperty: "transform",
          transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
        }}
        onClick={(e) => e.stopPropagation()}>
        <div className="relative flex items-center gap-2 pb-4">
          <KeyboardIcon />
          <p>{title}</p>
          <XIcon
            className="absolute top-0 right-0 cursor-pointer"
            onClick={handlePortalClose}
          />
        </div>
        <Tabs value={tabValue} onValueChange={setTabValue}>
          <div className="flex w-full">
            <TabsList className="align-center mb-4 flex w-full justify-center gap-2">
              <TabsTrigger value="general">
                {generalShortcuts.title}
              </TabsTrigger>
              <TabsTrigger value="editor">{editorShortcuts.title}</TabsTrigger>
              <TabsTrigger value="canvas">{canvasShortcuts.title}</TabsTrigger>
              <TabsTrigger value="debug">{debugShortcuts.title}</TabsTrigger>
            </TabsList>
          </div>
          <TabsContent value="general">
            <div className="flex h-[200px] justify-center">
              <Shortcuts shortcuts={generalShortcuts.shortcuts} />
            </div>
          </TabsContent>
          <TabsContent value="editor">
            <div className="flex h-[200px] justify-center">
              <Shortcuts shortcuts={editorShortcuts.shortcuts} />
            </div>
          </TabsContent>
          <TabsContent value="canvas">
            <div className="flex h-[200px] justify-center">
              <Shortcuts shortcuts={canvasShortcuts.shortcuts} />
            </div>
          </TabsContent>
          <TabsContent value="debug">
            <div className="flex h-[200px] justify-center">
              <Shortcuts shortcuts={debugShortcuts.shortcuts} />
            </div>
          </TabsContent>
        </Tabs>
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
      className="fixed inset-0 z-1000 flex items-end justify-center"
      onClick={onClose}>
      {children}
    </div>,
    portalContainer,
  );
};

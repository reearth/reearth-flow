import { XYPosition } from "@xyflow/react";

import type { KeyBinding, Node } from "@flow/types";

type ContextMenuStyles = {
  styles: React.CSSProperties;
  mousePosition?: XYPosition;
};

type CanvasContextMenuMeta = {
  data?: Node | Node[];
};

export type ContextMenuMeta = ContextMenuStyles & CanvasContextMenuMeta;
type ContextMenuProps = {
  items: ContextMenuItemType[];
  contextMenuMeta: ContextMenuMeta;
};
const os = window.navigator.userAgent.toLowerCase();

const ContextMenu: React.FC<ContextMenuProps> = ({
  items,
  contextMenuMeta,
}) => {
  return (
    <div className="absolute z-50" style={{ ...contextMenuMeta.styles }}>
      <div className="min-w-[160px] select-none rounded-md border bg-card p-1 text-popover-foreground shadow-md">
        {items.map((item, index) =>
          item.type === "action" ? (
            <ContextMenuItem key={index} {...item.props} />
          ) : (
            <ContextMenuSeparator key={index} />
          ),
        )}
      </div>{" "}
    </div>
  );
};

type ContextMenuItemProps = {
  label: string;
  icon?: React.ReactNode;
  shortcut?: React.ReactNode;
  className?: string;
  destructive?: boolean;
  disabled?: boolean;
  onCallback: () => void;
};

export type ContextMenuItemType =
  | { type: "action"; props: ContextMenuItemProps }
  | { type: "separator" };

const ContextMenuItem: React.FC<ContextMenuItemProps> = ({
  label,
  icon,
  shortcut,
  className,
  destructive,
  disabled,
  onCallback,
}) => {
  return (
    <div
      className={`flex items-center justify-between rounded-sm px-2 py-1.5 text-xs ${destructive ? "text-destructive" : ""} ${
        disabled
          ? "pointer-events-none opacity-50 text-muted-foreground"
          : "hover:bg-accent cursor-pointer"
      } hover:bg-accent ${className}`}
      onClick={() => {
        if (!disabled) {
          onCallback();
        }
      }}>
      <div className="flex items-center gap-1">
        {icon}
        <p>{label}</p>
      </div>
      <div className="flex flex-row gap-1">{shortcut}</div>
    </div>
  );
};

const ContextMenuSeparator: React.FC = () => (
  <div className="-mx-1 my-1 h-px bg-border" />
);

const ContextMenuShortcut = ({ keyBinding }: { keyBinding?: KeyBinding }) => {
  const commandKey = keyBinding?.commandKey
    ? os.indexOf("mac os x") !== -1
      ? "⌘"
      : "CTRL"
    : undefined;

  const shiftKey = keyBinding?.shiftKey ? "SHIFT" : undefined;
  const altKey = keyBinding?.altKey ? "ALT" : undefined;

  return (
    <>
      {commandKey && <KeyStroke keystroke={commandKey} />}
      {shiftKey && <KeyStroke keystroke={shiftKey} />}
      {altKey && <KeyStroke keystroke={altKey} />}
      <KeyStroke keystroke={keyBinding?.key.toUpperCase()} />
    </>
  );
};

const KeyStroke = ({ keystroke }: { keystroke?: string }) => (
  <div className="flex min-h-1 min-w-1 items-center rounded bg-accent px-1">
    <p className="text-xs dark:font-extralight">{keystroke}</p>
  </div>
);
export {
  ContextMenu,
  ContextMenuItem,
  ContextMenuShortcut,
  ContextMenuSeparator,
};

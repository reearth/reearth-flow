import type { Node } from "@flow/types";

type ContextMenuStyles = {
  styles: React.CSSProperties;
};

type NodeContextMenuMeta = {
  type: "node";
  data: Node;
};

type SelectionContextMenuMeta = {
  type: "selection";
  data: Node[];
};

type PaneContextMenuMeta = {
  type: "pane";
};

export type ContextMenuMeta =
  | (ContextMenuStyles & NodeContextMenuMeta)
  | (ContextMenuStyles & SelectionContextMenuMeta)
  | (ContextMenuStyles & PaneContextMenuMeta);

type ContextMenuProps = {
  items: ContextMenuItemType[];
  contextMenuMeta: ContextMenuMeta;
};

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
  className?: string;
  onCallback: () => void;
  destructive?: boolean;
  disabled?: boolean;
};

export type ContextMenuItemType =
  | { type: "action"; props: ContextMenuItemProps }
  | { type: "separator" };

const ContextMenuItem: React.FC<ContextMenuItemProps> = ({
  label,
  icon,
  className,
  destructive,
  disabled,
  onCallback,
}) => {
  return (
    <>
      <div
        className={`flex items-center justify-between gap-4 rounded-sm px-2 py-1.5 text-xs ${destructive ? "text-destructive" : ""} ${
          disabled
            ? "pointer-events-none opacity-50 text-muted-foreground"
            : "hover:bg-accent cursor-pointer"
        } hover:bg-accent ${className}`}
        onClick={() => {
          if (!disabled) {
            onCallback();
          }
        }}>
        <p>{label}</p>
        {icon}
      </div>
    </>
  );
};

const ContextMenuSeparator: React.FC = () => (
  <div className="-mx-1 my-1 h-px bg-border" />
);

export { ContextMenu, ContextMenuItem, ContextMenuSeparator };

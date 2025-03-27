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

export type ContextMenuMeta =
  | (ContextMenuStyles & NodeContextMenuMeta)
  | (ContextMenuStyles & SelectionContextMenuMeta);

type ContextMenuProps = {
  items: ContextMenuItemProps[];
  contextMenuMeta: ContextMenuMeta;
};

const ContextMenu: React.FC<ContextMenuProps> = ({
  items,
  contextMenuMeta,
}) => {
  return (
    <div className="absolute z-50" style={{ ...contextMenuMeta.styles }}>
      <div className="min-w-[160px] select-none rounded-md border bg-card p-1 text-popover-foreground shadow-md">
        {items.map((item, index) => (
          <ContextMenuItem
            key={index}
            label={item.label}
            icon={item.icon}
            onCallback={item.onCallback}
            destructive={item.destructive}
            disabled={item.disabled}
          />
        ))}
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
      {destructive && <div className="-mx-1 my-1 h-px bg-border" />}
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

export { ContextMenu, ContextMenuItem };

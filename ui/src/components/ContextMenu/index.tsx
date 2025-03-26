import type { Node } from "@flow/types";

export type ContextMenuMeta = {
  node?: Node;
  top: number | false;
  left: number | false;
  right: number | false;
  bottom: number | false;
};

type ContextMenuProps = {
  items: ContextMenuItemProps[];
  onClose: () => void;
  contextMenuMeta: ContextMenuMeta;
};

const ContextMenu: React.FC<ContextMenuProps> = ({
  items,
  onClose,
  contextMenuMeta,
}) => {
  const { top, left, right, bottom } = contextMenuMeta;
  return (
    <div
      className="absolute z-50"
      style={{
        top: top !== false ? top : undefined,
        left: left !== false ? left : undefined,
        right: right !== false ? right : undefined,
        bottom: bottom !== false ? bottom : undefined,
      }}>
      <div className="min-w-[160px] select-none rounded-md border bg-card p-1 text-popover-foreground shadow-md">
        {items.map((item, index) => (
          <ContextMenuItem
            key={index}
            label={item.label}
            icon={item.icon}
            onCallback={item.onCallback}
            onClose={onClose}
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
  onClose: () => void;
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
  onClose,
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
            onClose();
          }
        }}>
        <p>{label}</p>
        {icon}
      </div>
    </>
  );
};

export { ContextMenu, ContextMenuItem };

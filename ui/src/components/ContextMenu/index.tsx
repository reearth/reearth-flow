type ContextMenuProps = {
  items: ContextMenuItemProps[];
  onClose: () => void;
  menuPosition: { x: number; y: number };
};

const ContextMenu: React.FC<ContextMenuProps> = ({
  items,
  onClose,
  menuPosition,
}) => {
  return (
    <>
      <div
        className="absolute z-50"
        style={{
          top: menuPosition.y,
          left: menuPosition.x,
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
      {menuPosition && (
        <div
          className="fixed inset-0 z-40"
          onClick={onClose}
          onContextMenu={onClose}
        />
      )}
    </>
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
  const isDescructive = destructive ? "text-destructive" : "";

  const isDisabled = disabled
    ? "pointer-events-none opacity-50 text-muted-foreground"
    : "hover:bg-accent cursor-pointer";

  return (
    <>
      {destructive && <div className="-mx-1 my-1 h-px bg-border" />}
      <div
        className={`flex items-center justify-between gap-4 rounded-sm px-2 py-1.5 text-xs ${isDescructive} ${isDisabled} hover:bg-accent ${className}`}
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

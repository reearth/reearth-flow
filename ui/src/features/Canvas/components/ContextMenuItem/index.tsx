type Props = {
  label: string;
  icon: React.ReactNode;
  className?: string;
  onAction: () => void;
  onClose: () => void;
  destructive?: boolean;
  disabled?: boolean;
};

const CustomContextMenuItem: React.FC<Props> = ({
  label,
  icon,
  className,
  destructive,
  onAction,
  onClose,
}) => {
  const isDescructive = destructive ? "text-destructive" : "";
  return (
    <div
      className={`flex items-center justify-between gap-4 rounded-sm px-2 py-1.5 text-xs ${isDescructive} hover:bg-accent ${className}`}
      onClick={() => {
        onAction();
        onClose();
      }}>
      <p>{label}</p>
      {icon}
    </div>
  );
};

export default CustomContextMenuItem;

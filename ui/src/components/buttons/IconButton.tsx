import { Button } from "./BaseButton";

type Props = {
  key?: string;
  className?: string;
  style?: React.CSSProperties;
  icon: React.ReactNode;
  onClick?: () => void;
};

const IconButton: React.FC<Props> = ({ key, className, style, icon, onClick }) => (
  <Button
    key={key}
    className={`transition-all text-zinc-400 hover:bg-zinc-700 hover:text-zinc-100 cursor-pointer ${className}`}
    variant="ghost"
    style={style}
    size="icon"
    onClick={onClick}>
    {icon}
  </Button>
);

export { IconButton };

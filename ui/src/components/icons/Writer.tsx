import { CSSProperties } from "react";

const WriterIcon: React.FC<{ className?: string; id?: string; style?: CSSProperties }> = ({
  className,
  id,
  style,
}) => (
  <svg
    className={className}
    id={id}
    style={style}
    width="18px"
    height="18px"
    version="1.1"
    xmlSpace="preserve"
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 64 64"
    strokeWidth="3"
    stroke="currentColor"
    fill="none">
    <path d="M55.5,23.9V53.5a2,2,0,0,1-2,2h-43a2,2,0,0,1-2-2v-43a2,2,0,0,1,2-2H41.64" />
    <path d="M19.48,38.77l-.64,5.59a.84.84,0,0,0,.92.93l5.56-.64a.87.87,0,0,0,.5-.24L54.9,15.22a1.66,1.66,0,0,0,0-2.35L51.15,9.1a1.67,1.67,0,0,0-2.36,0L19.71,38.28A.83.83,0,0,0,19.48,38.77Z" />
    <line x1="44.87" y1="13.04" x2="50.9" y2="19.24" />
  </svg>
);

export { WriterIcon };

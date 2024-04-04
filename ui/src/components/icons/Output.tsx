import { CSSProperties } from "react";

const OutputIcon: React.FC<{ className?: string; id?: string; style?: CSSProperties }> = ({
  className,
  id,
  style,
}) => (
  <svg
    className={className}
    id={id}
    style={style}
    fill="currentColor"
    width="18px"
    height="18px"
    viewBox="0 0 256 256"
    xmlns="http://www.w3.org/2000/svg">
    <path d="M116,128a3.99875,3.99875,0,0,1-1.34277,2.98926l-72,64a3.99957,3.99957,0,1,1-5.31446-5.97852L105.979,128,37.34277,66.98926a3.99957,3.99957,0,1,1,5.31446-5.97852l72,64A3.99875,3.99875,0,0,1,116,128Zm99.99414,60h-96a4,4,0,0,0,0,8h96a4,4,0,1,0,0-8Z" />
  </svg>
);

export { OutputIcon };

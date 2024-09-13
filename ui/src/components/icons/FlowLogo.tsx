import { CSSProperties } from "react";

const FlowLogo: React.FC<{
  className?: string;
  wrapperClassName?: string;
  id?: string;
  style?: CSSProperties;
}> = ({ className, wrapperClassName, id, style }) => (
  <div id={id} className={wrapperClassName}>
    <svg
      className={`text-primary dark:text-secondary-foreground ${className}`}
      id={id}
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      style={style}
      xmlns="http://www.w3.org/2000/svg">
      <g>
        <rect width="24" height="24" fill="none" />
        <mask
          id="path-1-outside-1_43_3"
          maskUnits="userSpaceOnUse"
          x="4"
          y="-1"
          width="18"
          height="25"
          fill="black">
          <rect fill="white" x="4" y="-1" width="18" height="25" />
          <path d="M16.8591 11.288H7.35513L17.1471 1.56H6.33113V23H5.37113V0.599998H19.4831L9.69113 10.328H16.8591V11.288Z" />
        </mask>
        <path
          d="M16.8591 11.288H7.35513L17.1471 1.56H6.33113V23H5.37113V0.599998H19.4831L9.69113 10.328H16.8591V11.288Z"
          fill="currentColor"
        />
        <path
          d="M16.8591 11.288V12.288H17.8591V11.288H16.8591ZM7.35513 11.288L6.65034 10.5786L4.92967 12.288H7.35513V11.288ZM17.1471 1.56L17.8519 2.26942L19.5726 0.559999H17.1471V1.56ZM6.33113 1.56V0.559999H5.33113V1.56H6.33113ZM6.33113 23V24H7.33113V23H6.33113ZM5.37113 23H4.37113V24H5.37113V23ZM5.37113 0.599998V-0.400002H4.37113V0.599998H5.37113ZM19.4831 0.599998L20.1879 1.30942L21.9086 -0.400002H19.4831V0.599998ZM9.69113 10.328L8.98634 9.61858L7.26567 11.328H9.69113V10.328ZM16.8591 10.328H17.8591V9.328H16.8591V10.328ZM16.8591 10.288H7.35513V12.288H16.8591V10.288ZM8.05991 11.9974L17.8519 2.26942L16.4423 0.850578L6.65034 10.5786L8.05991 11.9974ZM17.1471 0.559999H6.33113V2.56H17.1471V0.559999ZM5.33113 1.56V23H7.33113V1.56H5.33113ZM6.33113 22H5.37113V24H6.33113V22ZM6.37113 23V0.599998H4.37113V23H6.37113ZM5.37113 1.6H19.4831V-0.400002H5.37113V1.6ZM18.7783 -0.109423L8.98634 9.61858L10.3959 11.0374L20.1879 1.30942L18.7783 -0.109423ZM9.69113 11.328H16.8591V9.328H9.69113V11.328ZM15.8591 10.328V11.288H17.8591V10.328H15.8591Z"
          fill="currentColor"
          mask="url(#path-1-outside-1_43_3)"
        />
      </g>
    </svg>
  </div>
);

export { FlowLogo };

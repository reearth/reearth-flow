import { CesiumViewer } from "@flow/components";

type Props = {
  className?: string;
};

const Map: React.FC<Props> = ({ className }) => {
  return (
    <div className={`relative size-full ${className}`}>
      <CesiumViewer />
    </div>
  );
};

export { Map };

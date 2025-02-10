import {
  // Map as MapComponent,
  CesiumViewer,
} from "@flow/components";

type Props = {
  className?: string;
};

const Map: React.FC<Props> = ({ className }) => {
  return (
    <div className={`relative w-full ${className}`}>
      {/* <MapComponent mapMode={mapMode} /> */}
      <CesiumViewer />
    </div>
  );
};

export { Map };

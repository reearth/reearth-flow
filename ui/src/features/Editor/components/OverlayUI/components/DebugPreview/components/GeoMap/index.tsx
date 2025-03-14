import { CesiumViewer } from "@flow/components";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

type Props = {
  className?: string;
  fileContent: string | null;
  fileType: SupportedDataTypes | null;
};

const GeoMap: React.FC<Props> = ({ className, fileContent, fileType }) => {
  return (
    <div className={`relative size-full ${className}`}>
      <CesiumViewer fileContent={fileContent} fileType={fileType} />
    </div>
  );
};

export { GeoMap };

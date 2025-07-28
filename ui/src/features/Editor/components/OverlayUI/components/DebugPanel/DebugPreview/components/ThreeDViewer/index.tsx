import { memo } from "react";

import { CesiumViewer, RenderFallback } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

type Props = {
  className?: string;
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
};

const ThreeDViewer: React.FC<Props> = ({
  className,
  fileContent,
  fileType,
}) => {
  const t = useT();
  return (
    <RenderFallback
      message={t("3D Viewer Could Not Be Loaded. Check if the data is valid.")}
      textSize="sm">
      <div className={`relative size-full ${className}`}>
        <CesiumViewer fileContent={fileContent} fileType={fileType} />
      </div>
    </RenderFallback>
  );
};

export default memo(ThreeDViewer);

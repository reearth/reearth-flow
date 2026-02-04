import { memo } from "react";

import { CesiumViewer, RenderFallback } from "@flow/components";
import { SupportedDataTypes } from "@flow/hooks/useStreamingDebugRunQuery";
import { useT } from "@flow/lib/i18n";

type Props = {
  className?: string;
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  cesiumViewerRef: React.RefObject<any>;
  selectedFeaturedId?: string | null;
  onSelectedFeature?: (featureId: string | null) => void;
};

const ThreeDViewer: React.FC<Props> = ({
  className,
  fileContent,
  fileType,
  cesiumViewerRef,
  selectedFeaturedId,
  onSelectedFeature,
}) => {
  const t = useT();
  return (
    <RenderFallback
      message={t("3D Viewer Could Not Be Loaded. Check if the data is valid.")}
      textSize="sm">
      <div className={`relative size-full ${className}`}>
        <CesiumViewer
          fileContent={fileContent}
          fileType={fileType}
          viewerRef={cesiumViewerRef}
          selectedFeatureId={selectedFeaturedId}
          onSelectedFeature={onSelectedFeature}
        />
      </div>
    </RenderFallback>
  );
};

export default memo(ThreeDViewer);

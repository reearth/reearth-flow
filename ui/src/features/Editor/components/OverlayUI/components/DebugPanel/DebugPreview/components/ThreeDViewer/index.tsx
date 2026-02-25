import { BoundingSphere } from "cesium";
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
  detailsOverlayOpen: boolean;
  onSelectedFeature?: (featureId: string | null) => void;
  onShowFeatureDetailsOverlay: (value: boolean) => void;
  setCityGmlBoundingSphere: (value: BoundingSphere | null) => void;
};

const ThreeDViewer: React.FC<Props> = ({
  className,
  fileContent,
  fileType,
  cesiumViewerRef,
  selectedFeaturedId,
  detailsOverlayOpen,
  onSelectedFeature,
  onShowFeatureDetailsOverlay,
  setCityGmlBoundingSphere,
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
          detailsOverlayOpen={detailsOverlayOpen}
          onSelectedFeature={onSelectedFeature}
          onShowFeatureDetailsOverlay={onShowFeatureDetailsOverlay}
          setCityGmlBoundingSphere={setCityGmlBoundingSphere}
        />
      </div>
    </RenderFallback>
  );
};

export default memo(ThreeDViewer);

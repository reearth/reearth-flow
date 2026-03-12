import { BoundingSphere } from "cesium";
import { memo } from "react";

import { CesiumViewer, RenderFallback } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  className?: string;
  fileContent: any | null;
  visualizerType: "2d-map" | "3d-map";
  cesiumViewerRef: React.RefObject<any>;
  selectedFeaturedId?: string | null;
  detailsOverlayOpen: boolean;
  showSelectedFeatureOnly: boolean;
  onSelectedFeature?: (featureId: string | null) => void;
  onShowFeatureDetailsOverlay: (value: boolean) => void;
  setCityGmlBoundingSphere: (value: BoundingSphere | null) => void;
};

const GeoViewer: React.FC<Props> = ({
  className,
  fileContent,
  visualizerType,
  cesiumViewerRef,
  selectedFeaturedId,
  detailsOverlayOpen,
  showSelectedFeatureOnly,
  onSelectedFeature,
  onShowFeatureDetailsOverlay,
  setCityGmlBoundingSphere,
}) => {
  const t = useT();
  return (
    <RenderFallback
      message={t("Geo Viewer Could Not Be Loaded. Check if the data is valid.")}
      textSize="sm">
      <div className={`relative size-full ${className}`}>
        <CesiumViewer
          fileContent={fileContent}
          visualizerType={visualizerType}
          viewerRef={cesiumViewerRef}
          selectedFeatureId={selectedFeaturedId}
          detailsOverlayOpen={detailsOverlayOpen}
          showSelectedFeatureOnly={showSelectedFeatureOnly}
          onSelectedFeature={onSelectedFeature}
          onShowFeatureDetailsOverlay={onShowFeatureDetailsOverlay}
          setCityGmlBoundingSphere={setCityGmlBoundingSphere}
        />
      </div>
    </RenderFallback>
  );
};

export default memo(GeoViewer);

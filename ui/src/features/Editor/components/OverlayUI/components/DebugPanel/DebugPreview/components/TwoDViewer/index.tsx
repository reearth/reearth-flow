import { memo } from "react";

import { RenderFallback } from "@flow/components";
import { MapLibre } from "@flow/components/visualizations/MapLibre";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

type Props = {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  enableClustering?: boolean;
  selectedFeature: any;
  shouldFlyToFeature?: boolean;
  fitDataToBounds?: boolean;
  onSelectedFeature: (value: any) => void;
  onFitDataToBoundsChange?: (value: boolean) => void;
  onShouldFlyToFeatureChange?: (value: boolean) => void;
};

const TwoDViewer: React.FC<Props> = ({
  fileContent,
  fileType,
  enableClustering,
  selectedFeature,
  shouldFlyToFeature,
  fitDataToBounds,
  onSelectedFeature,
  onFitDataToBoundsChange,
  onShouldFlyToFeatureChange,
}) => {
  const t = useT();
  return (
    <RenderFallback
      message={t("2D Viewer Could Not Be Loaded. Check if the data is valid.")}
      textSize="sm">
      <MapLibre
        fileContent={fileContent}
        fileType={fileType}
        enableClustering={enableClustering}
        selectedFeature={selectedFeature}
        shouldFlyToFeature={shouldFlyToFeature}
        fitDataToBounds={fitDataToBounds}
        onSelectedFeature={onSelectedFeature}
        onFitDataToBoundsChange={onFitDataToBoundsChange}
        onShouldFlyToFeatureChange={onShouldFlyToFeatureChange}
      />
    </RenderFallback>
  );
};

export default memo(TwoDViewer);

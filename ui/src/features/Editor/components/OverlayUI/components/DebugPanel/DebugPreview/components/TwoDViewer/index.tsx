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
  onSelectedFeature: (value: any) => void;
  shouldFlyToFeature?: boolean;
  fitDataToBounds?: boolean;
  onFitDataToBoundsChange?: (value: boolean) => void;
};

const TwoDViewer: React.FC<Props> = ({
  fileContent,
  fileType,
  enableClustering,
  selectedFeature,
  onSelectedFeature,
  shouldFlyToFeature,
  fitDataToBounds,
  onFitDataToBoundsChange,
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
        onSelectedFeature={onSelectedFeature}
        shouldFlyToFeature={shouldFlyToFeature}
        fitDataToBounds={fitDataToBounds}
        onFitDataToBoundsChange={onFitDataToBoundsChange}
      />
    </RenderFallback>
  );
};

export default memo(TwoDViewer);

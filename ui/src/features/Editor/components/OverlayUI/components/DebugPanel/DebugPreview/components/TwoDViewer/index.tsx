import { memo } from "react";

import { RenderFallback } from "@flow/components";
import { MapLibre } from "@flow/components/visualizations/MapLibre";
import { SupportedDataTypes } from "@flow/hooks/useStreamingDebugRunQuery";
import { useT } from "@flow/lib/i18n";

type Props = {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  enableClustering?: boolean;
  convertedSelectedFeature?: any;
  mapRef: React.RefObject<maplibregl.Map | null>;
  onSelectedFeature: (value: any) => void;
  onMapLoad: (onCenter?: boolean) => void;
  onFlyToSelectedFeature?: (selectedFeature: any) => void;
};

const TwoDViewer: React.FC<Props> = ({
  fileContent,
  fileType,
  enableClustering,
  convertedSelectedFeature,
  mapRef,
  onMapLoad,
  onSelectedFeature,
  onFlyToSelectedFeature,
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
        convertedSelectedFeature={convertedSelectedFeature}
        mapRef={mapRef}
        onMapLoad={onMapLoad}
        onSelectedFeature={onSelectedFeature}
        onFlyToSelectedFeature={onFlyToSelectedFeature}
      />
    </RenderFallback>
  );
};

export default memo(TwoDViewer);

import { memo, useEffect, useState } from "react";
import { GeoJsonDataSource, useCesium } from "resium";

type Props = {
  geoJsonData: any | null;
  selectedFeatureId?: string | null;
};

const GeoJsonData: React.FC<Props> = ({ geoJsonData, selectedFeatureId }) => {
  const { viewer } = useCesium();
  const [dataSourceKey, setDataSourceKey] = useState(0);
  useEffect(() => {
    setDataSourceKey(dataSourceKey + 1);
  }, [geoJsonData]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    geoJsonData && (
      <GeoJsonDataSource
        key={dataSourceKey}
        data={geoJsonData}
        onLoad={(geoJsonDataSource) => {
          if (viewer) {
            if (dataSourceKey === 1 && !selectedFeatureId) {
              viewer.zoomTo(geoJsonDataSource.entities);
            }

            if (selectedFeatureId) {
              const entity =
                geoJsonDataSource.entities.getById(selectedFeatureId);
              if (entity) {
                viewer.zoomTo(entity);
              }
            }
          }
        }}
      />
    )
  );
};

export default memo(GeoJsonData);

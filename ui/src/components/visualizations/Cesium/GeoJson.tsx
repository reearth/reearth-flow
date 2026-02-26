import { Entity } from "cesium";
import { memo, useEffect, useRef, useState } from "react";
import { GeoJsonDataSource, useCesium } from "resium";

type Props = {
  geoJsonData: any | null;
  selectedFeatureId?: string | null;
  showSelectedFeatureOnly: boolean;
};

const GeoJsonData: React.FC<Props> = ({
  geoJsonData,
  selectedFeatureId,
  showSelectedFeatureOnly,
}) => {
  const { viewer } = useCesium();
  const [dataSourceKey, setDataSourceKey] = useState(0);
  const dataSourceRef = useRef<any>(null);

  useEffect(() => {
    setDataSourceKey(dataSourceKey + 1);
  }, [geoJsonData]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const dataSource = dataSourceRef.current;
    if (!dataSource) return;

    dataSource.entities.values.forEach((entity: Entity) => {
      if (showSelectedFeatureOnly) {
        const props = entity.properties?.getValue?.();
        const id = props?._originalId ?? entity.id;
        entity.show = id === selectedFeatureId;
      } else {
        entity.show = true;
      }
    });
  }, [showSelectedFeatureOnly, selectedFeatureId]);

  return (
    geoJsonData && (
      <GeoJsonDataSource
        key={dataSourceKey}
        data={geoJsonData}
        onLoad={(geoJsonDataSource) => {
          dataSourceRef.current = geoJsonDataSource;
          // Ensure entity visibility respects the current selection state on load

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
          geoJsonDataSource.entities.values.forEach((entity: Entity) => {
            if (showSelectedFeatureOnly) {
              const props = entity.properties?.getValue?.();
              const id = props?._originalId ?? entity.id;
              entity.show = id === selectedFeatureId;
            } else {
              entity.show = true;
            }
          });
        }}
      />
    )
  );
};

export default memo(GeoJsonData);

import { memo, useEffect, useState } from "react";
import { GeoJsonDataSource, useCesium } from "resium";

type Props = {
  geoJsonData: any | null;
};

const GeoJsonData: React.FC<Props> = ({ geoJsonData }) => {
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
          // Enhance entities with better ID mapping for fly-to functionality
          geoJsonDataSource.entities.values.forEach((entity) => {
            // Try to preserve original feature ID in entity properties
            if (entity.properties) {
              const currentProps = entity.properties.getValue();
              if (currentProps && geoJsonData?.features) {
                // Find the matching original feature by comparing properties
                const matchingFeature = geoJsonData.features.find(
                  (feature: any) => {
                    // Compare by feature ID first
                    if (
                      feature.id !== undefined &&
                      currentProps.id === feature.id
                    ) {
                      return true;
                    }
                    // Compare by properties content
                    if (feature.properties) {
                      const propKeys = Object.keys(feature.properties);
                      return propKeys.every(
                        (key) => currentProps[key] === feature.properties[key],
                      );
                    }
                    return false;
                  },
                );

                if (matchingFeature && matchingFeature.id !== undefined) {
                  // Store original feature ID for easier lookup
                  entity.properties.addProperty(
                    "_originalId",
                    matchingFeature.id,
                  );
                }
              }
            }

            // TODO: Add more styling options
            // if (entity.polygon) {
            //   entity.polygon.material = new ColorMaterialProperty(
            //     Color.BLACK.withAlpha(0.5),
            //   );
            //   entity.polygon.outlineColor = new ColorMaterialProperty(
            //     Color.BLACK,
            //   );
            // }
          });

          if (viewer) {
            viewer.zoomTo(geoJsonDataSource.entities);
          }
        }}
      />
    )
  );
};

export default memo(GeoJsonData);

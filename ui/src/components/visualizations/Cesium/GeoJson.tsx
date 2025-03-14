import { memo, useEffect, useState } from "react";
import { GeoJsonDataSource, useCesium } from "resium";

type Props = {
  geoJsonData: string | null;
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
          // geoJsonDataSource.entities.values.forEach((entity) => {
          //   // TODO: Add more styling options
          //   if (entity.polygon) {
          //     entity.polygon.material = new ColorMaterialProperty(
          //       Color.BLACK.withAlpha(0.5),
          //     );
          //     entity.polygon.outlineColor = new ColorMaterialProperty(
          //       Color.BLACK,
          //     );
          //   }
          // });

          if (viewer) {
            viewer.zoomTo(geoJsonDataSource.entities);
          }
        }}
      />
    )
  );
};

export default memo(GeoJsonData);

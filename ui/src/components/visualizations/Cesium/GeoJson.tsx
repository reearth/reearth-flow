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
          if (viewer && dataSourceKey === 1) {
            viewer.zoomTo(geoJsonDataSource.entities);
          }
        }}
      />
    )
  );
};

export default memo(GeoJsonData);

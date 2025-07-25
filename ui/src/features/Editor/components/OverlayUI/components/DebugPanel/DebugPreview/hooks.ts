import bbox from "@turf/bbox";
import { LngLatBounds } from "maplibre-gl";
import { useCallback, useMemo } from "react";

export default ({
  mapRef,
  selectedOutputData,
}: {
  mapRef: any;
  selectedOutputData: any;
}) => {
  const dataBounds = useMemo(() => {
    if (!selectedOutputData) return null;

    try {
      const [minLng, minLat, maxLng, maxLat] = bbox(selectedOutputData);
      return new LngLatBounds([minLng, minLat], [maxLng, maxLat]);
    } catch (err) {
      console.error("Error computing bbox:", err);
      return null;
    }
  }, [selectedOutputData]);

  const handleMapLoad = useCallback(
    (onCenter?: boolean) => {
      if (mapRef.current && dataBounds) {
        mapRef.current.fitBounds(dataBounds, {
          padding: 40,
          duration: onCenter ? 500 : 0,
          maxZoom: 16,
        });
      }
    },
    [mapRef, dataBounds],
  );
  return {
    handleMapLoad,
  };
};

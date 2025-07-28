import bbox from "@turf/bbox";
import { LngLatBounds } from "maplibre-gl";
import { useCallback } from "react";

export default ({
  mapRef,
  selectedOutputData,
}: {
  mapRef: any;
  selectedOutputData: any;
}) => {
  const handleMapLoad = useCallback(
    (onCenter?: boolean) => {
      if (mapRef.current && selectedOutputData) {
        try {
          const [minLng, minLat, maxLng, maxLat] = bbox(selectedOutputData);
          const dataBounds = new LngLatBounds(
            [minLng, minLat],
            [maxLng, maxLat],
          );

          mapRef.current.fitBounds(dataBounds, {
            padding: 40,
            duration: onCenter ? 500 : 0,
            maxZoom: 16,
          });
        } catch (err) {
          console.error("Error computing bbox:", err);
        }
      }
    },
    [mapRef, selectedOutputData],
  );

  return {
    handleMapLoad,
  };
};

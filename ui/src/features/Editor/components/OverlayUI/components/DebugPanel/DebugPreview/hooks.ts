import bbox from "@turf/bbox";
import { BoundingSphere } from "cesium";
import { LngLatBounds } from "maplibre-gl";
import { RefObject, useCallback, useEffect, useState } from "react";

import { ThreeJSViewerRef } from "@flow/components/visualizations/ThreeJS";

export default ({
  mapRef,
  cesiumViewerRef,
  threeJSViewerRef,
  selectedOutputData,
  convertedSelectedFeature,
}: {
  mapRef: any;
  cesiumViewerRef: RefObject<any>;
  threeJSViewerRef: RefObject<ThreeJSViewerRef | null>;
  selectedOutputData: any;
  convertedSelectedFeature: any;
}) => {
  const [cityGmlBoundingSphere, setCityGmlBoundingSphere] =
    useState<BoundingSphere | null>(null);
  const [showSelectedFeatureOnly, setShowSelectedFeatureOnly] = useState(false);
  const handleMapLoad = useCallback(
    (onCenter?: boolean) => {
      if (mapRef.current && selectedOutputData) {
        try {
          if (convertedSelectedFeature) {
            const [minLng, minLat, maxLng, maxLat] = bbox(
              convertedSelectedFeature,
            );
            const featureBounds = new LngLatBounds(
              [minLng, minLat],
              [maxLng, maxLat],
            );

            mapRef.current.fitBounds(featureBounds, {
              padding: 100,
              duration: onCenter ? 500 : 0,
              maxZoom: 16,
            });
            return;
          }

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
    [mapRef, selectedOutputData, convertedSelectedFeature],
  );

  const handleThreeDViewerReset = useCallback(() => {
    if (cesiumViewerRef?.current?.cesiumElement) {
      const cesiumViewer = cesiumViewerRef.current.cesiumElement;

      if (cesiumViewer) {
        // Handle cityGml primitives
        if (cityGmlBoundingSphere) {
          cesiumViewer.camera.flyToBoundingSphere(cityGmlBoundingSphere, {
            duration: 1.5,
          });
        }
      }
    }
  }, [cesiumViewerRef, cityGmlBoundingSphere]);

  const handleThreeJsReset = useCallback(() => {
    threeJSViewerRef.current?.resetCamera();
  }, [threeJSViewerRef]);

  const handleShowSelectedFeatureOnly = useCallback(() => {
    setShowSelectedFeatureOnly((prev) => !prev);
  }, []);

  useEffect(() => {
    if (!convertedSelectedFeature) {
      setShowSelectedFeatureOnly(false);
    }
  }, [convertedSelectedFeature]);

  return {
    showSelectedFeatureOnly,
    handleMapLoad,
    handleThreeDViewerReset,
    handleThreeJsReset,
    handleShowSelectedFeatureOnly,
    setCityGmlBoundingSphere,
  };
};

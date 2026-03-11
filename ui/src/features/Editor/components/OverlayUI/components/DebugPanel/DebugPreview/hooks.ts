import { BoundingSphere } from "cesium";
import { RefObject, useCallback, useEffect, useState } from "react";

import { ThreeJSViewerRef } from "@flow/components/visualizations/ThreeJS";

export default ({
  cesiumViewerRef,
  threeJSViewerRef,
  convertedSelectedFeature,
}: {
  cesiumViewerRef: RefObject<any>;
  threeJSViewerRef: RefObject<ThreeJSViewerRef | null>;
  selectedOutputData: any;
  convertedSelectedFeature: any;
}) => {
  const [cityGmlBoundingSphere, setCityGmlBoundingSphere] =
    useState<BoundingSphere | null>(null);
  const [showSelectedFeatureOnly, setShowSelectedFeatureOnly] = useState(false);

  const handleGeoViewerReset = useCallback(() => {
    if (cesiumViewerRef?.current?.cesiumElement) {
      const cesiumViewer = cesiumViewerRef.current.cesiumElement;
      if (cesiumViewer) {
        // Handle cityGml primitives
        if (cityGmlBoundingSphere) {
          cesiumViewer.camera.flyToBoundingSphere(cityGmlBoundingSphere, {
            duration: 1.5,
          });
        } else if (cesiumViewer.dataSources.length > 0) {
          // Zoom to all entities
          const allEntities: any[] = [];
          for (let i = 0; i < cesiumViewer.dataSources.length; i++) {
            allEntities.push(
              ...cesiumViewer.dataSources.get(i).entities.values,
            );
          }
          if (allEntities.length > 0) {
            cesiumViewer.zoomTo(allEntities);
          }
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
    handleGeoViewerReset,
    handleThreeJsReset,
    handleShowSelectedFeatureOnly,
    setCityGmlBoundingSphere,
  };
};

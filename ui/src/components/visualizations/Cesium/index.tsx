import {
  CesiumTerrainProvider,
  EllipsoidTerrainProvider,
  UrlTemplateImageryProvider,
  BoundingSphere,
  defined,
  SceneMode,
  ScreenSpaceEventType,
} from "cesium";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  ImageryLayer,
  ScreenSpaceEvent,
  ScreenSpaceEventHandler,
  useCesium,
  Viewer,
  ViewerProps,
} from "resium";

import { config } from "@flow/config";
import useDoubleClick from "@flow/hooks/useDoubleClick";

import CityGmlData from "./CityGmlData";
import GeoJsonData from "./GeoJson";

const REEARTH_TERRAIN_URL =
  "https://terrain.reearth.land/cesium-mesh/ellipsoid";
const ESRI_WORLD_IMAGERY_URL =
  "https://services.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}";

const defaultCesiumProps: Partial<ViewerProps> = {
  timeline: false,
  baseLayerPicker: false,
  fullscreenButton: false,
  sceneModePicker: false,
  infoBox: false,
  homeButton: false,
  geocoder: false,
  animation: false,
  requestRenderMode: true,
  maximumRenderTimeChange: Infinity,
  navigationHelpButton: false,
  baseLayer: false,
};

const TerrainController: React.FC<{ show3DTerrain: boolean }> = ({
  show3DTerrain,
}) => {
  const { viewer } = useCesium();

  useEffect(() => {
    if (!viewer || viewer.isDestroyed()) return;
    let cancelled = false;

    if (show3DTerrain) {
      CesiumTerrainProvider.fromUrl(REEARTH_TERRAIN_URL, {
        requestVertexNormals: true,
        requestWaterMask: false,
      })
        .then((terrainProvider) => {
          if (cancelled || viewer.isDestroyed()) return;
          viewer.terrainProvider = terrainProvider;
          viewer.scene.requestRender();
        })
        .catch((e) => {
          console.error("Failed to load Re:Earth terrain:", e);
          if (cancelled || viewer.isDestroyed()) return;
          viewer.terrainProvider = new EllipsoidTerrainProvider();
          viewer.scene.requestRender();
        });
    } else {
      viewer.terrainProvider = new EllipsoidTerrainProvider();
      viewer.scene.requestRender();
    }

    return () => {
      cancelled = true;
    };
  }, [viewer, show3DTerrain]);

  return null;
};

type Props = {
  fileContent: any | null;
  visualizerType: "2d-map" | "3d-map";
  viewerRef?: React.RefObject<any>;
  selectedFeatureId?: string | null;
  detailsOverlayOpen: boolean;
  showSelectedFeatureOnly: boolean;
  onSelectedFeature?: (featureId: string | null) => void;
  onShowFeatureDetailsOverlay: (value: boolean) => void;
  setCityGmlBoundingSphere: (value: BoundingSphere | null) => void;
};

const CesiumViewer: React.FC<Props> = ({
  fileContent,
  visualizerType,
  viewerRef,
  selectedFeatureId,
  detailsOverlayOpen,
  showSelectedFeatureOnly,
  onSelectedFeature,
  onShowFeatureDetailsOverlay,
  setCityGmlBoundingSphere,
}) => {
  const [isLoaded, setIsLoaded] = useState(false);

  useEffect(() => {
    if (isLoaded) return;
    setIsLoaded(true);
  }, [isLoaded]);

  const pickOriginalId = useCallback(
    (movement: any) => {
      if (!viewerRef?.current?.cesiumElement) return null;
      const cesiumViewer = viewerRef.current.cesiumElement;
      if (cesiumViewer.isDestroyed()) return null;
      const pickedObject = cesiumViewer.scene.pick(movement.position);
      if (!defined(pickedObject) || !defined(pickedObject.id)) return null;
      const pickedId = pickedObject.id;
      // Support both Entity (PropertyBag) and Primitive (plain object) ids
      return (
        pickedId?.properties?.getValue?.()?.["_originalId"] ??
        pickedId?._originalId ??
        null
      );
    },
    [viewerRef],
  );

  const onSingleClick = useCallback(
    (movement?: any) => {
      if (!onSelectedFeature || !movement) return;
      try {
        const originalId = pickOriginalId(movement);
        if (originalId) {
          onSelectedFeature(originalId);
        } else {
          onSelectedFeature(null);
          onShowFeatureDetailsOverlay(false);
        }
      } catch (e) {
        console.error("Cesium viewer error:", e);
      }
    },
    [onSelectedFeature, onShowFeatureDetailsOverlay, pickOriginalId],
  );

  const onDoubleClick = useCallback(
    (movement?: any) => {
      if (!onSelectedFeature || !movement) return;
      try {
        const originalId = pickOriginalId(movement);
        if (originalId) {
          onSelectedFeature(originalId);
          onShowFeatureDetailsOverlay(true);
        } else {
          onSelectedFeature(null);
          onShowFeatureDetailsOverlay(false);
        }
      } catch (e) {
        console.error("Cesium viewer error:", e);
      }
    },
    [onSelectedFeature, onShowFeatureDetailsOverlay, pickOriginalId],
  );

  const [handleSingleClick, handleDoubleClick] = useDoubleClick(
    onSingleClick,
    onDoubleClick,
  );

  const baseImageryProvider = useMemo(() => {
    const tileServerBaseUrl = config().tileServerBaseUrl;
    if (tileServerBaseUrl) {
      return new UrlTemplateImageryProvider({
        url: `${tileServerBaseUrl.replace(/\/$/, "")}/imagery/{z}/{x}/{y}.webp`,
        minimumLevel: 0,
        maximumLevel: 19,
        credit: "© Google",
      });
    } else {
      return new UrlTemplateImageryProvider({
        url: ESRI_WORLD_IMAGERY_URL,
        maximumLevel: 19,
      });
    }
  }, []);

  // Separate features by geometry type
  const { geoJsonData, cityGmlData } = useMemo(() => {
    const features = fileContent?.features || [];

    const geoJsonFeatures: any[] = [];
    const cityGmlFeatures: any[] = [];

    for (const feature of features) {
      if (feature?.geometry?.type === "CityGmlGeometry") {
        cityGmlFeatures.push(feature);
      } else {
        geoJsonFeatures.push(feature);
      }
    }

    return {
      geoJsonData:
        geoJsonFeatures.length > 0
          ? { type: "FeatureCollection" as const, features: geoJsonFeatures }
          : null,
      cityGmlData:
        cityGmlFeatures.length > 0
          ? { type: "FeatureCollection" as const, features: cityGmlFeatures }
          : null,
    };
  }, [fileContent]);

  return (
    <Viewer
      ref={viewerRef}
      sceneMode={
        visualizerType === "2d-map" ? SceneMode.SCENE2D : SceneMode.SCENE3D
      }
      full
      {...defaultCesiumProps}>
      <ImageryLayer imageryProvider={baseImageryProvider} />
      <TerrainController
        show3DTerrain={visualizerType === "3d-map" && !cityGmlData}
      />
      {onSelectedFeature && (
        <ScreenSpaceEventHandler>
          <ScreenSpaceEvent
            action={handleSingleClick}
            type={ScreenSpaceEventType.LEFT_CLICK}
          />
          <ScreenSpaceEvent
            action={handleDoubleClick}
            type={ScreenSpaceEventType.LEFT_DOUBLE_CLICK}
          />
        </ScreenSpaceEventHandler>
      )}

      {isLoaded && (
        <>
          {/* Standard GeoJSON features */}
          {geoJsonData && (
            <GeoJsonData
              geoJsonData={geoJsonData}
              selectedFeatureId={selectedFeatureId}
              showSelectedFeatureOnly={showSelectedFeatureOnly}
              clampToGround={visualizerType === "3d-map"}
            />
          )}

          {/* CityGML features */}
          {cityGmlData && (
            <CityGmlData
              cityGmlData={cityGmlData}
              setCityGmlBoundingSphere={setCityGmlBoundingSphere}
              selectedFeatureId={selectedFeatureId}
              detailsOverlayOpen={detailsOverlayOpen}
              showSelectedFeatureOnly={showSelectedFeatureOnly}
            />
          )}
        </>
      )}
    </Viewer>
  );
};
export { CesiumViewer };

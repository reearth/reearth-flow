import {
  Color,
  ColorMaterialProperty,
  ConstantProperty,
  Entity,
  GeoJsonDataSource,
} from "cesium";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  useCesium,
  GeoJsonDataSource as ResiumGeoJsonDataSource,
} from "resium";

type Props = {
  geoJsonData: any | null;
  selectedFeatureId?: string | null;
  showSelectedFeatureOnly: boolean;
};

type EntityRecord = {
  entity: Entity;
  origPolygonMaterial?: any;
  origPolygonOutlineColor?: any;
  origPointColor?: any;
  origPolylineMaterial?: any;
  origBillboardColor?: any;
};

function sanitizeCoords(coords: any): any {
  if (!Array.isArray(coords)) return null;

  if (typeof coords[0] === "number") {
    const [lon, lat] = coords;
    if (typeof lon === "number" && typeof lat === "number") {
      return coords;
    }
    return null;
  }

  const cleaned = coords
    .map((c) => sanitizeCoords(c))
    .filter((c) => c !== null);

  return cleaned.length ? cleaned : null;
}

function sanitizeGeoJson(geoJson: any) {
  if (!geoJson) return undefined;

  if (geoJson.type === "FeatureCollection") {
    geoJson.features = geoJson.features
      .map((feature: any) => {
        const cleaned = sanitizeCoords(feature?.geometry?.coordinates);
        if (!cleaned) return null;

        return {
          ...feature,
          geometry: {
            ...feature.geometry,
            coordinates: cleaned,
          },
        };
      })
      .filter(Boolean);
  }

  return geoJson;
}

const HIGHLIGHT_COLOR = Color.CYAN.withAlpha(0.7);
const HIGHLIGHT_FILL = Color.CYAN.withAlpha(0.4);

const GeoJsonData: React.FC<Props> = ({
  geoJsonData,
  selectedFeatureId,
  showSelectedFeatureOnly,
}) => {
  const { viewer } = useCesium();
  const latestViewerRef = useRef(viewer);
  latestViewerRef.current = viewer;

  const [dataSourceKey, setDataSourceKey] = useState(0);
  const dataSourceRef = useRef<GeoJsonDataSource | null>(null);
  const prevSelectedRef = useRef<string | null>(null);
  const featureMapRef = useRef<Map<string, EntityRecord[]>>(new Map());
  const hasEverLoadedRef = useRef(false);
  const isMountedRef = useRef(true);
  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);
  // Keep prop values in refs so callbacks that use them don't need them in deps,
  // preventing handleLoad from being recreated (and re-called by Resium) on every selection.
  const selectedFeatureIdRef = useRef(selectedFeatureId ?? null);
  selectedFeatureIdRef.current = selectedFeatureId ?? null;
  const showSelectedFeatureOnlyRef = useRef(showSelectedFeatureOnly);
  showSelectedFeatureOnlyRef.current = showSelectedFeatureOnly;

  const sanitizedData = useMemo(() => {
    if (!geoJsonData) return null;
    return sanitizeGeoJson(structuredClone(geoJsonData));
  }, [geoJsonData]);

  useEffect(() => {
    setDataSourceKey((k) => k + 1);
  }, [sanitizedData]);

  const updateVisibility = useCallback(() => {
    featureMapRef.current.forEach((records) => {
      records.forEach(({ entity }) => {
        if (!showSelectedFeatureOnlyRef.current) {
          entity.show = true;
          return;
        }

        const props = entity.properties?.getValue?.();
        const entityId = props?._originalId ?? entity.id;

        entity.show = entityId === selectedFeatureIdRef.current;
      });
    });
  }, []); // stable — reads props via refs

  const handleDeselectFeature = useCallback(
    (prevId: string) => {
      const records = featureMapRef.current.get(prevId);
      if (!records || !viewer) return;

      records.forEach(
        ({
          entity,
          origPolygonMaterial,
          origPolygonOutlineColor,
          origPolylineMaterial,
          origBillboardColor,
        }) => {
          if (entity.polygon) {
            if (origPolygonMaterial !== undefined)
              entity.polygon.material = origPolygonMaterial;
            if (origPolygonOutlineColor !== undefined)
              entity.polygon.outlineColor = origPolygonOutlineColor;
          }

          if (entity.polyline && origPolylineMaterial !== undefined) {
            entity.polyline.material = origPolylineMaterial;
          }

          if (entity.billboard) {
            entity.billboard.color = origBillboardColor;
          }
        },
      );
      viewer?.scene.requestRender();
    },
    [viewer],
  );

  const handleHighlightSelectedFeature = useCallback(
    (featureId: string) => {
      const records = featureMapRef.current.get(featureId);
      if (!records || !viewer) return;

      records.forEach(({ entity }) => {
        if (entity.polygon) {
          entity.polygon.material = new ColorMaterialProperty(HIGHLIGHT_FILL);
          entity.polygon.outlineColor = new ConstantProperty(HIGHLIGHT_COLOR);
        }
        if (entity.billboard) {
          entity.billboard.color = new ConstantProperty(HIGHLIGHT_COLOR);
        }
        if (entity.polyline) {
          entity.polyline.material = new ColorMaterialProperty(HIGHLIGHT_COLOR);
        }
      });
      viewer?.scene.requestRender();
    },
    [viewer],
  );

  const flyTo = useCallback(async () => {
    const ds = dataSourceRef.current;
    if (!ds || !viewer || viewer.isDestroyed() || !isMountedRef.current) return;

    try {
      const selectedId = selectedFeatureIdRef.current;
      const records = selectedId
        ? featureMapRef.current.get(selectedId)
        : undefined;
      const entity = records?.[0]?.entity;

      if (entity) {
        await viewer.flyTo(entity, { duration: 0 });
      } else if (!hasEverLoadedRef.current || selectedId) {
        await viewer.zoomTo(ds);
      }
    } catch {
      console.log("FlyTo failed, likely due to invalid entity state");
    } finally {
      if (!viewer.isDestroyed() && isMountedRef.current) {
        viewer.scene.requestRender();
      }
    }
  }, [viewer]);

  const handleLoad = useCallback(
    async (ds: GeoJsonDataSource) => {
      dataSourceRef.current = ds;
      prevSelectedRef.current = null;
      featureMapRef.current = new Map();

      ds.entities.values.forEach((entity: Entity) => {
        const props = entity.properties?.getValue?.();
        const id = props?._originalId ?? entity.id;

        const record: EntityRecord = {
          entity,
          origPolygonMaterial: entity.polygon?.material,
          origPolygonOutlineColor: entity.polygon?.outlineColor,
          origPolylineMaterial: entity.polyline?.material,
          origBillboardColor: entity.billboard?.color,
        };

        const existing = featureMapRef.current.get(id) ?? [];
        existing.push(record);
        featureMapRef.current.set(id, existing);
      });

      updateVisibility();
      await flyTo();

      // Bail if unmounted during flyTo (e.g. switched to 3D data mid-flight).
      if (!isMountedRef.current) return;

      hasEverLoadedRef.current = true;

      // Two renders: DataSourceDisplay may not commit entities until the frame after requestRender().
      const v = latestViewerRef.current;
      if (v && !v.isDestroyed()) {
        v.scene.requestRender();
        const renderAgain = () => {
          v.scene.postRender.removeEventListener(renderAgain);
          if (!v.isDestroyed() && isMountedRef.current) v.scene.requestRender();
        };
        v.scene.postRender.addEventListener(renderAgain);
      }
    },
    [updateVisibility, flyTo],
  );

  useEffect(() => {
    const prevId = prevSelectedRef.current;
    const currentId = selectedFeatureId ?? null;
    prevSelectedRef.current = currentId;

    if (prevId && prevId !== currentId) {
      handleDeselectFeature(prevId);
    }

    if (currentId) {
      handleHighlightSelectedFeature(currentId);
    }
  }, [
    selectedFeatureId,
    handleHighlightSelectedFeature,
    handleDeselectFeature,
  ]);

  useEffect(() => {
    if (!viewer || viewer.isDestroyed()) return;
    viewer.scene.requestRender();
  }, [dataSourceKey, viewer]);

  // Re-run visibility whenever the filtering props change (updateVisibility itself is stable)
  useEffect(() => {
    updateVisibility();
    viewer?.scene.requestRender();
  }, [selectedFeatureId, showSelectedFeatureOnly, updateVisibility, viewer]);

  if (!sanitizedData) return null;

  return (
    <ResiumGeoJsonDataSource
      key={dataSourceKey}
      data={sanitizedData}
      onLoad={handleLoad}
    />
  );
};

export default memo(GeoJsonData);

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
  if (!geoJson) return geoJson;

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
  const [dataSourceKey, setDataSourceKey] = useState(0);
  const dataSourceRef = useRef<GeoJsonDataSource | null>(null);
  const prevSelectedRef = useRef<string | null>(null);
  const featureMapRef = useRef<Map<string, EntityRecord[]>>(new Map());

  const sanitizedData = useMemo(() => {
    if (!geoJsonData) return null;
    return sanitizeGeoJson(structuredClone(geoJsonData));
  }, [geoJsonData]);

  useEffect(() => {
    setDataSourceKey((k) => k + 1);
  }, [sanitizedData]);

  const updateVisibility = useCallback(() => {
    const ds = dataSourceRef.current;
    if (!ds) return;
    ds.entities.values.forEach((entity: Entity) => {
      if (!showSelectedFeatureOnly) {
        entity.show = true;
        return;
      }

      const props = entity.properties?.getValue?.();
      const id = props?._originalId ?? entity.id;

      entity.show = id === selectedFeatureId;
    });
  }, [selectedFeatureId, showSelectedFeatureOnly]);

  const handleDeselectFeature = useCallback((prevId: string) => {
    const records = featureMapRef.current.get(prevId);
    if (!records) return;

    records.forEach(
      ({
        entity,
        origPolygonMaterial,
        origPolygonOutlineColor,
        origPointColor,
        origPolylineMaterial,
        origBillboardColor,
      }) => {
        if (entity.polygon) {
          if (origPolygonMaterial !== undefined)
            entity.polygon.material = origPolygonMaterial;
          if (origPolygonOutlineColor !== undefined)
            entity.polygon.outlineColor = origPolygonOutlineColor;
        }
        if (entity.point && origPointColor !== undefined) {
          entity.point.color = origPointColor;
        }
        if (entity.polyline && origPolylineMaterial !== undefined) {
          entity.polyline.material = origPolylineMaterial;
        }
        if (entity.billboard && origBillboardColor !== undefined) {
          entity.billboard.color = origBillboardColor;
        }
      },
    );
  }, []);

  const handleHighlightSelectedFeature = useCallback((featureId: string) => {
    const records = featureMapRef.current.get(featureId);
    if (!records) return;

    records.forEach(({ entity }) => {
      if (entity.polygon) {
        entity.polygon.material = new ColorMaterialProperty(HIGHLIGHT_FILL);
        entity.polygon.outlineColor = new ConstantProperty(HIGHLIGHT_COLOR);
      }
      if (entity.point) {
        entity.point.color = new ConstantProperty(HIGHLIGHT_COLOR);
      }
      if (entity.billboard) {
        entity.billboard.color = new ConstantProperty(HIGHLIGHT_COLOR);
      }
      if (entity.polyline) {
        entity.polyline.material = new ColorMaterialProperty(HIGHLIGHT_COLOR);
      }
    });
  }, []);

  const flyTo = useCallback(async () => {
    const ds = dataSourceRef.current;
    if (!ds || !viewer) return;

    if (!selectedFeatureId) {
      await viewer.zoomTo(ds);
      return;
    }

    const records = featureMapRef.current.get(selectedFeatureId);
    const entity = records?.[0]?.entity;

    if (entity) {
      await viewer.flyTo(entity, {
        duration: 1.2,
      });
    }
  }, [viewer, selectedFeatureId]);

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
          origPointColor: entity.point?.color,
          origPolylineMaterial: entity.polyline?.material,
          origBillboardColor: entity.billboard?.color,
        };

        const existing = featureMapRef.current.get(id) ?? [];
        existing.push(record);
        featureMapRef.current.set(id, existing);
      });

      updateVisibility();
      await flyTo();
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
    updateVisibility();
  });

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

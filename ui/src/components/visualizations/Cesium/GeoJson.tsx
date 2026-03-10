import { Entity, GeoJsonDataSource } from "cesium";
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

const GeoJsonData: React.FC<Props> = ({
  geoJsonData,
  selectedFeatureId,
  showSelectedFeatureOnly,
}) => {
  const { viewer } = useCesium();
  const [dataSourceKey, setDataSourceKey] = useState(0);
  const dataSourceRef = useRef<GeoJsonDataSource | null>(null);

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

  const flyTo = useCallback(async () => {
    const ds = dataSourceRef.current;
    if (!ds || !viewer) return;

    if (!selectedFeatureId) {
      await viewer.zoomTo(ds);
      return;
    }

    const entity = ds.entities.values.find((e: Entity) => {
      const props = e.properties?.getValue?.();
      const id = props?._originalId ?? e.id;
      return id === selectedFeatureId;
    });

    if (entity) {
      await viewer.flyTo(entity, {
        duration: 1.2,
      });
    }
  }, [viewer, selectedFeatureId]);

  useEffect(() => {
    updateVisibility();
  }, [updateVisibility]);

  const handleLoad = useCallback(
    async (ds: GeoJsonDataSource) => {
      dataSourceRef.current = ds;
      updateVisibility();
      await flyTo();
    },
    [updateVisibility, flyTo],
  );

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

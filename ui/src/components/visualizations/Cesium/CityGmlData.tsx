import {
  BoundingSphere,
  GroundPrimitive,
  Primitive,
  ShowGeometryInstanceAttribute,
} from "cesium";
import { memo, useCallback, useEffect, useRef } from "react";
import { useCesium } from "resium";

import { buildLodPrimitiveCollection } from "./utils/buildLodPrimitives";
import {
  CITYGML_3D_TYPES,
  convertFeatureCollectionToPrimitives,
  type FeatureInstanceData,
} from "./utils/cityGmlGeometryToPrimitives";
import { useLodWorker } from "./utils/useLodWorker";

type CityGmlFeature = {
  id?: string;
  type: "Feature";
  properties: Record<string, any>;
  geometry: {
    type: "CityGmlGeometry";
    [key: string]: any;
  };
};

type Props = {
  cityGmlData: {
    type: "FeatureCollection";
    features: CityGmlFeature[];
  } | null;
  selectedFeatureId?: string | null;
  detailsOverlayOpen: boolean;
  showSelectedFeatureOnly: boolean;
  setCityGmlBoundingSphere: (value: BoundingSphere | null) => void;
};

const WAIT_FOR_PRIMITIVE_TIMEOUT_MS = 10_000;

const CityGmlData: React.FC<Props> = ({
  cityGmlData,
  selectedFeatureId,
  detailsOverlayOpen,
  showSelectedFeatureOnly,
  setCityGmlBoundingSphere,
}) => {
  const { viewer } = useCesium();
  const absolutePrimitiveRef = useRef<Primitive | null>(null);
  const groundPrimitiveRef = useRef<GroundPrimitive | null>(null);
  const featureMapRef = useRef<Map<string, FeatureInstanceData>>(new Map());
  const prevSelectedRef = useRef<string | null>(null);
  const { buildLodGeometry, cancelPending } = useLodWorker();

  const waitForPrimitive = useCallback(
    (primitive: Primitive | null, callback: () => void) => {
      if (!primitive || !viewer) return;
      if ((primitive as any).ready) {
        callback();
        return;
      }
      const startTime = Date.now();
      const remove = viewer.scene.postRender.addEventListener(() => {
        if ((primitive as any).ready) {
          remove();
          callback();
          return;
        }
        if (Date.now() - startTime > WAIT_FOR_PRIMITIVE_TIMEOUT_MS) {
          remove();
        }
      });
    },
    [viewer],
  );

  const revertLod = useCallback(
    (entry: FeatureInstanceData) => {
      if (entry.lodPrimitiveCollection && viewer) {
        viewer.scene.primitives.remove(entry.lodPrimitiveCollection);
        entry.lodPrimitiveCollection = null;
      }
      waitForPrimitive(absolutePrimitiveRef.current, () => {
        entry.absoluteInstanceIds.forEach((id) => {
          const attrs =
            absolutePrimitiveRef.current?.getGeometryInstanceAttributes(id);
          if (attrs) attrs.show = ShowGeometryInstanceAttribute.toValue(true);
        });
      });
    },
    [viewer, waitForPrimitive],
  );

  const upgradeLod = useCallback(
    async (entry: FeatureInstanceData) => {
      if (entry.lodPrimitiveCollection) return;
      const typeConfig = CITYGML_3D_TYPES.find((cfg) =>
        cfg.detect(entry.feature.properties),
      );

      const featureId =
        entry.feature.properties?._originalId || entry.feature.id || "";

      // Send heavy work to the web worker
      const resultPromise = buildLodGeometry(entry.feature, typeConfig);
      if (!resultPromise) return;

      let result;
      try {
        result = await resultPromise;
      } catch {
        // Worker error or cancellation — nothing to do
        return;
      }

      // After await: re-check if this entry is still relevant (user may have switched)
      if (!viewer || entry.lodPrimitiveCollection) return;

      const lodPrimitive = buildLodPrimitiveCollection(result, featureId);
      if (!lodPrimitive) return;

      // Final check before adding to scene
      if (entry.lodPrimitiveCollection) return;

      viewer.scene.primitives.add(lodPrimitive);
      entry.lodPrimitiveCollection = lodPrimitive;

      waitForPrimitive(absolutePrimitiveRef.current, () => {
        entry.absoluteInstanceIds.forEach((id) => {
          const attrs =
            absolutePrimitiveRef.current?.getGeometryInstanceAttributes(id);
          if (attrs) attrs.show = ShowGeometryInstanceAttribute.toValue(false);
        });
      });
    },
    [viewer, waitForPrimitive, buildLodGeometry],
  );

  // Process CityGML data and create primitives (only on data change)
  useEffect(() => {
    if (!cityGmlData || !viewer) return;

    // Cancel any in-flight worker requests
    cancelPending();

    // Remove any active LOD primitives first
    featureMapRef.current.forEach((entry) => {
      if (entry.lodPrimitiveCollection) {
        viewer.scene.primitives.remove(entry.lodPrimitiveCollection);
        entry.lodPrimitiveCollection = null;
      }
    });

    if (absolutePrimitiveRef.current) {
      viewer.scene.primitives.remove(absolutePrimitiveRef.current);
      absolutePrimitiveRef.current = null;
    }
    if (groundPrimitiveRef.current) {
      viewer.scene.primitives.remove(groundPrimitiveRef.current);
      groundPrimitiveRef.current = null;
    }

    featureMapRef.current.clear();
    prevSelectedRef.current = null;

    const { absolutePrimitive, groundPrimitive, featureMap, boundingSphere } =
      convertFeatureCollectionToPrimitives(cityGmlData.features);

    absolutePrimitiveRef.current = absolutePrimitive;
    groundPrimitiveRef.current = groundPrimitive;
    featureMapRef.current = featureMap;

    if (absolutePrimitive) viewer.scene.primitives.add(absolutePrimitive);
    if (groundPrimitive) viewer.scene.primitives.add(groundPrimitive);

    if (boundingSphere) {
      viewer.camera.flyToBoundingSphere(boundingSphere, { duration: 1.5 });
      setCityGmlBoundingSphere(boundingSphere);
    }
  }, [cityGmlData, viewer, cancelPending, setCityGmlBoundingSphere]);

  // Handle LOD upgrade/revert when selectedFeatureId or detailsOverlayOpen changes
  useEffect(() => {
    if (!viewer) return;

    const prevId = prevSelectedRef.current;
    const currentId = selectedFeatureId ?? null;
    prevSelectedRef.current = currentId;

    // Revert previously selected feature back to LOD1
    if (prevId && prevId !== currentId) {
      cancelPending();
      const prevEntry = featureMapRef.current.get(prevId);
      if (prevEntry) revertLod(prevEntry);
    }

    if (currentId && !detailsOverlayOpen) {
      cancelPending();
      const entry = featureMapRef.current.get(currentId);
      if (entry) revertLod(entry);
    }

    // Upgrade newly selected feature to highest available LOD
    if (currentId && detailsOverlayOpen) {
      const entry = featureMapRef.current.get(currentId);
      if (entry) upgradeLod(entry);
    }
  }, [
    selectedFeatureId,
    viewer,
    detailsOverlayOpen,
    waitForPrimitive,
    revertLod,
    upgradeLod,
    cancelPending,
  ]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      cancelPending();
      if (viewer) {
        featureMapRef.current.forEach((entry) => {
          if (entry.lodPrimitiveCollection) {
            viewer.scene.primitives.remove(entry.lodPrimitiveCollection);
          }
        });
        viewer.scene.primitives.remove(absolutePrimitiveRef.current);
        viewer.scene.primitives.remove(groundPrimitiveRef.current);
      }
    };
  }, [viewer, cancelPending]);

  useEffect(() => {
    if (viewer && absolutePrimitiveRef.current) {
      absolutePrimitiveRef.current.show = !showSelectedFeatureOnly;
    }
  }, [viewer, showSelectedFeatureOnly]);

  return null;
};

export default memo(CityGmlData);

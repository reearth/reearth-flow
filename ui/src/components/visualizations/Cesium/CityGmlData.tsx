import {
  GroundPrimitive,
  Primitive,
  ShowGeometryInstanceAttribute,
} from "cesium";
import { memo, useEffect, useRef } from "react";
import { useCesium } from "resium";

import {
  CITYGML_3D_TYPES,
  convertFeatureCollectionToPrimitives,
  createLodUpgradePrimitiveCollection,
  type FeatureInstanceData,
} from "./utils/cityGmlGeometryToPrimitives";

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
};

const CityGmlData: React.FC<Props> = ({
  cityGmlData,
  selectedFeatureId,
  detailsOverlayOpen,
}) => {
  const { viewer } = useCesium();
  const absolutePrimitiveRef = useRef<Primitive | null>(null);
  const groundPrimitiveRef = useRef<GroundPrimitive | null>(null);
  const featureMapRef = useRef<Map<string, FeatureInstanceData>>(new Map());
  const prevSelectedRef = useRef<string | null>(null);

  // Process CityGML data and create primitives (only on data change)
  useEffect(() => {
    if (!cityGmlData || !viewer) return;

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
    }
  }, [cityGmlData, viewer]);

  // Handle LOD upgrade/revert when selectedFeatureId or detailsOverlayOpen changes
  useEffect(() => {
    if (!viewer) return;

    const prevId = prevSelectedRef.current;
    const currentId = selectedFeatureId ?? null;
    prevSelectedRef.current = currentId;

    function waitForPrimitive(
      primitive: Primitive | null,
      callback: () => void,
    ) {
      if (!primitive || !viewer) return;
      if ((primitive as any).ready) {
        callback();
        return;
      }
      const remove = viewer.scene.postRender.addEventListener(() => {
        if (!(primitive as any).ready) return;
        remove();
        callback();
      });
    }

    const revertLod = (entry: FeatureInstanceData) => {
      if (entry.lodPrimitiveCollection) {
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
    };

    const upgradeLod = (entry: FeatureInstanceData) => {
      if (entry.lodPrimitiveCollection) return;
      const typeConfig = CITYGML_3D_TYPES.find((cfg) =>
        cfg.detect(entry.feature.properties),
      );
      const lodPrimitive = createLodUpgradePrimitiveCollection(
        entry.feature,
        typeConfig,
      );

      console.log("PRIMITIVE COLLECTION CREATED", lodPrimitive);
      if (!lodPrimitive) return;
      viewer.scene.primitives.add(lodPrimitive);
      entry.lodPrimitiveCollection = lodPrimitive;
      waitForPrimitive(absolutePrimitiveRef.current, () => {
        entry.absoluteInstanceIds.forEach((id) => {
          const attrs =
            absolutePrimitiveRef.current?.getGeometryInstanceAttributes(id);
          if (attrs) attrs.show = ShowGeometryInstanceAttribute.toValue(false);
        });
      });
    };

    // Revert previously selected feature back to LOD1
    if (prevId && prevId !== currentId) {
      const prevEntry = featureMapRef.current.get(prevId);
      if (prevEntry) revertLod(prevEntry);
    }

    // Upgrade newly selected feature to highest available LOD
    if (currentId && detailsOverlayOpen) {
      const entry = featureMapRef.current.get(currentId);
      if (entry) upgradeLod(entry);
    }
  }, [selectedFeatureId, viewer, detailsOverlayOpen]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
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
  }, [viewer]);

  return null;
};

export default memo(CityGmlData);

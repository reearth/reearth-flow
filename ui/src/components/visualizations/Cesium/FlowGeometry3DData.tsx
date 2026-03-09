import {
  BoundingSphere,
  Primitive,
  ShowGeometryInstanceAttribute,
} from "cesium";
import { memo, useEffect, useRef } from "react";
import { useCesium } from "resium";

import {
  convertFlowGeometry3DCollectionToPrimitives,
  type FlowGeometry3DFeature,
  type FlowGeometry3DFeatureInstanceData,
} from "./utils/flowGeometry3DToPrimitives";

type Props = {
  flowGeometry3DData: {
    type: "FeatureCollection";
    features: FlowGeometry3DFeature[];
  } | null;
  selectedFeatureId?: string | null;
  showSelectedFeatureOnly: boolean;
  setBoundingSphere: (value: BoundingSphere | null) => void;
};

const FlowGeometry3DData: React.FC<Props> = ({
  flowGeometry3DData,
  selectedFeatureId,
  showSelectedFeatureOnly,
  setBoundingSphere,
}) => {
  const { viewer } = useCesium();
  const meshPrimitiveRef = useRef<Primitive | null>(null);
  const linePrimitiveRef = useRef<Primitive | null>(null);
  const featureMapRef = useRef<Map<string, FlowGeometry3DFeatureInstanceData>>(
    new Map(),
  );

  // Build primitives from data
  useEffect(() => {
    if (!flowGeometry3DData || !viewer) return;

    if (meshPrimitiveRef.current) {
      viewer.scene.primitives.remove(meshPrimitiveRef.current);
      meshPrimitiveRef.current = null;
    }
    if (linePrimitiveRef.current) {
      viewer.scene.primitives.remove(linePrimitiveRef.current);
      linePrimitiveRef.current = null;
    }
    featureMapRef.current.clear();

    const { meshPrimitive, linePrimitive, featureMap, boundingSphere } =
      convertFlowGeometry3DCollectionToPrimitives(flowGeometry3DData.features);

    meshPrimitiveRef.current = meshPrimitive;
    linePrimitiveRef.current = linePrimitive;
    featureMapRef.current = featureMap;

    if (meshPrimitive) viewer.scene.primitives.add(meshPrimitive);
    if (linePrimitive) viewer.scene.primitives.add(linePrimitive);

    if (boundingSphere) {
      viewer.camera.flyToBoundingSphere(boundingSphere, { duration: 1.5 });
      setBoundingSphere(boundingSphere);
    }
  }, [flowGeometry3DData, viewer, setBoundingSphere]);

  // Handle show/hide based on selection
  useEffect(() => {
    if (!viewer) return;

    const applyVisibility = (primitive: Primitive | null) => {
      if (!primitive || !(primitive as any).ready) return;
      featureMapRef.current.forEach((entry, id) => {
        const isSelected = id === selectedFeatureId;
        const shouldShow = !showSelectedFeatureOnly || isSelected;

        entry.meshInstanceIds.forEach((instanceId) => {
          const attrs =
            meshPrimitiveRef.current?.getGeometryInstanceAttributes(instanceId);
          if (attrs)
            attrs.show = ShowGeometryInstanceAttribute.toValue(shouldShow);
        });
        entry.lineInstanceIds.forEach((instanceId) => {
          const attrs =
            linePrimitiveRef.current?.getGeometryInstanceAttributes(instanceId);
          if (attrs)
            attrs.show = ShowGeometryInstanceAttribute.toValue(shouldShow);
        });
      });
    };

    // Primitives are synchronous (asynchronous: false), so they're ready immediately
    applyVisibility(meshPrimitiveRef.current);
    applyVisibility(linePrimitiveRef.current);
  }, [viewer, showSelectedFeatureOnly, selectedFeatureId]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (viewer) {
        if (meshPrimitiveRef.current)
          viewer.scene.primitives.remove(meshPrimitiveRef.current);
        if (linePrimitiveRef.current)
          viewer.scene.primitives.remove(linePrimitiveRef.current);
      }
    };
  }, [viewer]);

  return null;
};

export default memo(FlowGeometry3DData);

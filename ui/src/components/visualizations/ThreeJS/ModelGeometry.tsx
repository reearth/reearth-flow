import { useEffect, useMemo, useRef } from "react";
import * as THREE from "three";
import { useThree } from "@react-three/fiber";

type Props = {
  features: any[];
  resetTrigger?: number;
};

// Extract material color from feature properties
function extractMaterialColor(feature: any): THREE.Color {
  const props = feature?.properties;

  // Try to get material properties from OBJ data
  const materialProps = props?.materialProperties;
  if (materialProps && typeof materialProps === "object") {
    // Get the first material (features usually have one material)
    const materialNames = props?.materials;
    const firstMaterial = Array.isArray(materialNames)
      ? materialNames[0]
      : null;

    if (firstMaterial && materialProps[firstMaterial]) {
      const material = materialProps[firstMaterial];

      // Use diffuse color (most common for surface color)
      if (material.diffuse && Array.isArray(material.diffuse)) {
        const [r, g, b] = material.diffuse;
        return new THREE.Color(r, g, b);
      }
    }
  }

  // Default neutral gray
  return new THREE.Color(0x9ca3af);
}

// Helper to build BufferGeometry from polygon data with vertex colors
function buildGeometryFromPolygons(features: any[]): THREE.BufferGeometry {
  const vertices: number[] = [];
  const colors: number[] = [];
  const indices: number[] = [];
  let vertexOffset = 0;

  features.forEach((feature) => {
    const geometry = feature?.geometry;
    if (!geometry) return;

    // Extract material color for this feature
    const color = extractMaterialColor(feature);

    // Handle GeoJSON Polygon
    if (geometry.type === "Polygon" && geometry.coordinates) {
      const rings = geometry.coordinates;
      if (rings.length === 0) return;

      const exterior = rings[0]; // First ring is exterior
      if (exterior.length < 3) return; // Need at least 3 points

      // Add vertices (remove last point if it's a closed ring)
      const points = exterior.slice(0, -1); // Remove closing point
      const startIndex = vertexOffset;

      points.forEach((coord: number[]) => {
        vertices.push(coord[0], coord[1], coord[2] || 0);
        colors.push(color.r, color.g, color.b);
        vertexOffset++;
      });

      // Simple triangulation for convex polygons (fan triangulation)
      // For a polygon with n vertices: (0,1,2), (0,2,3), (0,3,4), ...
      for (let i = 1; i < points.length - 1; i++) {
        indices.push(startIndex, startIndex + i, startIndex + i + 1);
      }
    }

    // Handle GeoJSON MultiPolygon
    if (geometry.type === "MultiPolygon" && geometry.coordinates) {
      geometry.coordinates.forEach((polygon: number[][][]) => {
        if (polygon.length === 0) return;

        const exterior = polygon[0];
        if (exterior.length < 3) return;

        const points = exterior.slice(0, -1);
        const startIndex = vertexOffset;

        points.forEach((coord: number[]) => {
          vertices.push(coord[0], coord[1], coord[2] || 0);
          colors.push(color.r, color.g, color.b);
          vertexOffset++;
        });

        for (let i = 1; i < points.length - 1; i++) {
          indices.push(startIndex, startIndex + i, startIndex + i + 1);
        }
      });
    }
  });

  const bufferGeometry = new THREE.BufferGeometry();
  bufferGeometry.setAttribute(
    "position",
    new THREE.Float32BufferAttribute(vertices, 3),
  );
  bufferGeometry.setAttribute(
    "color",
    new THREE.Float32BufferAttribute(colors, 3),
  );
  bufferGeometry.setIndex(indices);
  bufferGeometry.computeVertexNormals();

  return bufferGeometry;
}

const ModelGeometry: React.FC<Props> = ({ features, resetTrigger }) => {
  const meshRef = useRef<THREE.Mesh>(null);
  const { camera, controls } = useThree();

  // Build geometry from features
  const geometry = useMemo(() => {
    if (!features || features.length === 0) return null;
    return buildGeometryFromPolygons(features);
  }, [features]);

  // Auto-frame the model on load or reset
  useEffect(() => {
    if (geometry && meshRef.current && controls) {
      // Compute bounding box
      geometry.computeBoundingBox();
      const box = geometry.boundingBox;

      if (box) {
        const center = new THREE.Vector3();
        box.getCenter(center);

        const size = new THREE.Vector3();
        box.getSize(size);

        // Position camera to view entire model
        const maxDim = Math.max(size.x, size.y, size.z);
        const distance = maxDim * 2;

        camera.position.set(
          center.x + distance,
          center.y + distance,
          center.z + distance,
        );
        camera.lookAt(center);
        camera.updateProjectionMatrix();

        // Update OrbitControls target to center of model
        // @ts-expect-error - OrbitControls has target property
        if (controls.target) {
          // @ts-expect-error - OrbitControls has target property
          controls.target.copy(center);
          // @ts-expect-error - OrbitControls has update method
          controls.update();
        }
      }
    }
  }, [geometry, camera, controls, resetTrigger]);

  // Cleanup geometry on unmount
  useEffect(() => {
    return () => {
      geometry?.dispose();
    };
  }, [geometry]);

  if (!geometry) {
    return null;
  }

  return (
    <mesh ref={meshRef} geometry={geometry}>
      <meshStandardMaterial
        vertexColors
        side={THREE.DoubleSide}
        flatShading={false}
        metalness={0.1}
        roughness={0.8}
      />
    </mesh>
  );
};

export default ModelGeometry;

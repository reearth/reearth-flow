import { useEffect, useMemo, useRef } from "react";
import * as THREE from "three";
import { useThree } from "@react-three/fiber";

type Props = {
  features: any[];
};

// Helper to build BufferGeometry from polygon data
function buildGeometryFromPolygons(features: any[]): THREE.BufferGeometry {
  const vertices: number[] = [];
  const indices: number[] = [];
  let vertexOffset = 0;

  features.forEach((feature) => {
    const geometry = feature?.geometry;
    if (!geometry) return;

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
    new THREE.Float32BufferAttribute(vertices, 3)
  );
  bufferGeometry.setIndex(indices);
  bufferGeometry.computeVertexNormals();

  return bufferGeometry;
}

const ModelGeometry: React.FC<Props> = ({ features }) => {
  const meshRef = useRef<THREE.Mesh>(null);
  const { camera } = useThree();
  const hasFramed = useRef(false);

  // Build geometry from features
  const geometry = useMemo(() => {
    if (!features || features.length === 0) return null;
    return buildGeometryFromPolygons(features);
  }, [features]);

  // Auto-frame the model on load
  useEffect(() => {
    if (geometry && meshRef.current && !hasFramed.current) {
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
          center.z + distance
        );
        camera.lookAt(center);
        camera.updateProjectionMatrix();

        hasFramed.current = true;
      }
    }
  }, [geometry, camera]);

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
        color="#9ca3af"
        side={THREE.DoubleSide}
        flatShading={false}
        metalness={0.1}
        roughness={0.8}
      />
    </mesh>
  );
};

export default ModelGeometry;

import { OrbitControls, Grid, PerspectiveCamera } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { forwardRef, Suspense, useImperativeHandle, useState } from "react";

import ModelGeometry from "./ModelGeometry";

type Props = {
  fileContent: any;
};

export type ThreeJSViewerRef = {
  resetCamera: () => void;
};

const ThreeJSViewer = forwardRef<ThreeJSViewerRef, Props>(
  ({ fileContent }, ref) => {
    const [resetTrigger, setResetTrigger] = useState(0);

    // Expose reset function via ref
    useImperativeHandle(
      ref,
      () => ({
        resetCamera: () => {
          setResetTrigger((prev) => prev + 1);
        },
      }),
      [],
    );

    return (
      <div className="h-full w-full bg-background">
        <Canvas>
          <Suspense fallback={null}>
            {/* Camera */}
            <PerspectiveCamera makeDefault position={[2, 2, 2]} />

            {/* Lights */}
            <ambientLight intensity={0.5} />
            <directionalLight position={[10, 10, 5]} intensity={1} />
            <hemisphereLight
              color="#ffffff"
              groundColor="#444444"
              intensity={0.6}
            />

            {/* Grid helper */}
            <Grid
              args={[10, 10]}
              cellSize={0.5}
              cellThickness={0.5}
              cellColor="#6b7280"
              sectionSize={1}
              sectionThickness={1}
              sectionColor="#9ca3af"
              fadeDistance={25}
              fadeStrength={1}
              followCamera={false}
              infiniteGrid
            />

            {/* Model geometry - resetTrigger forces re-frame */}
            <ModelGeometry
              features={fileContent?.features || []}
              resetTrigger={resetTrigger}
            />

            {/* Controls */}
            <OrbitControls makeDefault />
          </Suspense>
        </Canvas>
      </div>
    );
  },
);

ThreeJSViewer.displayName = "ThreeJSViewer";

export default ThreeJSViewer;

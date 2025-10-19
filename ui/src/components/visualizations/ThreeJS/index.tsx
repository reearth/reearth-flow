import { Canvas } from "@react-three/fiber";
import { OrbitControls, Grid, PerspectiveCamera } from "@react-three/drei";
import { memo, Suspense } from "react";

import ModelGeometry from "./ModelGeometry";

type Props = {
  fileContent: any;
};

const ThreeJSViewer: React.FC<Props> = ({ fileContent }) => {
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

          {/* Model geometry */}
          <ModelGeometry features={fileContent?.features || []} />

          {/* Controls */}
          <OrbitControls makeDefault />
        </Suspense>
      </Canvas>
    </div>
  );
};

export default memo(ThreeJSViewer);

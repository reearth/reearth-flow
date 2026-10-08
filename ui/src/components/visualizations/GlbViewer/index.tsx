import { Grid, OrbitControls, PerspectiveCamera } from "@react-three/drei";
import { Canvas, useLoader, useThree } from "@react-three/fiber";
import { memo, Suspense, useLayoutEffect, useMemo, useState } from "react";
import { PerspectiveCamera as ThreePerspectiveCamera } from "three";
import dracoDecoderWasmUrl from "three/examples/jsm/libs/draco/gltf/draco_decoder.wasm?url";
import dracoWasmWrapperUrl from "three/examples/jsm/libs/draco/gltf/draco_wasm_wrapper.js?url";
import { DRACOLoader } from "three/examples/jsm/loaders/DRACOLoader.js";
import type { GLTF } from "three/examples/jsm/loaders/GLTFLoader.js";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";

import type { Framing } from "./frameModel";
import { frameModel } from "./frameModel";
import { placeUpright } from "./placeUpright";
import { showBackFaces } from "./showBackFaces";
import { tintDefaultMaterials } from "./tintDefaultMaterials";

// Rendered views compress their meshes with Draco. The decoder is served from
// this app's own build, out of the copy three ships, rather than fetched from a
// public CDN at runtime.
const dracoLoader = new DRACOLoader().setDecoderPath({
  js: dracoWasmWrapperUrl,
  wasm: dracoDecoderWasmUrl,
});

const extendLoader = (loader: GLTFLoader) => {
  loader.setDRACOLoader(dracoLoader);
};

type Props = {
  /** A glb, such as the entry point of a rendered row. */
  url: string;
};

const Model: React.FC<Props & { onFramed: (framing: Framing) => void }> = ({
  url,
  onFramed,
}) => {
  const gltf: GLTF = useLoader(GLTFLoader, url, extendLoader);
  const model = useMemo(
    () => showBackFaces(tintDefaultMaterials(placeUpright(gltf.scene))),
    [gltf.scene],
  );
  const getState = useThree((state) => state.get);

  // Framed once, when the model arrives, and then left to the controls alone:
  // anything else that keeps moving the camera fights the user for it. The
  // controls take their target and zoom limit from the framing as props.
  useLayoutEffect(() => {
    // Read here rather than held from render: the camera belongs to the
    // scene, and this is where it is set.
    const { camera } = getState();
    if (!(camera instanceof ThreePerspectiveCamera)) return;
    const framing = frameModel(model, camera.fov);
    camera.position.copy(framing.position);
    camera.near = framing.near;
    camera.far = framing.far;
    camera.updateProjectionMatrix();
    onFramed(framing);
  }, [model, getState, onFramed]);

  return <primitive object={model} />;
};

/**
 * Shows a single glb, stood upright at the origin when it is placed on the
 * globe, and framed by the camera once it has loaded. The back of any face is
 * drawn in `BACK_FACE_COLOR` rather than hidden.
 */
const GlbViewer: React.FC<Props> = ({ url }) => {
  const [framing, setFraming] = useState<Framing>();

  return (
    <div className="h-full w-full bg-background">
      <Canvas>
        <PerspectiveCamera makeDefault position={[20, 20, 20]} />
        <ambientLight intensity={0.5} />
        <directionalLight position={[10, 10, 5]} intensity={1} />
        <hemisphereLight
          color="#ffffff"
          groundColor="#444444"
          intensity={0.6}
        />
        {/* Just below y = 0, where the model's floor rests: two surfaces in
          one plane flicker as the camera moves. */}
        <Grid
          position={[0, -0.05, 0]}
          cellSize={1}
          cellThickness={0.5}
          cellColor="#6b7280"
          sectionSize={10}
          sectionThickness={1}
          sectionColor="#9ca3af"
          fadeDistance={500}
          fadeStrength={1}
          followCamera={false}
          infiniteGrid
        />
        <Suspense fallback={null}>
          <Model url={url} onFramed={setFraming} />
        </Suspense>
        <OrbitControls
          makeDefault
          target={framing?.target}
          maxDistance={framing?.maxDistance ?? Infinity}
        />
      </Canvas>
    </div>
  );
};

export { BACK_FACE_COLOR } from "./showBackFaces";

export default memo(GlbViewer);

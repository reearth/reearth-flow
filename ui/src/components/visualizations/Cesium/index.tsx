import { useEffect, useState } from "react";
import { Viewer } from "resium";

import { CesiumContents } from "./Contents";

const dummyCredit = document.createElement("div");

const defaultCesiumProps = {
  // timeline: false,
  // homeButton: false,
  // baseLayerPicker: false,
  // sceneModePicker: false,
  fullscreenButton: false,
  geocoder: false,
  animation: false,
  navigationHelpButton: false,
  creditContainer: dummyCredit,
};

const CesiumViewer: React.FC = () => {
  const [isLoaded, setIsLoaded] = useState(false);

  useEffect(() => {
    if (isLoaded) return;
    setIsLoaded(true);
  }, [isLoaded]);

  return (
    <Viewer full {...defaultCesiumProps}>
      <CesiumContents isLoaded={isLoaded} />
    </Viewer>
  );
};
export { CesiumViewer };

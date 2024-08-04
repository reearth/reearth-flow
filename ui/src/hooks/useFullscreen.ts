import { useCallback, useState } from "react";

import {
  checkIsFullscreen,
  closeFullscreen,
  openFullscreen,
} from "@flow/utils";

export default () => {
  const [isFullscreen, setIsFullscreen] = useState(false);

  const handleFullscreenToggle = useCallback(() => {
    const isFullscreen = checkIsFullscreen();
    if (isFullscreen) {
      closeFullscreen();
    } else {
      openFullscreen();
    }
    setIsFullscreen(!isFullscreen);
  }, []);

  return {
    isFullscreen,
    handleFullscreenToggle,
  };
};

import { useEffect, useState } from "react";

export default (duration: number) => {
  const [running, setIsRunning] = useState(true);

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      setIsRunning(false);
    }, duration);

    return () => clearTimeout(timeoutId);
  }, []); // eslint-disable-line

  return {
    running,
  };
};

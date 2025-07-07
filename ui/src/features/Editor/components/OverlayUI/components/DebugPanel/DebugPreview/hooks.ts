import { useState } from "react";

export default () => {
  const [minimized, setMinimized] = useState(false);

  const handleTabChange = () => {
    if (minimized) {
      setMinimized(false);
    }
  };

  return {
    handleTabChange,
  };
};

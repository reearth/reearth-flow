import { useState } from "react";

import { useDoubleClick as useDoubleClickSetter } from "@flow/hooks";

export default () => {
  const [softSelect, setSoftSelect] = useState<string[] | undefined>(undefined);
  const [hardSelect, setHardSelect] = useState<string | undefined>(undefined);

  const handleSingleClick = () => (id: string) =>
    setSoftSelect(s => {
      if (s?.includes(id)) return s?.filter(i => i !== id);
      return [...(s ?? []), id];
    });

  const handleDoubleClick = () => (id: string) => setHardSelect(id);

  const [useSingleClick, useDoubleClick] = useDoubleClickSetter(
    handleSingleClick,
    handleDoubleClick,
  );

  return {
    softSelect,
    hardSelect,
    useSingleClick,
    useDoubleClick,
  };
};

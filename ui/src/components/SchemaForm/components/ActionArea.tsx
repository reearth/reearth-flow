import { ArrowUDownLeftIcon, PencilLineIcon } from "@phosphor-icons/react";
import { RefObject, useCallback } from "react";

import { IconButton } from "@flow/components/buttons";
import { useT } from "@flow/lib/i18n";

type Props = {
  value?: any;
  defaultValue?: RefObject<any>;
  onEditorOpen?: () => void;
  onReset?: () => void;
};

const ActionArea: React.FC<Props> = ({
  value,
  defaultValue,
  onEditorOpen,
  onReset,
}) => {
  const t = useT();

  const handleEditorOpen = useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      e.preventDefault();
      onEditorOpen?.();
    },
    [onEditorOpen],
  );

  return (
    <div className="flex items-center justify-end">
      {onEditorOpen && (
        <IconButton
          icon={<PencilLineIcon />}
          tooltipText={t("Open Editor")}
          onClick={handleEditorOpen}
          disabled={!onEditorOpen}
        />
      )}
      {onReset && (
        <IconButton
          icon={<ArrowUDownLeftIcon />}
          disabled={value === defaultValue?.current}
          tooltipText={t("Reset to Default")}
          aria-label={`Reset value to default: ${defaultValue?.current}`}
          onClick={onReset}
        />
      )}
    </div>
  );
};

export default ActionArea;

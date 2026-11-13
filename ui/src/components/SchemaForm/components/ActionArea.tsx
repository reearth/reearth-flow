import { ArrowUDownLeftIcon, CodeBlockIcon } from "@phosphor-icons/react";
import type { RefObject } from "react";
import { useCallback } from "react";

import { IconButton } from "@flow/components/buttons";
import { useT } from "@flow/lib/i18n";

type Props = {
  value?: any;
  defaultValue?: RefObject<any>;
  readonly?: boolean;
  onPythonEditorOpen?: () => void;
  onReset?: () => void;
};

const ActionArea: React.FC<Props> = ({
  value,
  defaultValue,
  readonly,
  onPythonEditorOpen,
  onReset,
}) => {
  const t = useT();

  const handlePythonEditorOpen = useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      e.preventDefault();
      onPythonEditorOpen?.();
    },
    [onPythonEditorOpen],
  );

  return (
    <div className="flex items-center justify-end">
      {onPythonEditorOpen && (
        <IconButton
          icon={<CodeBlockIcon />}
          tooltipText={t("Open Python Editor")}
          onClick={handlePythonEditorOpen}
          disabled={!onPythonEditorOpen || readonly}
        />
      )}
      {onReset && (
        <IconButton
          icon={<ArrowUDownLeftIcon />}
          disabled={value === defaultValue?.current || readonly}
          tooltipText={t("Reset to Default")}
          aria-label={`Reset value to default: ${defaultValue?.current}`}
          onClick={onReset}
        />
      )}
    </div>
  );
};

export default ActionArea;

import { WrenchIcon } from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import { Button, Input } from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { type ExpressionType } from "./ExpressionTypePicker";
import SubExpressionBuilderModal from "./SubExpressionBuilderModal";

type Props = {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
  label?: string;
  allowedExpressionTypes?: ExpressionType[]; // Restrict which builders can be used
};

const ExpressionInput: React.FC<Props> = ({
  value,
  onChange,
  placeholder,
  disabled,
  className,
  label,
  allowedExpressionTypes,
}) => {
  const t = useT();
  const [isBuilderOpen, setIsBuilderOpen] = useState(false);

  const handleOpenBuilder = useCallback(() => {
    if (!disabled) {
      setIsBuilderOpen(true);
    }
  }, [disabled]);

  const handleBuilderClose = useCallback(() => {
    setIsBuilderOpen(false);
  }, []);

  const handleExpressionBuilt = useCallback(
    (expression: string) => {
      onChange(expression);
      setIsBuilderOpen(false);
    },
    [onChange],
  );

  return (
    <>
      <div className="flex gap-2">
        <div className="flex-1">
          <Input
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder={placeholder}
            disabled={disabled}
            className={className}
          />
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={handleOpenBuilder}
          disabled={disabled}
          className="flex-shrink-0 px-2"
          title={t("Build Expression")}>
          <WrenchIcon className="h-4 w-4" />
        </Button>
      </div>

      {isBuilderOpen && (
        <SubExpressionBuilderModal
          isOpen={isBuilderOpen}
          onClose={handleBuilderClose}
          onExpressionBuilt={handleExpressionBuilt}
          allowedExpressionTypes={allowedExpressionTypes}
          initialValue={value}
          fieldLabel={label}
        />
      )}
    </>
  );
};

export default ExpressionInput;

import { useState, useEffect } from "react";

import { Input } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { AnyProjectVariable, VarType } from "@flow/types";
import { removeWhiteSpace } from "@flow/utils";

export const NameInput: React.FC<{
  variable: AnyProjectVariable;
  onUpdate: (variable: AnyProjectVariable) => void;
  placeholder: string;
}> = ({ variable, onUpdate, placeholder }) => {
  const [localValue, setLocalValue] = useState(variable.name);

  useEffect(() => {
    setLocalValue(variable.name);
  }, [variable.name]);

  const handleBlur = () => {
    if (localValue !== variable.name) {
      onUpdate({ ...variable, name: localValue });
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.currentTarget.blur();
    }
  };

  return (
    <Input
      value={localValue}
      onChange={(e) => {
        e.stopPropagation();
        const cleansedValue = removeWhiteSpace(e.currentTarget.value);
        setLocalValue(cleansedValue);
      }}
      onBlur={handleBlur}
      onKeyDown={handleKeyDown}
      onClick={(e) => e.stopPropagation()}
      onFocus={(e) => e.stopPropagation()}
      placeholder={placeholder}
    />
  );
};

export const DefaultValueDisplay: React.FC<{
  variable: AnyProjectVariable;
}> = ({ variable }) => {
  const t = useT();

  // Determine the original type from the user-facing name
  const getOriginalType = (type: VarType): VarType => {
    const typeMapping: Record<string, VarType> = {
      [t("Attribute Name")]: "attribute_name",
      [t("Choice")]: "choice",
      [t("Color")]: "color",
      [t("Coordinate System")]: "coordinate_system",
      [t("Database Connection")]: "database_connection",
      [t("Date and Time")]: "datetime",
      [t("File or Folder")]: "file_folder",
      [t("Geometry")]: "geometry",
      [t("Message")]: "message",
      [t("Number")]: "number",
      [t("Password")]: "password",
      [t("Reprojection File")]: "reprojection_file",
      [t("Text")]: "text",
      [t("Web Connection")]: "web_connection",
      [t("Yes/No")]: "yes_no",
      [t("Unsupported")]: "unsupported",
    };

    return typeMapping[type] || type;
  };

  const originalType = getOriginalType(variable.type);
  const { defaultValue } = variable;

  // Handle empty/undefined values
  if (
    defaultValue === undefined ||
    defaultValue === null ||
    defaultValue === ""
  ) {
    return (
      <span className="text-muted-foreground italic">{t("(Not set)")}</span>
    );
  }

  switch (originalType) {
    case "choice": {
      // Handle new choice format with config.choices
      if (
        variable.config &&
        typeof variable.config === "object" &&
        "choices" in variable.config
      ) {
        const choiceConfig = variable.config as {
          choices: string[];
          allowMultiple?: boolean;
        };
        const choices = choiceConfig.choices || [];
        const isMultiple = choiceConfig.allowMultiple || false;

        // Handle multiple selection (array of strings)
        if (isMultiple && Array.isArray(defaultValue)) {
          if (defaultValue.length === 0) {
            return (
              <span className="text-muted-foreground">
                {t("No default")} ({choices.length} {t("options")})
              </span>
            );
          }
          return (
            <div className="flex items-center gap-2">
              <span className="font-medium">
                {defaultValue.length === 1
                  ? defaultValue[0]
                  : `${defaultValue.length} ${t("selected")}`}
              </span>
              <span className="text-sm text-muted-foreground">
                ({choices.length} {t("options")})
              </span>
            </div>
          );
        }

        // Handle single selection (string)
        if (typeof defaultValue === "string" && defaultValue) {
          return (
            <div className="flex items-center gap-2">
              <span className="font-medium">{defaultValue}</span>
              <span className="text-sm text-muted-foreground">
                ({choices.length} {t("options")})
              </span>
            </div>
          );
        }

        // No default selected
        return (
          <span className="text-muted-foreground">
            {t("No default")} ({choices.length} {t("options")})
          </span>
        );
      }

      // Handle legacy choice format
      if (typeof defaultValue === "object" && defaultValue?.options) {
        const selectedOption = defaultValue.selectedOption;
        const optionsCount = defaultValue.options.length;

        if (selectedOption) {
          return (
            <div className="flex items-center gap-2">
              <span className="font-medium">{selectedOption}</span>
              <span className="text-sm text-muted-foreground">
                ({optionsCount} {t("options")})
              </span>
            </div>
          );
        } else {
          return (
            <span className="text-muted-foreground">
              {t("No default")} ({optionsCount} {t("options")})
            </span>
          );
        }
      }

      return (
        <span className="text-muted-foreground">
          {t("Legacy choice format")}
        </span>
      );
    }

    case "color":
      if (typeof defaultValue === "string" && defaultValue.startsWith("#")) {
        return (
          <div className="flex items-center gap-2">
            <div
              className="h-4 w-4 rounded border"
              style={{ backgroundColor: defaultValue }}
            />
            <span className="font-mono text-sm">{defaultValue}</span>
          </div>
        );
      }
      return <span>{defaultValue}</span>;

    case "yes_no":
      return (
        <span className="font-medium">{defaultValue ? t("Yes") : t("No")}</span>
      );

    case "datetime":
      if (defaultValue instanceof Date) {
        return <span>{defaultValue.toLocaleString()}</span>;
      }
      return <span>{defaultValue}</span>;

    case "number":
      return <span className="font-mono">{defaultValue}</span>;

    case "password":
      return <span className="text-muted-foreground">••••••••</span>;

    default: {
      // For text, attribute_name, and other simple types
      const displayValue = String(defaultValue);
      if (displayValue.length > 50) {
        return (
          <span title={displayValue}>{displayValue.substring(0, 47)}...</span>
        );
      }
      return <span>{displayValue}</span>;
    }
  }
};

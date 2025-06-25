import { useState, useEffect } from "react";

import {
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable, VarType } from "@flow/types";

export const NameInput: React.FC<{
  variable: ProjectVariable;
  onUpdate: (variable: ProjectVariable) => void;
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
        setLocalValue(e.currentTarget.value);
      }}
      onBlur={handleBlur}
      onKeyDown={handleKeyDown}
      onClick={(e) => e.stopPropagation()}
      onFocus={(e) => e.stopPropagation()}
      placeholder={placeholder}
    />
  );
};

export const DefaultValueInput: React.FC<{
  variable: ProjectVariable;
  onUpdate: (variable: ProjectVariable) => void;
}> = ({ variable, onUpdate }) => {
  const t = useT();
  const [localValue, setLocalValue] = useState(variable.defaultValue || "");

  useEffect(() => {
    setLocalValue(variable.defaultValue || "");
  }, [variable.defaultValue]);

  const handleBlur = () => {
    if (localValue !== variable.defaultValue) {
      onUpdate({ ...variable, defaultValue: localValue });
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.currentTarget.blur();
    }
  };

  const handleSelectChange = (value: string) => {
    setLocalValue(value);
    onUpdate({ ...variable, defaultValue: value });
  };

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

  switch (originalType) {
    case "attribute_name":
      return (
        <Input
          value={localValue}
          onChange={(e) => {
            e.stopPropagation();
            setLocalValue(e.currentTarget.value);
          }}
          onBlur={handleBlur}
          onKeyDown={handleKeyDown}
          onClick={(e) => e.stopPropagation()}
          onFocus={(e) => e.stopPropagation()}
          placeholder={t("Enter attribute name")}
        />
      );

    case "choice":
      return (
        <Select value={localValue} onValueChange={handleSelectChange}>
          <SelectTrigger onClick={(e) => e.stopPropagation()}>
            <SelectValue placeholder={t("Select option")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="option1">{t("Option 1")}</SelectItem>
            <SelectItem value="option2">{t("Option 2")}</SelectItem>
            <SelectItem value="option3">{t("Option 3")}</SelectItem>
          </SelectContent>
        </Select>
      );

    case "color":
      return (
        <div className="flex items-center gap-2">
          <Input
            type="color"
            value={localValue || "#000000"}
            onChange={(e) => {
              e.stopPropagation();
              const newValue = e.currentTarget.value;
              setLocalValue(newValue);
              onUpdate({ ...variable, defaultValue: newValue });
            }}
            onClick={(e) => e.stopPropagation()}
            onFocus={(e) => e.stopPropagation()}
            className="h-8 w-12 rounded border p-1"
          />
          <Input
            value={localValue}
            onChange={(e) => {
              e.stopPropagation();
              setLocalValue(e.currentTarget.value);
            }}
            onBlur={handleBlur}
            onKeyDown={handleKeyDown}
            onClick={(e) => e.stopPropagation()}
            onFocus={(e) => e.stopPropagation()}
            placeholder={t("Enter color (e.g., #ff0000)")}
            className="flex-1"
          />
        </div>
      );

    default:
      return (
        <Input
          value={localValue}
          onChange={(e) => {
            e.stopPropagation();
            setLocalValue(e.currentTarget.value);
          }}
          onBlur={handleBlur}
          onKeyDown={handleKeyDown}
          onClick={(e) => e.stopPropagation()}
          onFocus={(e) => e.stopPropagation()}
          placeholder={t("Enter default value")}
        />
      );
  }
};

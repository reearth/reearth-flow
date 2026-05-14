import { DatabaseIcon } from "@phosphor-icons/react";
import { useCallback, useState, useEffect } from "react";

import {
  Button,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import ExpressionInput from "./ExpressionInput";

type AttributeAccess = "named" | "indexed";

type Props = {
  onExpressionChange: (expression: string) => void;
};

const FeatureAttributeBuilder: React.FC<Props> = ({ onExpressionChange }) => {
  const t = useT();

  const [accessType, setAccessType] = useState<AttributeAccess>("named");
  const [attributeName, setAttributeName] = useState("");
  const [indexKey, setIndexKey] = useState("");

  const accessMethods = [
    {
      value: "named" as const,
      label: t("Named Attribute"),
      description: t("Read a feature attribute by name"),
      icon: <DatabaseIcon weight="thin" className="h-4 w-4" />,
      example: 'value("cityCode")',
    },
    {
      value: "indexed" as const,
      label: t("Indexed Access"),
      description: t("Read an attribute then index into it"),
      icon: <DatabaseIcon weight="thin" className="h-4 w-4" />,
      example: 'value("coordinates")[0]',
    },
  ];

  const [currentExpression, setCurrentExpression] = useState("");

  useEffect(() => {
    let expr = "";

    switch (accessType) {
      case "named":
        if (attributeName) {
          expr = `value("${attributeName}")`;
        }
        break;
      case "indexed":
        if (attributeName) {
          const idx = indexKey || "0";
          const numericIdx = /^\d+$/.test(idx);
          expr = numericIdx
            ? `value("${attributeName}")[${idx}]`
            : `value("${attributeName}")["${idx}"]`;
        }
        break;
    }

    setCurrentExpression(expr);
  }, [accessType, attributeName, indexKey]);

  const handleInsertExpression = useCallback(() => {
    if (currentExpression.trim()) {
      onExpressionChange(currentExpression);
    }
  }, [currentExpression, onExpressionChange]);

  const selectedMethod = accessMethods.find(
    (method) => method.value === accessType,
  );

  const commonAttributes = [
    "id",
    "name",
    "code",
    "type",
    "status",
    "cityCode",
    "path",
    "coordinates",
    "geometry",
    "properties",
    "metadata",
    "timestamp",
  ];

  return (
    <div className="flex size-full flex-col gap-4 p-4">
      <div className="flex-shrink-0">
        <h4 className="text-sm font-medium">{t("Feature Data Access")}</h4>
        <p className="text-xs text-muted-foreground">
          {t("Build expressions for accessing current feature data")}
        </p>
      </div>

      <div className="space-y-4">
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="access-method-select" className="text-xs">
              {t("Access Method")}
            </Label>
            <Select
              value={accessType}
              onValueChange={(value) =>
                setAccessType(value as AttributeAccess)
              }>
              <SelectTrigger id="access-method-select">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {accessMethods.map((method) => (
                  <SelectItem key={method.value} value={method.value}>
                    <div className="flex items-center gap-2">
                      {method.icon}
                      <div>
                        <div className="text-sm">{method.label}</div>
                        <div className="text-xs text-muted-foreground">
                          {method.description}
                        </div>
                      </div>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {selectedMethod && (
            <div className="rounded border bg-muted/30 p-3">
              <div className="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
                {selectedMethod.icon}
                <span>{t("Example:")} </span>
              </div>
              <code className="text-xs break-all">
                {selectedMethod.example}
              </code>
            </div>
          )}
        </div>

        <div className="space-y-3">
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="attribute-name" className="text-xs">
                {t("Attribute Name")}
              </Label>
              <ExpressionInput
                placeholder="cityCode"
                value={attributeName}
                onChange={setAttributeName}
                className="text-sm"
                label={t("Attribute Name")}
                allowedExpressionTypes={["environment-variable"]}
              />
            </div>
            <div className="space-y-2">
              <Label className="text-xs text-muted-foreground">
                {t("Common Attributes")}
              </Label>
              <div className="flex flex-wrap gap-1 pt-2">
                {commonAttributes.slice(0, 8).map((attr) => (
                  <button
                    key={attr}
                    onClick={() => setAttributeName(attr)}
                    className="rounded bg-muted px-2 py-1 text-xs transition-colors hover:bg-accent">
                    {attr}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {accessType === "indexed" && (
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="index-key" className="text-xs">
                  {t("Index / Key")}
                </Label>
                <ExpressionInput
                  placeholder="0"
                  value={indexKey}
                  onChange={setIndexKey}
                  className="text-sm"
                  label={t("Index / Key")}
                  allowedExpressionTypes={["environment-variable", "math"]}
                />
              </div>
              <div className="space-y-2">
                <Label className="text-xs text-muted-foreground">
                  {t("Tips")}
                </Label>
                <div className="rounded border bg-muted/30 p-3 text-xs text-muted-foreground">
                  <div className="space-y-1">
                    <div>• {t("Integer → positional index (0-based)")}</div>
                    <div>• {t("Negative integer → from end (-1 = last)")}</div>
                    <div>• {t("Non-numeric → map key lookup")}</div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>

        {currentExpression && (
          <div className="mt-6 border-t pt-4">
            <div className="mb-3">
              <Label className="text-xs text-muted-foreground">
                {t("Preview")}
              </Label>
              <div className="mt-1 rounded border bg-muted/30 p-2 font-mono text-sm">
                {currentExpression}
              </div>
            </div>
            <Button onClick={handleInsertExpression} className="w-full">
              {t("Insert Expression")}
            </Button>
          </div>
        )}
      </div>
    </div>
  );
};

export default FeatureAttributeBuilder;

import { DatabaseIcon, DotIcon } from "@phosphor-icons/react";
import { useCallback, useState, useEffect } from "react";

import {
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import ExpressionInput from "./ExpressionInput";

type AttributeAccess =
  | "env_value_direct"
  | "env_value_indexed"
  | "env_feature_type"
  | "env_feature_id"
  | "env_lod"
  | "feature_attributes"
  | "custom_path";

type Props = {
  onExpressionChange: (expression: string) => void;
};

const FeatureAttributeBuilder: React.FC<Props> = ({ onExpressionChange }) => {
  const t = useT();

  const [accessType, setAccessType] =
    useState<AttributeAccess>("env_value_direct");
  const [attributeName, setAttributeName] = useState("");
  const [indexKey, setIndexKey] = useState("");

  const accessMethods = [
    {
      value: "env_value_direct" as const,
      label: t("Environment Value (Direct)"),
      description: t("Access current feature value directly"),
      icon: <DatabaseIcon weight="thin" className="h-4 w-4" />,
      example: 'env.get("__value").cityCode',
    },
    {
      value: "env_value_indexed" as const,
      label: t("Environment Value (Indexed)"),
      description: t("Access current feature value with bracket notation"),
      icon: <DatabaseIcon weight="thin" className="h-4 w-4" />,
      example: 'env.get("__value")["path"]',
    },
    {
      value: "env_feature_type" as const,
      label: t("Feature Type"),
      description: t("Get the current feature type"),
      icon: <DatabaseIcon weight="thin" className="h-4 w-4" />,
      example: 'env.get("__feature_type")',
    },
    {
      value: "env_feature_id" as const,
      label: t("Feature ID"),
      description: t("Get the current feature identifier"),
      icon: <DatabaseIcon weight="thin" className="h-4 w-4" />,
      example: 'env.get("__feature_id")',
    },
    {
      value: "env_lod" as const,
      label: t("Level of Detail"),
      description: t("Get the current level of detail value"),
      icon: <DatabaseIcon weight="thin" className="h-4 w-4" />,
      example: 'env.get("__lod")',
    },
    {
      value: "feature_attributes" as const,
      label: t("Feature Attributes"),
      description: t("Access feature attributes directly"),
      icon: <DotIcon weight="thin" className="h-4 w-4" />,
      example: "feature.attributes.name",
    },
    {
      value: "custom_path" as const,
      label: t("Custom Property Path"),
      description: t("Access nested properties with custom path"),
      icon: <DatabaseIcon weight="thin" className="h-4 w-4" />,
      example: "data.geometry.coordinates[0]",
    },
  ];

  const generateExpression = useCallback(() => {
    let expr = "";

    switch (accessType) {
      case "env_value_direct":
        if (attributeName) {
          expr = `env.get("__value").${attributeName}`;
        }
        break;
      case "env_value_indexed":
        if (indexKey) {
          expr = `env.get("__value")["${indexKey}"]`;
        }
        break;
      case "env_feature_type":
        expr = 'env.get("__feature_type")';
        break;
      case "env_feature_id":
        expr = 'env.get("__feature_id")';
        break;
      case "env_lod":
        expr = 'env.get("__lod")';
        break;
      case "feature_attributes":
        if (attributeName) {
          expr = `feature.attributes.${attributeName}`;
        }
        break;
      case "custom_path":
        if (attributeName) {
          expr = attributeName;
        }
        break;
    }

    onExpressionChange(expr);
  }, [accessType, attributeName, indexKey, onExpressionChange]);

  // Generate expression whenever inputs change
  useEffect(() => {
    generateExpression();
  }, [generateExpression]);

  const selectedMethod = accessMethods.find(
    (method) => method.value === accessType,
  );

  // Common attribute suggestions
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
        {/* Two-column layout for method selection and example */}
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

        {/* Input fields with improved horizontal layout */}
        <div className="space-y-3">
          {(accessType === "env_value_direct" ||
            accessType === "feature_attributes") && (
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
                      className="rounded bg-muted px-2 py-1 text-xs hover:bg-accent transition-colors">
                      {attr}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}

          {accessType === "env_value_indexed" && (
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="index-key" className="text-xs">
                  {t("Index Key")}
                </Label>
                <ExpressionInput
                  placeholder="path"
                  value={indexKey}
                  onChange={setIndexKey}
                  className="text-sm"
                  label={t("Index Key")}
                  allowedExpressionTypes={["environment-variable"]}
                />
              </div>
              <div className="space-y-2">
                <Label className="text-xs text-muted-foreground">
                  {t("Common Keys")}
                </Label>
                <div className="flex flex-wrap gap-1 pt-2">
                  {["path", "id", "name", "type", "code", "status"].map(
                    (key) => (
                      <button
                        key={key}
                        onClick={() => setIndexKey(key)}
                        className="rounded bg-muted px-2 py-1 text-xs hover:bg-accent transition-colors">
                        {key}
                      </button>
                    ),
                  )}
                </div>
              </div>
            </div>
          )}

          {accessType === "custom_path" && (
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="custom-path" className="text-xs">
                  {t("Custom Property Path")}
                </Label>
                <ExpressionInput
                  placeholder="data.geometry.coordinates[0]"
                  value={attributeName}
                  onChange={setAttributeName}
                  className="text-sm"
                  label={t("Custom Property Path")}
                  allowedExpressionTypes={[
                    "environment-variable",
                    "feature-attribute",
                    "math",
                  ]}
                />
              </div>
              <div className="space-y-2">
                <Label className="text-xs text-muted-foreground">
                  {t("Path Examples")}
                </Label>
                <div className="space-y-1 pt-2 text-xs text-muted-foreground">
                  <div>
                    <code>data.properties.name</code>
                  </div>
                  <div>
                    <code>geometry.coordinates[0]</code>
                  </div>
                  <div>
                    <code>feature.metadata.timestamp</code>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default FeatureAttributeBuilder;

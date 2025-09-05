import {
  MagicWandIcon,
  EyeIcon,
  CheckCircleIcon,
  WarningCircleIcon,
} from "@phosphor-icons/react";
import { useCallback, useState, useEffect } from "react";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  Button,
  Input,
  Label,
  ScrollArea,
  Badge,
  Alert,
  AlertDescription,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import type { ExpressionTemplate } from "./templateData";
import {
  processTemplate,
  validatePlaceholderValue,
  generateTemplatePreview,
  isTemplateReady,
  type PlaceholderValue,
} from "./templateUtils";

type Props = {
  open: boolean;
  template: ExpressionTemplate | null;
  onClose: () => void;
  onInsert: (populatedCode: string) => void;
};

const TemplatePlaceholderDialog: React.FC<Props> = ({
  open,
  template,
  onClose,
  onInsert,
}) => {
  const t = useT();
  const [placeholderValues, setPlaceholderValues] = useState<
    PlaceholderValue[]
  >([]);
  const [showPreview, setShowPreview] = useState(false);
  const [validationErrors, setValidationErrors] = useState<
    Record<string, string>
  >({});

  // Initialize placeholder values when template changes
  useEffect(() => {
    if (template) {
      const initialValues = template.placeholders.map((placeholder) => ({
        key: placeholder.key,
        value: placeholder.defaultValue || "",
      }));
      setPlaceholderValues(initialValues);
      setValidationErrors({});
    }
  }, [template]);

  const updatePlaceholderValue = useCallback(
    (key: string, value: string) => {
      setPlaceholderValues((prev) =>
        prev.map((item) => (item.key === key ? { ...item, value } : item)),
      );

      // Validate the updated value
      if (template) {
        const validation = validatePlaceholderValue(key, value, template);
        setValidationErrors((prev) => ({
          ...prev,
          [key]: validation.isValid ? "" : validation.error || "Invalid value",
        }));
      }
    },
    [template],
  );

  const handleInsert = useCallback(() => {
    if (!template) return;

    const processed = processTemplate(template, placeholderValues);
    onInsert(processed.populatedCode);
    onClose();
  }, [template, placeholderValues, onInsert, onClose]);

  const canInsert = template
    ? isTemplateReady(template, placeholderValues)
    : false;
  const hasValidationErrors = Object.values(validationErrors).some(
    (error) => error !== "",
  );

  if (!template) return null;

  const processedTemplate = processTemplate(template, placeholderValues);
  const preview = generateTemplatePreview(template, placeholderValues);

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent size="3xl" className="max-h-[90vh]">
        <DialogHeader>
          <DialogTitle>
            <div className="flex items-center gap-2">
              <MagicWandIcon weight="thin" />
              {t("Configure Template")}: {template.name}
            </div>
          </DialogTitle>
        </DialogHeader>

        <div className="flex flex-col space-y-6">
          {/* Template Info */}
          <div className="rounded-lg bg-muted/30 p-4">
            <p className="mb-2 text-sm text-muted-foreground">
              {template.description}
            </p>
            {template.usageExample && (
              <p className="text-xs text-muted-foreground">
                <strong>{t("Example use case")}:</strong>{" "}
                {template.usageExample}
              </p>
            )}
          </div>

          {/* Placeholder Configuration */}
          {template.placeholders.length > 0 && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-medium">
                  {t("Template Parameters")} ({template.placeholders.length})
                </h3>
                <Badge variant="outline" className="text-xs">
                  {placeholderValues.filter((p) => p.value.trim()).length} /{" "}
                  {template.placeholders.length} filled
                </Badge>
              </div>

              <ScrollArea className="max-h-64 pr-2">
                <div className="space-y-4">
                  {template.placeholders.map((placeholder) => {
                    const currentValue =
                      placeholderValues.find((v) => v.key === placeholder.key)
                        ?.value || "";
                    const hasError = validationErrors[placeholder.key];

                    return (
                      <div key={placeholder.key} className="space-y-2">
                        <Label
                          htmlFor={`placeholder-${placeholder.key}`}
                          className="text-sm">
                          <code className="mr-2 rounded bg-muted/50 px-1.5 py-0.5 font-mono text-xs">
                            {placeholder.key}
                          </code>
                          {placeholder.description}
                          {!placeholder.defaultValue && (
                            <span className="ml-1 text-red-500">*</span>
                          )}
                        </Label>

                        <Input
                          id={`placeholder-${placeholder.key}`}
                          value={currentValue}
                          onChange={(e) =>
                            updatePlaceholderValue(
                              placeholder.key,
                              e.target.value,
                            )
                          }
                          placeholder={
                            placeholder.defaultValue ||
                            `Enter ${placeholder.key}...`
                          }
                          className={hasError ? "border-red-500" : ""}
                        />

                        {placeholder.defaultValue && (
                          <p className="text-xs text-muted-foreground">
                            {t("Default")}:{" "}
                            <code className="rounded bg-muted/50 px-1">
                              {placeholder.defaultValue}
                            </code>
                          </p>
                        )}

                        {hasError && (
                          <p className="flex items-center gap-1 text-xs text-red-600">
                            <WarningCircleIcon className="h-3 w-3" />
                            {hasError}
                          </p>
                        )}
                      </div>
                    );
                  })}
                </div>
              </ScrollArea>
            </div>
          )}

          {/* Preview Section */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium">{t("Preview")}</h3>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowPreview(!showPreview)}
                className="h-8">
                <EyeIcon className="mr-1 h-4 w-4" />
                {showPreview ? t("Hide") : t("Show")} {t("Full Code")}
              </Button>
            </div>

            <div className="rounded-lg bg-muted/30 p-4">
              <pre className="overflow-x-auto font-mono text-xs">
                <code>
                  {showPreview
                    ? processedTemplate.populatedCode
                    : preview.split("\n").slice(0, 3).join("\n") +
                      (preview.split("\n").length > 3 ? "\n..." : "")}
                </code>
              </pre>
            </div>

            {!canInsert && !hasValidationErrors && (
              <Alert>
                <CheckCircleIcon className="h-4 w-4" />
                <AlertDescription className="text-xs">
                  {t(
                    "Fill in all required parameters to preview the complete expression.",
                  )}
                </AlertDescription>
              </Alert>
            )}

            {hasValidationErrors && (
              <Alert variant="destructive">
                <WarningCircleIcon className="h-4 w-4" />
                <AlertDescription className="text-xs">
                  {t(
                    "Please fix validation errors before inserting the template.",
                  )}
                </AlertDescription>
              </Alert>
            )}
          </div>
        </div>

        <DialogFooter className="gap-2">
          <Button variant="outline" onClick={onClose}>
            {t("Cancel")}
          </Button>
          <Button
            onClick={handleInsert}
            disabled={!canInsert || hasValidationErrors}
            className="min-w-[100px]">
            <MagicWandIcon className="mr-2 h-4 w-4" />
            {t("Insert Template")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default TemplatePlaceholderDialog;

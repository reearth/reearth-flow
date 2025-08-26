import { CaretLeftIcon, WrenchIcon } from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import ConditionalBuilder from "./ConditionalBuilder";
import EnvironmentVariableBuilder from "./EnvironmentVariableBuilder";
import ExpressionTypePicker, {
  type ExpressionType,
} from "./ExpressionTypePicker";
import FeatureAttributeBuilder from "./FeatureAttributeBuilder";
import FilePathBuilder from "./FilePathBuilder";
import MathBuilder from "./MathBuilder";

type Props = {
  isOpen: boolean;
  onClose: () => void;
  onExpressionBuilt: (expression: string) => void;
  allowedExpressionTypes?: ExpressionType[]; // Filter available expression types
  initialValue?: string;
  fieldLabel?: string; // For modal title context
};

const SubExpressionBuilderModal: React.FC<Props> = ({
  isOpen,
  onClose,
  onExpressionBuilt,
  allowedExpressionTypes,
  initialValue,
  fieldLabel,
}) => {
  const t = useT();
  const [selectedExpressionType, setSelectedExpressionType] =
    useState<ExpressionType | null>(null);
  const [currentExpression, setCurrentExpression] = useState(
    initialValue || "",
  );

  const handleExpressionTypeSelect = useCallback(
    (type: ExpressionType) => {
      setSelectedExpressionType(type);

      // For now, if user selects "custom", just close and let them use raw mode
      if (type === "custom") {
        onClose();
      }
    },
    [onClose],
  );

  const handleExpressionBuilderChange = useCallback((expression: string) => {
    setCurrentExpression(expression);
  }, []);

  const handleBackToSelection = useCallback(() => {
    setSelectedExpressionType(null);
  }, []);

  const handleUseExpression = useCallback(() => {
    onExpressionBuilt(currentExpression);
  }, [currentExpression, onExpressionBuilt]);

  const getModalTitle = () => {
    if (fieldLabel) {
      return t("Build Expression for {{field}}", { field: fieldLabel });
    }
    return t("Build Sub-Expression");
  };

  // Filter expression types if specified
  const filteredExpressionTypes = allowedExpressionTypes;

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent size="2xl" className="max-h-[80vh] overflow-hidden">
        <DialogHeader>
          <DialogTitle>
            <div className="flex items-center gap-2">
              <WrenchIcon weight="thin" />
              {getModalTitle()}
            </div>
          </DialogTitle>
        </DialogHeader>

        <div className="flex h-[500px] flex-col">
          {!selectedExpressionType ? (
            // Expression Type Selection
            <div className="flex-1 p-4">
              <ExpressionTypePicker
                onTypeSelect={handleExpressionTypeSelect}
                allowedTypes={filteredExpressionTypes}
              />
            </div>
          ) : (
            // Selected Builder Interface
            <div className="flex h-full flex-col">
              {/* Back Button */}
              <div className="flex-shrink-0 border-b p-4 pb-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleBackToSelection}
                  className="h-8 gap-1 px-2">
                  <CaretLeftIcon className="h-4 w-4" />
                  {t("Back to Type Selection")}
                </Button>
              </div>

              {/* Builder Content */}
              <div className="min-h-0 flex-1 overflow-auto">
                {selectedExpressionType === "file-path" && (
                  <FilePathBuilder
                    onExpressionChange={handleExpressionBuilderChange}
                  />
                )}
                {selectedExpressionType === "feature-attribute" && (
                  <FeatureAttributeBuilder
                    onExpressionChange={handleExpressionBuilderChange}
                  />
                )}
                {selectedExpressionType === "conditional" && (
                  <ConditionalBuilder
                    onExpressionChange={handleExpressionBuilderChange}
                  />
                )}
                {selectedExpressionType === "math" && (
                  <MathBuilder
                    onExpressionChange={handleExpressionBuilderChange}
                  />
                )}
                {selectedExpressionType === "environment-variable" && (
                  <EnvironmentVariableBuilder
                    onExpressionChange={handleExpressionBuilderChange}
                  />
                )}
                {![
                  "file-path",
                  "feature-attribute",
                  "conditional",
                  "math",
                  "environment-variable",
                ].includes(selectedExpressionType) && (
                  <div className="flex flex-1 flex-col items-center justify-center p-4 text-center text-muted-foreground">
                    <p className="mb-4">
                      {t("Selected:")} {selectedExpressionType}
                    </p>
                    <div className="text-sm">
                      {t("Expression builder for {{type}} will go here", {
                        type: selectedExpressionType,
                      })}
                    </div>
                  </div>
                )}
              </div>

              {/* Action Buttons */}
              <div className="flex-shrink-0 border-t p-4">
                <div className="flex items-center justify-between">
                  {/* Expression Preview */}
                  <div className="mr-4 flex-1">
                    <div className="mb-1 text-xs text-muted-foreground">
                      {t("Generated Expression:")}
                    </div>
                    <code className="rounded bg-muted/50 px-2 py-1 text-xs break-all">
                      {currentExpression || t("No expression generated yet")}
                    </code>
                  </div>

                  {/* Buttons */}
                  <div className="flex gap-2">
                    <Button variant="outline" onClick={onClose}>
                      {t("Cancel")}
                    </Button>
                    <Button
                      onClick={handleUseExpression}
                      disabled={!currentExpression}>
                      {t("Use This Expression")}
                    </Button>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default SubExpressionBuilderModal;

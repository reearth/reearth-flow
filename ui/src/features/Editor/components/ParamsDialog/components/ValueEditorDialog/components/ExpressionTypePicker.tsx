import {
  FileIcon,
  MathOperationsIcon,
  GitBranchIcon,
  DatabaseIcon,
  CircleIcon,
  QuestionIcon,
} from "@phosphor-icons/react";
import { useCallback } from "react";

import {
  Button,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

export type ExpressionType =
  | "file-path"
  | "feature-attribute"
  | "conditional"
  | "math"
  | "environment-variable"
  | "json-query";

type ExpressionTypeOption = {
  type: ExpressionType;
  title: string;
  description: string;
  icon: React.ReactNode;
  examples: string[];
};

type Props = {
  onTypeSelect: (type: ExpressionType) => void;
  allowedTypes?: ExpressionType[]; // Filter which types to show
};

const ExpressionTypePicker: React.FC<Props> = ({
  onTypeSelect,
  allowedTypes,
}) => {
  const t = useT();

  const handleTypeSelect = useCallback(
    (type: ExpressionType) => {
      onTypeSelect(type);
    },
    [onTypeSelect],
  );

  const expressionTypes: ExpressionTypeOption[] = [
    {
      type: "file-path",
      title: t("File Path Operations"),
      description: t("Build file paths, extract filenames, join directories"),
      icon: <FileIcon weight="thin" className="h-6 w-6" />,
      examples: [
        'file::join_path("/output", "result.zip")',
        "file::extract_filename(path)",
        "file::extract_filename_without_ext(path)",
      ],
    },
    {
      type: "feature-attribute",
      title: t("Feature Data Access"),
      description: t("Access and manipulate current feature attributes"),
      icon: <DatabaseIcon weight="thin" className="h-6 w-6" />,
      examples: [
        'env.get("__value").cityCode',
        'env.get("__value")["path"]',
        "feature.attributes.name",
      ],
    },
    {
      type: "conditional",
      title: t("Conditional Logic"),
      description: t("Create if-then-else expressions for dynamic behavior"),
      icon: <GitBranchIcon weight="thin" className="h-6 w-6" />,
      examples: [
        'if condition { "result" } else { "default" }',
        'city == "Tokyo" ? "JP" : "Other"',
        'value > 100 ? "High" : "Low"',
      ],
    },
    {
      type: "math",
      title: t("Mathematical Operations"),
      description: t("Perform calculations on numeric values"),
      icon: <MathOperationsIcon weight="thin" className="h-6 w-6" />,
      examples: [
        "(xmin + xmax) / 2.0",
        "area * 0.0001", // Convert to hectares
        "round(distance, 2)",
      ],
    },
    {
      type: "environment-variable",
      title: t("Environment Variables"),
      description: t("Access project variables and workflow parameters"),
      icon: <CircleIcon weight="thin" className="h-6 w-6" />,
      examples: [
        'env.get("outputPath")',
        'env.get("cityGmlPath")',
        'env.get("targetPackages")',
      ],
    },
    {
      type: "json-query",
      title: t("JSON Data Query"),
      description: t("Query and extract data from JSON using JSONPath"),
      icon: <DatabaseIcon weight="thin" className="h-6 w-6" />,
      examples: [
        'json::find_value_by_json_path(data, "$.items[0].name")',
        'json::exists_value_by_json_path(data, "$.user")',
        'json::find_value_by_json_path(env.get("__value"), "$.attributes")',
      ],
    },
  ];

  return (
    <div className="flex size-full min-h-0 flex-col gap-4">
      <div className="flex-shrink-0 py-2 text-center">
        <h3 className="text-lg font-medium">{t("Choose Expression Type")}</h3>
        <p className="text-sm text-muted-foreground">
          {t("Select the type of expression you want to create")}
        </p>
      </div>

      <div className="min-h-0 overflow-scroll rounded px-2 pt-1">
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
          {expressionTypes
            .filter(
              (option) => !allowedTypes || allowedTypes.includes(option.type),
            )
            .map((option) => (
              <div
                key={option.type}
                className="overflow-hidden rounded-lg border">
                <Button
                  variant="ghost"
                  className="flex size-full flex-col items-start justify-between p-3 text-left hover:bg-accent/50"
                  onDoubleClick={() => handleTypeSelect(option.type)}
                  data-testid={`expression-type-${option.type}`}>
                  <div className="flex w-full items-start justify-between gap-2">
                    <div className="flex items-center gap-2">
                      <div className="mt-0.5 flex-shrink-0">{option.icon}</div>
                      <div className="min-w-0 flex-1">
                        <span className="block text-sm leading-tight font-medium">
                          {option.title}
                        </span>
                      </div>
                    </div>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div className="flex flex-shrink-0 items-center justify-center p-1">
                          <QuestionIcon className="h-4 w-4 text-muted-foreground hover:text-foreground" />
                        </div>
                      </TooltipTrigger>
                      <TooltipContent
                        side="bottom"
                        className="max-w-xs bg-secondary">
                        <div className="space-y-2">
                          <p className="text-xs font-medium">
                            {t("Examples:")}
                          </p>
                          <div className="space-1 flex flex-col">
                            {option.examples.map((example, index) => (
                              <code
                                key={index}
                                className="rounded border-b px-2 py-1 text-xs wrap-break-word whitespace-break-spaces">
                                {example}
                              </code>
                            ))}
                          </div>
                        </div>
                      </TooltipContent>
                    </Tooltip>
                  </div>
                  <p className="mt-2 text-xs leading-relaxed wrap-break-word whitespace-break-spaces text-muted-foreground">
                    {option.description}
                  </p>
                </Button>
              </div>
            ))}
        </div>
      </div>
    </div>
  );
};

export default ExpressionTypePicker;

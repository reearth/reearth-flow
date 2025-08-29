import {
  MagnifyingGlassIcon,
  TagIcon,
  CodeIcon,
  EyeIcon,
} from "@phosphor-icons/react";
import { useCallback, useState, useMemo } from "react";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  ScrollArea,
  Input,
  Button,
  Badge,
  Tabs,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import {
  getExpressionTemplates,
  getTemplateCategories,
  getTemplatesByCategory,
  searchTemplates,
  type ExpressionTemplate,
} from "./templateData";
import { formatTemplateCode } from "./templateUtils";

type Props = {
  open: boolean;
  onClose: () => void;
  onTemplateSelect: (template: ExpressionTemplate) => void;
};

const TemplateLibraryDialog: React.FC<Props> = ({
  open,
  onClose,
  onTemplateSelect,
}) => {
  const t = useT();
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState<string>("all");
  const [previewTemplate, setPreviewTemplate] =
    useState<ExpressionTemplate | null>(null);

  // Filter templates based on search and category
  const filteredTemplates = useMemo(() => {
    let templates = getExpressionTemplates(t);

    // Apply search filter
    if (searchQuery.trim()) {
      templates = searchTemplates(searchQuery, t);
    }

    // Apply category filter
    if (selectedCategory !== "all") {
      templates = templates.filter(
        (tmpl) => tmpl.category === selectedCategory,
      );
    }

    return templates;
  }, [searchQuery, selectedCategory, t]);

  const handleTemplateSelect = useCallback(
    (template: ExpressionTemplate) => {
      onTemplateSelect(template);
      onClose();
    },
    [onTemplateSelect, onClose],
  );

  const handlePreviewToggle = useCallback((template: ExpressionTemplate) => {
    setPreviewTemplate((prev) => (prev?.id === template.id ? null : template));
  }, []);

  const categories = Object.entries(getTemplateCategories(t));

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent size="3xl" className="max-h-[90vh]">
        <DialogHeader>
          <DialogTitle>
            <div className="flex items-center gap-2">
              <CodeIcon weight="thin" />
              {t("Expression Template Library")}
            </div>
          </DialogTitle>
        </DialogHeader>

        <div className="flex h-[70vh] gap-4">
          {/* Left Panel - Categories and Search */}
          <div className="w-80 border-r pr-4">
            {/* Search */}
            <div className="relative mb-4">
              <MagnifyingGlassIcon className="absolute top-3 left-3 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder={t("Search templates...")}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10"
              />
            </div>

            {/* Category Tabs */}
            <Tabs value={selectedCategory} onValueChange={setSelectedCategory}>
              <TabsList className="grid w-full grid-cols-1">
                <TabsTrigger value="all" className="justify-start">
                  <span className="mr-2">📚</span>
                  {t("All Templates")} ({getExpressionTemplates(t).length})
                </TabsTrigger>
              </TabsList>

              <div className="mt-4 space-y-2">
                {categories.map(([key, category]) => {
                  const count = getTemplatesByCategory(
                    key as keyof ReturnType<typeof getTemplateCategories>,
                    t,
                  ).length;
                  return (
                    <button
                      key={key}
                      onClick={() => setSelectedCategory(key)}
                      className={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors hover:bg-accent ${
                        selectedCategory === key ? "bg-accent" : ""
                      }`}>
                      <div className="flex items-center gap-2">
                        <span>{category.icon}</span>
                        <span className="font-medium">{category.name}</span>
                      </div>
                      <Badge variant="secondary" className="text-xs">
                        {count}
                      </Badge>
                    </button>
                  );
                })}
              </div>
            </Tabs>
          </div>

          {/* Right Panel - Template List */}
          <div className="flex-1">
            <ScrollArea className="h-full pr-4">
              <div className="space-y-3">
                {filteredTemplates.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-12 text-center">
                    <MagnifyingGlassIcon className="mb-4 h-12 w-12 text-muted-foreground/50" />
                    <h3 className="mb-2 text-lg font-medium text-muted-foreground">
                      {t("No templates found")}
                    </h3>
                    <p className="max-w-md text-sm text-muted-foreground">
                      {searchQuery
                        ? t(
                            "Try adjusting your search terms or browse a different category.",
                          )
                        : t("No templates available in this category.")}
                    </p>
                  </div>
                ) : (
                  filteredTemplates.map((template) => (
                    <div
                      key={template.id}
                      className="rounded-lg border p-4 transition-colors hover:bg-accent/50">
                      {/* Template Header */}
                      <div className="mb-2 flex items-start justify-between">
                        <div className="flex-1">
                          <h3 className="mb-1 text-sm font-medium">
                            {template.name}
                          </h3>
                          <p className="mb-2 text-xs text-muted-foreground">
                            {template.description}
                          </p>
                          <div className="mb-3 flex items-center gap-1">
                            {template.tags.slice(0, 3).map((tag) => (
                              <Badge
                                key={tag}
                                variant="outline"
                                className="text-xs">
                                <TagIcon className="mr-1 h-3 w-3" />
                                {tag}
                              </Badge>
                            ))}
                            {template.tags.length > 3 && (
                              <Badge variant="outline" className="text-xs">
                                +{template.tags.length - 3}
                              </Badge>
                            )}
                          </div>
                        </div>
                        <div className="ml-4 flex gap-2">
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handlePreviewToggle(template)}
                            className="px-2">
                            <EyeIcon className="h-4 w-4" />
                          </Button>
                          <Button
                            size="sm"
                            onClick={() => handleTemplateSelect(template)}>
                            {t("Use Template")}
                          </Button>
                        </div>
                      </div>

                      {/* Code Preview */}
                      <div className="mb-3 rounded border bg-muted/30 p-3">
                        <code className="font-mono text-xs text-muted-foreground">
                          {template.preview ||
                            template.rhaiCode.split("\n")[0] + "..."}
                        </code>
                      </div>

                      {/* Expandable Details */}
                      {previewTemplate?.id === template.id && (
                        <div>
                          <div className="mt-3 border-t pt-3">
                            <div className="space-y-3">
                              {template.usageExample && (
                                <div>
                                  <h4 className="mb-1 text-xs font-medium text-muted-foreground">
                                    {t("Usage Example")}
                                  </h4>
                                  <p className="text-xs text-muted-foreground">
                                    {template.usageExample}
                                  </p>
                                </div>
                              )}

                              <div>
                                <h4 className="mb-2 text-xs font-medium text-muted-foreground">
                                  {t("Full Code")}
                                </h4>
                                <pre className="overflow-x-auto rounded bg-muted/50 p-3 font-mono text-xs">
                                  <code>
                                    {formatTemplateCode(template.rhaiCode)}
                                  </code>
                                </pre>
                              </div>

                              {template.placeholders.length > 0 && (
                                <div>
                                  <h4 className="mb-2 text-xs font-medium text-muted-foreground">
                                    {t("Placeholders")} (
                                    {template.placeholders.length})
                                  </h4>
                                  <div className="space-y-2">
                                    {template.placeholders.map(
                                      (placeholder) => (
                                        <div
                                          key={placeholder.key}
                                          className="flex gap-2 text-xs">
                                          <Badge
                                            variant="secondary"
                                            className="font-mono text-xs">
                                            {placeholder.key}
                                          </Badge>
                                          <span className="text-muted-foreground">
                                            {placeholder.description}
                                            {placeholder.defaultValue && (
                                              <span className="ml-2 rounded bg-muted/50 px-1 font-mono">
                                                {placeholder.defaultValue}
                                              </span>
                                            )}
                                          </span>
                                        </div>
                                      ),
                                    )}
                                  </div>
                                </div>
                              )}
                            </div>
                          </div>
                        </div>
                      )}
                    </div>
                  ))
                )}
              </div>
            </ScrollArea>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default TemplateLibraryDialog;

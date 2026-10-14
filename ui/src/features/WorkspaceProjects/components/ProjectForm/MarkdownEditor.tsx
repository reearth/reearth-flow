import type { ReactNode } from "react";

import {
  Label,
  Markdown,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  TextArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  id: string;
  label: string;
  value: string;
  placeholder?: string;
  // Shown beside the Edit/Preview switch, e.g. a template picker.
  adornment?: ReactNode;
  onChange: (value: string) => void;
};

const MarkdownEditor: React.FC<Props> = ({
  id,
  label,
  value,
  placeholder,
  adornment,
  onChange,
}) => {
  const t = useT();
  return (
    <Tabs defaultValue="edit" className="flex flex-col gap-2">
      <div className="flex items-center justify-between gap-2">
        <Label htmlFor={id}>{label}</Label>
        <div className="flex items-center gap-2">
          {adornment}
          <TabsList className="gap-1">
            <TabsTrigger value="edit">{t("Edit")}</TabsTrigger>
            <TabsTrigger value="preview">{t("Preview")}</TabsTrigger>
          </TabsList>
        </div>
      </div>
      <TabsContent value="edit">
        <TextArea
          id={id}
          className="min-h-[200px] font-mono text-xs"
          placeholder={placeholder}
          value={value}
          onChange={(e) => onChange(e.target.value)}
        />
      </TabsContent>
      <TabsContent
        value="preview"
        className="max-h-[400px] min-h-[200px] overflow-y-auto rounded-md border px-3 py-1">
        {value.trim() ? (
          <Markdown>{value}</Markdown>
        ) : (
          <p className="my-2 text-sm text-muted-foreground">
            {t("Nothing to preview")}
          </p>
        )}
      </TabsContent>
    </Tabs>
  );
};

export { MarkdownEditor };

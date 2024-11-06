import { Download } from "@phosphor-icons/react";

import { Button, Input, Label } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { openLinkInNewTab } from "@flow/utils";

export type DetailsBoxContent = {
  id: string;
  name: string;
  value: string;
  type?: "link" | "download" | "textbox";
};

type Props = {
  title: string;
  content?: DetailsBoxContent[];
  onContentChange?: (content: DetailsBoxContent) => void;
};

const DetailsBox: React.FC<Props> = ({ title, content, onContentChange }) => {
  const t = useT();
  const filteredContent = content?.filter(
    (detail) => detail.type !== "download",
  );
  const downloadContent = content?.filter(
    (detail) => detail.type === "download",
  );
  return (
    <div className="rounded-md border dark:font-thin">
      <div className="flex justify-between border-b px-4 py-2">
        <p className="text-xl">{title}</p>
        <div className="flex gap-2">
          {downloadContent?.map((detail) => (
            <Button
              key={detail.id}
              className="p-0"
              variant="outline"
              type="button">
              <a
                className="flex h-full items-center gap-2 rounded px-4 py-2"
                href={detail.value}
                download>
                <Download />
                <p className="font-light">{detail.name}</p>
              </a>
            </Button>
          ))}
        </div>
      </div>
      <div className="flex flex-col gap-2 p-4">
        {filteredContent ? (
          filteredContent.map((detail) =>
            detail.type === "textbox" ? (
              <div key={detail.id} className="flex items-center gap-2">
                <Label>{detail.name}: </Label>
                <Input
                  className="max-w-[500px]"
                  value={detail.value}
                  onChange={(e) =>
                    onContentChange?.({
                      ...detail,
                      value: e.currentTarget.value,
                    })
                  }
                />
              </div>
            ) : (
              <p key={detail.id}>
                {detail.name}
                {": "}
                <span
                  className={`${detail.type === "link" ? "cursor-pointer font-light text-blue-400 hover:text-blue-300" : "font-normal"}`}
                  onClick={openLinkInNewTab(detail.value)}>
                  {detail.value}
                </span>
              </p>
            ),
          )
        ) : (
          <p>{t("No content to display")}</p>
        )}
      </div>
    </div>
  );
};

export { DetailsBox };

import { CaretDown, CaretUp, Download } from "@phosphor-icons/react";
import { useState } from "react";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { openLinkInNewTab } from "@flow/utils";

export type DetailsBoxContent = {
  id: string;
  name: string;
  value: string | string[];
  type?: "link" | "download" | "status";
};

type Props = {
  title: string;
  content?: DetailsBoxContent[];
  collapsible?: boolean;
};

const DetailsBox: React.FC<Props> = ({ title, content, collapsible }) => {
  const t = useT();
  const [collapsed, setCollapsed] = useState(false);

  const filteredContent = content?.filter(
    (detail) => detail.type !== "download" && detail.type !== "status",
  );
  const downloadContent = content?.filter(
    (detail) => detail.type === "download",
  );

  const status = content?.find((detail) => detail.id === "status")?.value;

  // This function is necessary because without it the file will sometimes open in the browser instead of downloading
  const handleDownload =
    (url: string) => async (e: React.MouseEvent<HTMLAnchorElement>) => {
      e.preventDefault();

      try {
        const response = await fetch(url);
        const blob = await response.blob();

        const blobUrl = window.URL.createObjectURL(blob);

        const link = document.createElement("a");
        link.href = blobUrl;

        const fileName =
          response.headers.get("Content-Disposition")?.split("filename=")[1] ||
          url.split("/").pop() ||
          "workflow.json";
        link.download = fileName;
        document.body.appendChild(link);
        link.click();

        document.body.removeChild(link);
        window.URL.revokeObjectURL(blobUrl);
      } catch (error) {
        console.error("Download failed:", error);
        // Fallback to direct navigation if fetch fails
        window.location.href = url;
      }
    };

  return (
    <div className="rounded-md border dark:font-thin">
      <div
        className={`flex ${collapsible ? "cursor-pointer" : ""} justify-between px-4 py-2`}
        onClick={() => collapsible && setCollapsed(!collapsed)}>
        <div className="flex items-center gap-4">
          <p className="text-xl">{title}</p>
          {collapsible ? collapsed ? <CaretUp /> : <CaretDown /> : null}
        </div>
        <div className="flex items-center gap-2">
          {!collapsed &&
            downloadContent?.map((detail) =>
              Array.isArray(detail.value) ? (
                detail.value.map((value, index) => (
                  <Button
                    key={detail.id + index}
                    className="p-0"
                    variant="outline"
                    size="sm"
                    type="button">
                    <a
                      className="flex h-full items-center gap-2 rounded px-4 py-2"
                      href={value}
                      onClick={() => value && handleDownload(value)}>
                      <Download />
                      <p className="max-w-[100px] truncate font-light">
                        {value.split("/").pop()}
                      </p>
                    </a>
                  </Button>
                ))
              ) : (
                <Button
                  key={detail.id}
                  className="p-0"
                  variant="outline"
                  size="sm"
                  type="button">
                  <a
                    className="flex h-full items-center gap-2 rounded px-4 py-2"
                    href={detail.value}
                    onClick={() =>
                      typeof detail.value === "string" &&
                      handleDownload(detail.value)
                    }>
                    <Download />
                    <p className="font-light">{detail.name}</p>
                  </a>
                </Button>
              ),
            )}
          {status && (
            <div
              className={`${status === "completed" ? "bg-success" : status === "running" ? "active-node-status" : status === "cancelled" ? "bg-warning" : status === "failed" ? "bg-destructive" : "queued-node-status"} size-4 rounded-full`}
            />
          )}
        </div>
      </div>
      <div
        className={`flex flex-col gap-1 border-t p-4 ${collapsed ? "hidden" : ""}`}>
        {filteredContent ? (
          filteredContent.map(({ name, value, type }) => (
            <div key={name + value + type} className="flex items-center">
              <p className="w-[150px] shrink-0">{name}</p>
              {Array.isArray(value) ? (
                <div className="flex flex-col gap-1">
                  {value.map((v, idx) => (
                    <div key={idx} className="flex items-center gap-4">
                      <p>({idx + 1})</p>
                      <p
                        className={`${type === "link" ? "cursor-pointer text-sm font-light text-blue-400 hover:text-blue-300" : "font-light"}`}
                        onClick={
                          type === "link" && v ? openLinkInNewTab(v) : undefined
                        }>
                        {v}
                      </p>
                    </div>
                  ))}
                </div>
              ) : (
                <p
                  className={`${type === "link" ? "cursor-pointer font-light text-blue-400 hover:text-blue-300" : "font-light"}`}
                  onClick={
                    type === "link" && value
                      ? openLinkInNewTab(value)
                      : undefined
                  }>
                  {value}
                </p>
              )}
            </div>
          ))
        ) : (
          <p>{t("No content to display")}</p>
        )}
      </div>
    </div>
  );
};

export { DetailsBox };

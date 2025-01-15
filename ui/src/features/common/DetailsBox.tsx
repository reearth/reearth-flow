import { Download } from "@phosphor-icons/react";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { openLinkInNewTab } from "@flow/utils";

export type DetailsBoxContent = {
  id: string;
  name: string;
  value?: string;
  type?: "link" | "download" | "status";
};

type Props = {
  title: string;
  content?: DetailsBoxContent[];
};

const DetailsBox: React.FC<Props> = ({ title, content }) => {
  const t = useT();
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
      <div className="flex justify-between border-b px-4 py-2">
        <p className="text-xl">{title}</p>
        <div className="flex items-center gap-2">
          {downloadContent?.map((detail) => (
            <Button
              key={detail.id}
              className="p-0"
              variant="outline"
              type="button">
              <a
                className="flex h-full items-center gap-2 rounded px-4 py-2"
                href={detail.value}
                onClick={() => detail.value && handleDownload(detail.value)}>
                <Download />
                <p className="font-light">{detail.name}</p>
              </a>
            </Button>
          ))}
          {status && (
            <div
              className={`${status === "COMPLETED" ? "bg-success" : status === "RUNNING" ? "active-node-status" : status === "FAILED" ? "bg-destructive" : "queued-node-status"} size-4 rounded-full`}
            />
          )}
        </div>
      </div>
      <div className="flex gap-4 p-4">
        {filteredContent ? (
          <>
            <div className="flex flex-col gap-2">
              {filteredContent.map(({ name }) => (
                <p>{name}</p>
              ))}
            </div>
            <div className="flex flex-col gap-2">
              {filteredContent.map(({ value, type }) => (
                <p
                  className={`${type === "link" ? "cursor-pointer font-light text-blue-400 hover:text-blue-300" : "font-light"}`}
                  onClick={
                    type === "link" && value
                      ? openLinkInNewTab(value)
                      : undefined
                  }>
                  {value}
                </p>
              ))}
            </div>
          </>
        ) : (
          <p>{t("No content to display")}</p>
        )}
      </div>
    </div>
  );
};

export { DetailsBox };

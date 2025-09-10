import { DownloadIcon, FolderIcon } from "@phosphor-icons/react";
import { useCallback } from "react";

import {
  ButtonWithTooltip,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type OutputDataItem = {
  url: string;
  name: string;
};

type Props = {
  outputData?: OutputDataItem[];
};

const OutputDataDownload: React.FC<Props> = ({ outputData }) => {
  const t = useT();

  const handleDownload = useCallback((url: string, filename: string) => {
    // Create a temporary anchor element to trigger download
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }, []);

  const handleDownloadAll = useCallback(() => {
    if (!outputData) return;
    
    // Download each file with a small delay to avoid overwhelming the browser
    outputData.forEach((item, index) => {
      setTimeout(() => {
        handleDownload(item.url, item.name);
      }, index * 500); // 500ms delay between downloads
    });
  }, [outputData, handleDownload]);

  if (!outputData || outputData.length === 0) {
    return null;
  }

  const count = outputData.length;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <ButtonWithTooltip
          className="h-[25px] gap-1 px-2 text-xs font-thin hover:bg-primary"
          variant="outline"
          tooltipText={t("Download output files")}
          tooltipOffset={12}
        >
          <FolderIcon size={14} />
          {t("Output data")} ({count})
        </ButtonWithTooltip>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        {count > 1 && (
          <>
            <DropdownMenuItem onClick={handleDownloadAll}>
              <DownloadIcon size={16} />
              {t("Download All")}
            </DropdownMenuItem>
            <div className="my-1 h-px bg-border" />
          </>
        )}
        {outputData.map((item, index) => (
          <DropdownMenuItem
            key={`${item.url}-${index}`}
            onClick={() => handleDownload(item.url, item.name)}
          >
            <DownloadIcon size={16} />
            {item.name}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export default OutputDataDownload;
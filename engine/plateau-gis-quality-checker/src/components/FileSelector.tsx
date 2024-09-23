import { File, Folder } from "@phosphor-icons/react";
import { open } from "@tauri-apps/api/dialog";
import { MouseEvent } from "react";

import { Label } from "./Label";

type Props = {
  label: string;
  selectedPath?: string;
  directorySelect?: boolean;
  onDirectoryToggle?: (isDirectory: boolean) => void;
  onFilePathSelected: (filePath: string) => void;
};

const FileSelector: React.FC<Props> = ({
  label,
  directorySelect,
  selectedPath,
  onDirectoryToggle,
  onFilePathSelected,
}) => {
  const handleSelectFile = async () => {
    // Open the file dialog and allow the user to select a file or directory
    const filePath = await open({
      multiple: false, // Allow single selection
      directory: !!directorySelect, // Set to true if you want to select directories
    });

    if (filePath) {
      onFilePathSelected(filePath as string); // Pass the selected file path to the parent component
    }
  };

  const handleDirectoryToggle = (e: MouseEvent) => {
    if (onDirectoryToggle) {
      e.stopPropagation();
      onFilePathSelected(""); // Reset the selected file path
      onDirectoryToggle?.(!directorySelect);
    }
  };

  return (
    <div className="flex flex-col gap-2 font-thin">
      <Label htmlFor="file">{label}</Label>
      <div className="flex cursor-pointer items-center rounded border" onClick={handleSelectFile}>
        <div className="flex h-[25px] w-[30px] items-center justify-center border-r" onClick={handleDirectoryToggle}>
          {directorySelect ? <Folder /> : <File />}
        </div>
        <div className="flex-1 truncate pl-4 pr-1">
          <p className="truncate text-xs">{selectedPath || "-"}</p>
        </div>
      </div>
    </div>
  );
};

export { FileSelector };

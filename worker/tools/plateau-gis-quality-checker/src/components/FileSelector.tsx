import { File } from "@phosphor-icons/react";
import { open } from "@tauri-apps/api/dialog";
// import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "react";

import { Label } from "./Label";

type Props = {
  label: string;
  directorySelect?: boolean;
  onFilePathSelected: (filePath: string) => void;
};

const FileSelector: React.FC<Props> = ({ label, directorySelect, onFilePathSelected }) => {
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  const handleSelectFile = async () => {
    // Open the file dialog and allow the user to select a file or directory
    const filePath = await open({
      multiple: false, // Allow single selection
      directory: !!directorySelect, // Set to true if you want to select directories
    });

    if (filePath) {
      setSelectedPath(filePath as string); // Save the selected file path
      onFilePathSelected(filePath as string); // Pass the selected file path to the parent component
    }
  };

  return (
    <div className="flex flex-col gap-2 font-thin">
      <Label htmlFor="file">{label}</Label>
      <div className="flex items-center rounded border" onClick={handleSelectFile}>
        <div className="flex h-[25px] w-[30px] items-center justify-center border-r">
          <File />
        </div>
        {selectedPath && (
          <div className="flex-1 truncate pl-4 pr-1">
            <p className="truncate text-xs">{selectedPath}</p>
          </div>
        )}
      </div>
    </div>
  );
};

export { FileSelector };

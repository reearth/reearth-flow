import { Minus, Plus } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { debounce } from "lodash-es";
import { useState } from "react";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogHeader,
  DialogTitle,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  Input,
  Switch,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable, VarType } from "@flow/types";

import { ProjectVariablesTable } from "./ProjectVariablesTable";

type Props = {
  isOpen: boolean;
  currentProjectVariables?: ProjectVariable[];
  onClose: () => void;
  onAdd: (type: VarType) => Promise<void>;
  onChange: (projectVariable: ProjectVariable) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
};

const allVarTypes: VarType[] = [
  "attribute_name",
  "choice",
  "color",
  "coordinate_system",
  "database_connection",
  "datetime",
  "file_folder",
  "geometry",
  "message",
  "number",
  "password",
  "reprojection_file",
  "text",
  "web_connection",
  "yes_no",
  "unsupported",
];

const ProjectVariableDialog: React.FC<Props> = ({
  isOpen,
  currentProjectVariables,
  onClose,
  onAdd,
  onChange,
  onDelete,
}) => {
  const t = useT();
  const [selectedIndex, setSelectedIndex] = useState<number | undefined>();

  // Create a debounced function for handling name changes
  const debouncedNameChange = debounce((rowIndex: number, newName: string) => {
    if (!currentProjectVariables) return;
    const updatedProjectVariable = {
      ...currentProjectVariables[rowIndex],
    };
    updatedProjectVariable.name = newName;
    onChange(updatedProjectVariable);
  }, 500);

  const handleDelete = () => {
    if (selectedIndex === undefined || !currentProjectVariables) return;
    const varToDelete = currentProjectVariables[selectedIndex];
    onDelete(varToDelete.id);
  };

  // const handleMoveUp = () => {
  //   setProjectVariables((pvs) => {
  //     if (selectedIndex !== undefined && selectedIndex > 0) {
  //       const newProjectVariables = [...pvs];
  //       const temp = newProjectVariables[selectedIndex];
  //       newProjectVariables[selectedIndex] =
  //         newProjectVariables[selectedIndex - 1];
  //       newProjectVariables[selectedIndex - 1] = temp;
  //       setSelectedIndex(selectedIndex - 1);
  //       return newProjectVariables;
  //     }
  //     return pvs;
  //   });
  // };

  // const handleMoveDown = () => {
  //   setProjectVariables((pvs) => {
  //     if (selectedIndex !== undefined && selectedIndex < pvs.length - 1) {
  //       const newProjectVariables = [...pvs];
  //       const temp = newProjectVariables[selectedIndex];
  //       newProjectVariables[selectedIndex] =
  //         newProjectVariables[selectedIndex + 1];
  //       newProjectVariables[selectedIndex + 1] = temp;
  //       setSelectedIndex(selectedIndex + 1);
  //       return newProjectVariables;
  //     }
  //     return pvs;
  //   });
  // };

  const getUserFacingName = (type: VarType): string => {
    switch (type) {
      case "attribute_name":
        return t("Attribute Name");
      case "choice":
        return t("Choice");
      case "color":
        return t("Color");
      case "coordinate_system":
        return t("Coordinate System");
      case "database_connection":
        return t("Database Connection");
      case "datetime":
        return t("Date and Time");
      case "file_folder":
        return t("File or Folder");
      case "geometry":
        return t("Geometry");
      case "message":
        return t("Message");
      case "number":
        return t("Number");
      case "password":
        return t("Password");
      case "reprojection_file":
        return t("Reprojection File");
      case "text":
        return t("Text");
      case "web_connection":
        return t("Web Connection");
      case "yes_no":
        return t("Yes/No");
      case "unsupported":
        return t("Unsupported");
      default:
        return t("Unknown");
    }
  };

  const columns: ColumnDef<ProjectVariable>[] = [
    {
      accessorKey: "name",
      header: t("Name"),
      cell: ({ row }) => {
        const value = row.getValue("name") as string;
        return (
          <Input
            key={row.id}
            defaultValue={value}
            onClick={(e) => {
              e.stopPropagation();
            }}
            onFocus={(e) => {
              e.stopPropagation();
            }}
            onChange={(e) => {
              e.stopPropagation();
              debouncedNameChange(row.index, e.currentTarget.value);
            }}
            placeholder={t("Enter name")}
            disabled={false}
          />
        );
      },
    },
    {
      accessorKey: "defaultValue",
      header: t("Default Value"),
    },
    {
      accessorKey: "type",
      header: t("Type"),
      // cell: ({ row }) => {
      //   const value = row.getValue("type") as VarType;
      //   return (
      //     <Select
      //       defaultValue={value}
      //       onValueChange={(newValue) => {
      //         if (!currentProjectVariables) return;
      //         const updatedProjectVariable = currentProjectVariables[row.index];
      //         updatedProjectVariable.type = newValue as VarType;
      //         onChange(updatedProjectVariable);
      //       }}>
      //       <SelectTrigger className="w-[180px]">
      //         <SelectValue placeholder={t("Select type")} />
      //       </SelectTrigger>
      //       <SelectContent>
      //         {allVarTypes.map((type) => (
      //           <SelectItem value={type}>{getUserFacingName(type)}</SelectItem>
      //         ))}
      //       </SelectContent>
      //     </Select>
      //   );
      // },
    },
    {
      accessorKey: "required",
      header: t("Required"),
      cell: ({ row }) => {
        const isChecked = row.getValue("required") as boolean;
        return (
          <Switch
            checked={isChecked}
            onCheckedChange={() => {
              if (!currentProjectVariables) return;
              const projectVar = { ...currentProjectVariables[row.index] };
              projectVar.required = !isChecked;
              console.log(projectVar);
              onChange(projectVar);
            }}
          />
        );
      },
    },
    { accessorKey: "public", header: t("Public"), cell: () => <Switch /> },
  ];

  const handleRowSelect = (projectVar: ProjectVariable) => {
    const index = currentProjectVariables?.findIndex(
      (pv) => pv.id === projectVar.id,
    );
    if (index !== -1) {
      setSelectedIndex(index);
    } else {
      setSelectedIndex(undefined);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="h-[50vh]" size="2xl" position="off-center">
        <div className="flex h-full flex-col">
          <DialogHeader>
            <DialogTitle>{t("Edit Project Variables")}</DialogTitle>
          </DialogHeader>
          <div className="flex h-full">
            <DialogContentSection className="flex-3 bg-card">
              <DialogContentSection className="flex flex-row items-center gap-2 p-2">
                <DropdownMenu>
                  <DropdownMenuTrigger>
                    <IconButton icon={<Plus />} />
                  </DropdownMenuTrigger>
                  <DropdownMenuContent>
                    {allVarTypes.map((type) => (
                      <DropdownMenuItem
                        key={type}
                        onClick={() => {
                          onAdd(type);
                        }}>
                        {getUserFacingName(type)}
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
                <IconButton icon={<Minus />} onClick={handleDelete} />
                {/* <IconButton icon={<ArrowUp />} onClick={handleMoveUp} /> */}
                {/* <IconButton icon={<ArrowDown />} onClick={handleMoveDown} /> */}
              </DialogContentSection>
              <DialogContentSection>
                <ProjectVariablesTable
                  projectVariables={currentProjectVariables ?? []}
                  columns={columns}
                  selectedRow={selectedIndex}
                  onRowClick={handleRowSelect}
                />
              </DialogContentSection>
            </DialogContentSection>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default ProjectVariableDialog;

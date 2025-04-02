import { ArrowDown, ArrowUp, Minus, Plus } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  IconButton,
  Switch,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable } from "@flow/types";
import { generateUUID } from "@flow/utils";

import { ProjectVariablesTable } from "./ProjectVariablesTable";

type Props = {
  isOpen: boolean;
  currentProjectVariable?: ProjectVariable[];
  onClose: () => void;
  onSubmit: (newProjectVariables: ProjectVariable[]) => void;
};

const ProjectVariableDialog: React.FC<Props> = ({
  isOpen,
  currentProjectVariable,
  onClose,
  onSubmit,
}) => {
  const t = useT();
  const [projectVariables, setProjectVariables] = useState<ProjectVariable[]>(
    currentProjectVariable ?? [],
  );

  const [selectedIndex, setSelectedIndex] = useState<number | undefined>();

  const handleAdd = () => {
    setProjectVariables((pvs) => {
      if (selectedIndex !== undefined) {
        const newProjectVariables = [...pvs];
        newProjectVariables.splice(selectedIndex + 1, 0, {
          id: generateUUID(),
          name: "",
          value: "asldfkj",
          type: "text",
          required: false,
        });
        return newProjectVariables;
      }
      return [
        ...pvs,
        {
          id: generateUUID(),
          name: "",
          value: "dddd",
          type: "text",
          required: false,
        },
      ];
    });
  };

  const handleDelete = () => {
    setProjectVariables((pvs) => {
      if (selectedIndex !== undefined) {
        const newProjectVariables = [...pvs];
        newProjectVariables.splice(selectedIndex, 1);
        setSelectedIndex(undefined);
        return newProjectVariables;
      }
      return pvs;
    });
  };

  const handleMoveUp = () => {
    setProjectVariables((pvs) => {
      if (selectedIndex !== undefined && selectedIndex > 0) {
        const newProjectVariables = [...pvs];
        const temp = newProjectVariables[selectedIndex];
        newProjectVariables[selectedIndex] =
          newProjectVariables[selectedIndex - 1];
        newProjectVariables[selectedIndex - 1] = temp;
        setSelectedIndex(selectedIndex - 1);
        return newProjectVariables;
      }
      return pvs;
    });
  };

  const handleMoveDown = () => {
    setProjectVariables((pvs) => {
      if (selectedIndex !== undefined && selectedIndex < pvs.length - 1) {
        const newProjectVariables = [...pvs];
        const temp = newProjectVariables[selectedIndex];
        newProjectVariables[selectedIndex] =
          newProjectVariables[selectedIndex + 1];
        newProjectVariables[selectedIndex + 1] = temp;
        setSelectedIndex(selectedIndex + 1);
        return newProjectVariables;
      }
      return pvs;
    });
  };

  const handleClose = () => {
    setProjectVariables(currentProjectVariable ?? []);
    onClose();
  };

  const handleSubmit = () => onSubmit(projectVariables);

  const columns: ColumnDef<ProjectVariable>[] = [
    {
      accessorKey: "name",
      header: t("Name"),
      // cell: ({ getValue }) => formatTimestamp(getValue<string>()),
    },
    {
      accessorKey: "value",
      header: t("Value"),
    },
    {
      accessorKey: "type",
      header: t("Type"),
      // cell: ({ row }) => {
      //   const value = row.getValue("type");
      //   return (
      //     <Select
      //       defaultValue={value}
      //       onValueChange={(newValue) => {
      //         const newProjectVariables = [...projectVariables];
      //         newProjectVariables[row.index].type = newValue;
      //         setProjectVariables(newProjectVariables);
      //       }}
      //     >
      //       <SelectTrigger className="w-[180px]">
      //         <SelectValue placeholder={t("Select type")} />
      //       </SelectTrigger>
      //       <SelectContent>
      //         <SelectItem value="text">Text</SelectItem>
      //         <SelectItem value="number">Number</SelectItem>
      //         <SelectItem value="boolean">Boolean</SelectItem>
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
              const newProjectVariables = [...projectVariables];
              newProjectVariables[row.index].required = !isChecked;
              setProjectVariables(newProjectVariables);
            }}
          />
        );
      },
    },
    { accessorKey: "public", header: t("Public"), cell: () => <Switch /> },
  ];

  const handleRowSelect = (projectVar: ProjectVariable) => {
    const index = projectVariables.findIndex((pv) => pv.id === projectVar.id);
    if (index !== -1) {
      setSelectedIndex(index);
    } else {
      setSelectedIndex(undefined);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="h-[50vh]" size="2xl" position="off-center">
        <div className="flex h-full flex-col">
          <DialogHeader>
            <DialogTitle>{t("Edit Project Variables")}</DialogTitle>
          </DialogHeader>
          <DialogContentWrapper className="flex-1">
            <DialogContentSection className="flex flex-row items-center gap-2">
              <IconButton icon={<Plus />} onClick={handleAdd} />
              <IconButton icon={<Minus />} onClick={handleDelete} />
              <IconButton icon={<ArrowUp />} onClick={handleMoveUp} />
              <IconButton icon={<ArrowDown />} onClick={handleMoveDown} />
            </DialogContentSection>
            <DialogContentSection>
              <ProjectVariablesTable
                projectVariables={projectVariables}
                columns={columns}
                selectedRow={selectedIndex}
                onRowClick={handleRowSelect}
              />
            </DialogContentSection>
          </DialogContentWrapper>
          <DialogFooter className="flex justify-self-end">
            {/* <Button
              disabled={buttonDisabled}
              variant={"outline"}
              onClick={() => setEditProject(undefined)}
              >
              {t("Cancel")}
              </Button> */}
            <Button onClick={handleSubmit}>{t("Save")}</Button>
          </DialogFooter>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export { ProjectVariableDialog };

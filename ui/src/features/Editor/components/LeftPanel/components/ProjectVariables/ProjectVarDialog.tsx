import {
  ArrowDown,
  ArrowUp,
  DotsSixVertical,
  Minus,
  Plus,
} from "@phosphor-icons/react";
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
  Input,
  Label,
  ScrollArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { ProjectVar } from "@flow/types";
import { randomID } from "@flow/utils";

type Props = {
  isOpen: boolean;
  currentProjectVars: ProjectVar[];
  onClose: () => void;
  onSubmit: (newProjectVars: ProjectVar[]) => void;
};

const ProjectVarDialog: React.FC<Props> = ({
  isOpen,
  currentProjectVars,
  onClose,
  onSubmit,
}) => {
  const t = useT();
  const [projectVariables, setProjectVariables] =
    useState<ProjectVar[]>(currentProjectVars);

  const [selectedIndex, setSelectedIndex] = useState<number | undefined>();

  const handleAdd = () => {
    setProjectVariables((pvs) => {
      if (selectedIndex !== undefined) {
        const newProjectVars = [...pvs];
        newProjectVars.splice(selectedIndex + 1, 0, {
          id: randomID(10),
          name: "",
          definition: "",
          type: "string",
          required: false,
        });
        return newProjectVars;
      }
      return [
        ...pvs,
        {
          id: randomID(10),
          name: "",
          definition: "",
          type: "string",
          required: false,
        },
      ];
    });
  };

  const handleDelete = () => {
    setProjectVariables((pvs) => {
      if (selectedIndex !== undefined) {
        const newProjectVars = [...pvs];
        newProjectVars.splice(selectedIndex, 1);
        setSelectedIndex(undefined);
        return newProjectVars;
      }
      return pvs;
    });
  };

  const handleMoveUp = () => {
    setProjectVariables((pvs) => {
      if (selectedIndex !== undefined && selectedIndex > 0) {
        const newProjectVars = [...pvs];
        const temp = newProjectVars[selectedIndex];
        newProjectVars[selectedIndex] = newProjectVars[selectedIndex - 1];
        newProjectVars[selectedIndex - 1] = temp;
        setSelectedIndex(selectedIndex - 1);
        return newProjectVars;
      }
      return pvs;
    });
  };

  const handleMoveDown = () => {
    setProjectVariables((pvs) => {
      if (selectedIndex !== undefined && selectedIndex < pvs.length - 1) {
        const newProjectVars = [...pvs];
        const temp = newProjectVars[selectedIndex];
        newProjectVars[selectedIndex] = newProjectVars[selectedIndex + 1];
        newProjectVars[selectedIndex + 1] = temp;
        setSelectedIndex(selectedIndex + 1);
        return newProjectVars;
      }
      return pvs;
    });
  };

  const handleClose = () => {
    setProjectVariables(currentProjectVars);
    onClose();
  };

  const handleSubmit = () => onSubmit(projectVariables);

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="h-[50vh]" size="xl" position="off-center">
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
              <div className="flex">
                <Label className="flex-1">{t("Key")}</Label>
                <Label className="flex-1">{t("Value")}</Label>
              </div>
              <ScrollArea>
                <div className="flex flex-1 flex-col gap-1">
                  {projectVariables.map((variable, idx) => (
                    <div
                      key={variable.id}
                      className={`flex gap-2 rounded p-1 hover:bg-primary ${idx === selectedIndex && "bg-primary"}`}
                      onClick={() =>
                        setSelectedIndex((sidx) =>
                          sidx === idx ? undefined : idx,
                        )
                      }>
                      <div className="flex items-center">
                        <DotsSixVertical />
                      </div>
                      <Input
                        value={variable.name}
                        onClick={(e) => e.stopPropagation()}
                        onChange={(e) => {
                          setProjectVariables((pvs) => {
                            const newPvs = [...pvs];
                            const newValue = e.target.value;
                            newPvs[idx].name = newValue.split(/\s+/).join(""); // Don't allow white space in the name
                            return newPvs;
                          });
                        }}
                      />
                      <Input
                        type="text"
                        value={variable.definition}
                        onClick={(e) => e.stopPropagation()}
                        onChange={(e) => {
                          setProjectVariables((pvs) => {
                            const newPvs = [...pvs];
                            newPvs[idx].definition = e.target.value;
                            return newPvs;
                          });
                        }}
                      />
                    </div>
                  ))}
                </div>
              </ScrollArea>
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

export { ProjectVarDialog };

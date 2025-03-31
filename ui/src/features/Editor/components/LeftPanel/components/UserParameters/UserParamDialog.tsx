import { ArrowDown, ArrowUp, Minus, Plus } from "@phosphor-icons/react";
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
  Switch,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { UserParameter } from "@flow/types";
import { generateUUID } from "@flow/utils";

type Props = {
  isOpen: boolean;
  currentUserParameters?: UserParameter[];
  onClose: () => void;
  onSubmit: (newUserParams: UserParameter[]) => void;
};

const UserParameterDialog: React.FC<Props> = ({
  isOpen,
  currentUserParameters,
  onClose,
  onSubmit,
}) => {
  const t = useT();
  const [userParameters, setUserParameters] = useState<UserParameter[]>(
    currentUserParameters ?? [],
  );

  const [selectedIndex, setSelectedIndex] = useState<number | undefined>();

  const handleAdd = () => {
    setUserParameters((pvs) => {
      if (selectedIndex !== undefined) {
        const newUserParams = [...pvs];
        newUserParams.splice(selectedIndex + 1, 0, {
          id: generateUUID(),
          name: "",
          value: "asldfkj",
          type: "text",
          required: false,
        });
        return newUserParams;
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
    setUserParameters((pvs) => {
      if (selectedIndex !== undefined) {
        const newUserParams = [...pvs];
        newUserParams.splice(selectedIndex, 1);
        setSelectedIndex(undefined);
        return newUserParams;
      }
      return pvs;
    });
  };

  const handleMoveUp = () => {
    setUserParameters((pvs) => {
      if (selectedIndex !== undefined && selectedIndex > 0) {
        const newUserParams = [...pvs];
        const temp = newUserParams[selectedIndex];
        newUserParams[selectedIndex] = newUserParams[selectedIndex - 1];
        newUserParams[selectedIndex - 1] = temp;
        setSelectedIndex(selectedIndex - 1);
        return newUserParams;
      }
      return pvs;
    });
  };

  const handleMoveDown = () => {
    setUserParameters((pvs) => {
      if (selectedIndex !== undefined && selectedIndex < pvs.length - 1) {
        const newUserParams = [...pvs];
        const temp = newUserParams[selectedIndex];
        newUserParams[selectedIndex] = newUserParams[selectedIndex + 1];
        newUserParams[selectedIndex + 1] = temp;
        setSelectedIndex(selectedIndex + 1);
        return newUserParams;
      }
      return pvs;
    });
  };

  const handleClose = () => {
    setUserParameters(userParameters);
    onClose();
  };

  const handleSubmit = () => onSubmit(userParameters);

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
              <div className="flex">
                <Label className="flex-1">{t("Key")}</Label>
                <Label className="flex-1">{t("Value")}</Label>
                <Label className="flex-1">{t("Type")}</Label>
                <Label className="flex-1">{t("Required")}</Label>
              </div>
              <ScrollArea>
                <div className="flex flex-1 flex-col gap-1">
                  {userParameters.map((param, idx) => (
                    <div
                      key={param.id}
                      className={`flex gap-2 rounded p-1 hover:bg-primary ${idx === selectedIndex && "bg-primary"}`}
                      onClick={() =>
                        setSelectedIndex((sidx) =>
                          sidx === idx ? undefined : idx,
                        )
                      }>
                      {/* <div className="flex items-center">
                        <DotsSixVertical />
                      </div> */}
                      <Input
                        value={param.name}
                        onClick={(e) => e.stopPropagation()}
                        onChange={(e) => {
                          setUserParameters((pvs) => {
                            const newPvs = [...pvs];
                            const newValue = e.target.value;
                            newPvs[idx].name = newValue.split(/\s+/).join(""); // Don't allow white space in the name
                            return newPvs;
                          });
                        }}
                      />
                      <Input
                        type="text"
                        value={param.value}
                        onClick={(e) => e.stopPropagation()}
                        onChange={(e) => {
                          setUserParameters((pvs) => {
                            const newPvs = [...pvs];
                            newPvs[idx].value = e.target.value;
                            return newPvs;
                          });
                        }}
                      />
                      <div className="w-full">
                        <p>{param.type}</p>
                      </div>
                      <div className="w-full">
                        <Switch />
                      </div>
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

export { UserParameterDialog };

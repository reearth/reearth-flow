import { useState, useEffect, useCallback } from "react";

import { useT } from "@flow/lib/i18n";
import { ProjectVariable, VarType } from "@flow/types";
import { generateUUID, getDefaultValueForProjectVar } from "@flow/utils";

type PendingChange =
  | {
      type: "add";
      tempId: string;
      projectVariable: ProjectVariable;
    }
  | {
      type: "update";
      projectVariable: ProjectVariable;
    }
  | {
      type: "delete";
      id: string;
    }
  | {
      type: "reorder";
      paramId: string;
      newIndex: number;
    };

export default ({
  isOpen,
  currentProjectVariables,
  projectId,
  onClose,
  onAdd,
  onChange,
  onDelete,
  onDeleteBatch,
  onBatchUpdate,
}: {
  isOpen: boolean;
  currentProjectVariables?: ProjectVariable[];
  projectId?: string;
  onClose: () => void;
  onAdd: (projectVariable: ProjectVariable) => Promise<void>;
  onChange: (projectVariable: ProjectVariable) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onDeleteBatch?: (ids: string[]) => Promise<void>;
  onBatchUpdate?: (input: {
    projectId: string;
    creates?: {
      name: string;
      defaultValue: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
      index?: number;
    }[];
    updates?: {
      paramId: string;
      name?: string;
      defaultValue?: any;
      type?: VarType;
      required?: boolean;
      publicValue?: boolean;
    }[];
    deletes?: string[];
    reorders?: {
      paramId: string;
      newIndex: number;
    }[];
  }) => Promise<void>;
}) => {
  const t = useT();
  const [selectedIndices, setSelectedIndices] = useState<number[]>([]);
  const [localProjectVariables, setLocalProjectVariables] = useState<
    ProjectVariable[]
  >([]);
  const [pendingChanges, setPendingChanges] = useState<PendingChange[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const getUserFacingName = useCallback(
    (type: VarType): string => {
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
    },
    [t],
  );

  useEffect(() => {
    if (currentProjectVariables) {
      setLocalProjectVariables([...currentProjectVariables]);
      setPendingChanges([]);
      setSelectedIndices([]);
    }
  }, [currentProjectVariables, isOpen, getUserFacingName]);

  const handleLocalAdd = useCallback(
    (type: VarType) => {
      const tempId = `temp_${generateUUID()}`;
      const newVariable: ProjectVariable = {
        id: tempId,
        name: t("New Project Variable"),
        defaultValue: getDefaultValueForProjectVar(type),
        type,
        required: true,
        public: true,
      };

      setLocalProjectVariables((prev) => [...prev, newVariable]);
      setPendingChanges((prev) => [
        ...prev,
        { type: "add", tempId, projectVariable: newVariable },
      ]);
    },
    [t],
  );

  const handleLocalUpdate = useCallback((updatedVariable: ProjectVariable) => {
    setLocalProjectVariables((prev) =>
      prev.map((variable) =>
        variable.id === updatedVariable.id ? updatedVariable : variable,
      ),
    );

    // Handle pending changes differently for new vs existing variables
    setPendingChanges((prev) => {
      // If this is a new variable (temp ID), update the "add" change
      if (updatedVariable.id.startsWith("temp_")) {
        const existingAddIndex = prev.findIndex(
          (change) =>
            change.type === "add" && change.tempId === updatedVariable.id,
        );

        if (existingAddIndex >= 0) {
          const newChanges = [...prev];
          newChanges[existingAddIndex] = {
            type: "add",
            tempId: updatedVariable.id,
            projectVariable: updatedVariable,
          };
          return newChanges;
        }
      } else {
        // For existing variables, handle as update
        const existingUpdateIndex = prev.findIndex(
          (change) =>
            change.type === "update" &&
            change.projectVariable.id === updatedVariable.id,
        );

        if (existingUpdateIndex >= 0) {
          const newChanges = [...prev];
          newChanges[existingUpdateIndex] = {
            type: "update",
            projectVariable: updatedVariable,
          };
          return newChanges;
        } else {
          return [
            ...prev,
            { type: "update", projectVariable: updatedVariable },
          ];
        }
      }

      return prev;
    });
  }, []);

  const handleLocalDelete = useCallback(() => {
    if (selectedIndices.length === 0 || !localProjectVariables) return;

    const varsToDelete = selectedIndices.map(
      (index) => localProjectVariables[index],
    );

    setLocalProjectVariables((prev) =>
      prev.filter((_, index) => !selectedIndices.includes(index)),
    );

    varsToDelete.forEach((varToDelete) => {
      if (!varToDelete.id.startsWith("temp_")) {
        setPendingChanges((prev) => [
          ...prev,
          { type: "delete", id: varToDelete.id },
        ]);
      } else {
        setPendingChanges((prev) =>
          prev.filter(
            (change) =>
              !(change.type === "add" && change.tempId === varToDelete.id),
          ),
        );
      }
    });

    setSelectedIndices([]);
  }, [selectedIndices, localProjectVariables]);

  const handleMoveUp = useCallback(
    (index: number) => {
      if (index <= 0) return;

      const newProjectVariables = [...localProjectVariables];
      const temp = newProjectVariables[index];
      newProjectVariables[index] = newProjectVariables[index - 1];
      newProjectVariables[index - 1] = temp;

      setLocalProjectVariables(newProjectVariables);

      // Track reorder for non-temp variables
      if (!temp.id.startsWith("temp_")) {
        setPendingChanges((prev) => [
          ...prev,
          { type: "reorder", paramId: temp.id, newIndex: index - 1 },
        ]);
      }
      // Also track reorder for the swapped variable if it's not temp
      const swappedVar = newProjectVariables[index];
      if (!swappedVar.id.startsWith("temp_")) {
        setPendingChanges((prev) => [
          ...prev,
          { type: "reorder", paramId: swappedVar.id, newIndex: index },
        ]);
      }
    },
    [localProjectVariables],
  );

  const handleMoveDown = useCallback(
    (index: number) => {
      if (index >= localProjectVariables.length - 1) return;

      const newProjectVariables = [...localProjectVariables];
      const temp = newProjectVariables[index];
      newProjectVariables[index] = newProjectVariables[index + 1];
      newProjectVariables[index + 1] = temp;

      setLocalProjectVariables(newProjectVariables);

      // Track reorder for non-temp variables
      if (!temp.id.startsWith("temp_")) {
        setPendingChanges((prev) => [
          ...prev,
          { type: "reorder", paramId: temp.id, newIndex: index + 1 },
        ]);
      }
      // Also track reorder for the swapped variable if it's not temp
      const swappedVar = newProjectVariables[index];
      if (!swappedVar.id.startsWith("temp_")) {
        setPendingChanges((prev) => [
          ...prev,
          { type: "reorder", paramId: swappedVar.id, newIndex: index },
        ]);
      }
    },
    [localProjectVariables],
  );

  const handleSubmit = useCallback(async () => {
    setIsSubmitting(true);
    try {
      const addChanges = pendingChanges.filter(
        (change) => change.type === "add",
      );
      const updateChanges = pendingChanges.filter(
        (change) => change.type === "update",
      );
      const deleteChanges = pendingChanges.filter(
        (change) => change.type === "delete",
      );
      const reorderChanges = pendingChanges.filter(
        (change) => change.type === "reorder",
      );

      if (
        onBatchUpdate &&
        projectId &&
        (addChanges.length > 0 ||
          updateChanges.length > 0 ||
          deleteChanges.length > 0 ||
          reorderChanges.length > 0)
      ) {
        const creates = addChanges.map((change) => ({
          name: change.projectVariable.name,
          defaultValue: change.projectVariable.defaultValue,
          type: change.projectVariable.type,
          required: change.projectVariable.required,
          publicValue: change.projectVariable.public,
          index: localProjectVariables.length,
        }));

        const updates = updateChanges.map((change) => ({
          paramId: change.projectVariable.id,
          name: change.projectVariable.name,
          defaultValue: change.projectVariable.defaultValue,
          type: change.projectVariable.type,
          required: change.projectVariable.required,
          publicValue: change.projectVariable.public,
        }));

        const deletes = deleteChanges.map((change) => change.id);

        const reorders = reorderChanges.map((change) => ({
          paramId: change.paramId,
          newIndex: change.newIndex,
        }));

        await onBatchUpdate({
          projectId,
          ...(creates.length > 0 && { creates }),
          ...(updates.length > 0 && { updates }),
          ...(deletes.length > 0 && { deletes }),
          ...(reorders.length > 0 && { reorders }),
        });
      } else {
        for (const change of addChanges) {
          await onAdd(change.projectVariable);
        }

        for (const change of updateChanges) {
          await onChange(change.projectVariable);
        }

        if (deleteChanges.length > 0) {
          const deleteIds = deleteChanges.map((change) => change.id);

          if (onDeleteBatch && deleteChanges.length > 1) {
            await onDeleteBatch(deleteIds);
          } else {
            for (const change of deleteChanges) {
              await onDelete(change.id);
            }
          }
        }
      }

      await new Promise((resolve) => setTimeout(resolve, 100));

      setPendingChanges([]);
      onClose();
    } catch (error) {
      console.error("Failed to submit project variable changes:", error);
    } finally {
      setIsSubmitting(false);
    }
  }, [
    pendingChanges,
    onBatchUpdate,
    projectId,
    localProjectVariables.length,
    onAdd,
    onChange,
    onDeleteBatch,
    onDelete,
    onClose,
  ]);

  const handleCancel = useCallback(() => {
    if (currentProjectVariables) {
      setLocalProjectVariables([...currentProjectVariables]);
    }
    setPendingChanges([]);
    setSelectedIndices([]);
    onClose();
  }, [currentProjectVariables, onClose]);

  const handleRowSelect = useCallback((selectedIndicesFromTable: number[]) => {
    setSelectedIndices(selectedIndicesFromTable);
  }, []);

  return {
    selectedIndices,
    localProjectVariables,
    pendingChanges,
    isSubmitting,
    getUserFacingName,
    handleLocalAdd,
    handleLocalUpdate,
    handleLocalDelete,
    handleMoveUp,
    handleMoveDown,
    handleSubmit,
    handleCancel,
    handleRowSelect,
  };
};

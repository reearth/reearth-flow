import { useEffect, useState } from "react";

import { useWorkerConfig } from "@flow/lib/gql/workerConfig";
import { MachineTypeOption } from "@flow/types";

export default () => {
  const { useGetWorkerConfig, updateWorkerConfig, deleteWorkerConfig } =
    useWorkerConfig();

  const { workerConfig, isLoading } = useGetWorkerConfig();
  const [formData, setFormData] = useState({
    machineType: "",
    computeCpuMilli: "",
    computeMemoryMib: "",
    bootDiskSizeGB: "",
    taskCount: "",
    maxConcurrency: "",
    threadPoolSize: "",
    channelBufferSize: "",
    featureFlushThreshold: "",
    nodeStatusPropagationDelayMilli: "",
  });

  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);

  useEffect(() => {
    if (workerConfig && !isLoading) {
      setFormData({
        machineType: workerConfig.machineType ?? "",
        computeCpuMilli: workerConfig.computeCpuMilli?.toString() ?? "",
        computeMemoryMib: workerConfig.computeMemoryMib?.toString() ?? "",
        bootDiskSizeGB: workerConfig.bootDiskSizeGB?.toString() ?? "",
        taskCount: workerConfig.taskCount?.toString() ?? "",
        maxConcurrency: workerConfig.maxConcurrency?.toString() ?? "",
        threadPoolSize: workerConfig.threadPoolSize?.toString() ?? "",
        channelBufferSize: workerConfig.channelBufferSize?.toString() ?? "",
        featureFlushThreshold:
          workerConfig.featureFlushThreshold?.toString() ?? "",
        nodeStatusPropagationDelayMilli:
          workerConfig.nodeStatusPropagationDelayMilli?.toString() ?? "",
      });
    }
  }, [workerConfig, isLoading, formData.machineType]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const input = {
      machineType: formData.machineType || undefined,
      computeCpuMilli: formData.computeCpuMilli
        ? parseInt(formData.computeCpuMilli)
        : undefined,
      computeMemoryMib: formData.computeMemoryMib
        ? parseInt(formData.computeMemoryMib)
        : undefined,
      bootDiskSizeGB: formData.bootDiskSizeGB
        ? parseInt(formData.bootDiskSizeGB)
        : undefined,
      taskCount: formData.taskCount ? parseInt(formData.taskCount) : undefined,
      maxConcurrency: formData.maxConcurrency
        ? parseInt(formData.maxConcurrency)
        : undefined,
      threadPoolSize: formData.threadPoolSize
        ? parseInt(formData.threadPoolSize)
        : undefined,
      channelBufferSize: formData.channelBufferSize
        ? parseInt(formData.channelBufferSize)
        : undefined,
      featureFlushThreshold: formData.featureFlushThreshold
        ? parseInt(formData.featureFlushThreshold)
        : undefined,
      nodeStatusPropagationDelayMilli: formData.nodeStatusPropagationDelayMilli
        ? parseInt(formData.nodeStatusPropagationDelayMilli)
        : undefined,
    };

    await updateWorkerConfig(input);
  };

  const handleChange = (field: string, value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  };

  const handleDelete = async () => {
    await deleteWorkerConfig();
    setIsDeleteDialogOpen(false);
  };

  const machineTypeOptions = [
    MachineTypeOption.E2_STANDARD_2,
    MachineTypeOption.E2_STANDARD_4,
    MachineTypeOption.E2_STANDARD_8,
    MachineTypeOption.E2_STANDARD_16,
    MachineTypeOption.E2_HIGH_MEM_2,
    MachineTypeOption.E2_HIGH_MEM_4,
    MachineTypeOption.E2_HIGH_MEM_8,
    MachineTypeOption.E2_HIGH_MEM_16,
    MachineTypeOption.E2_HIGH_CPU_2,
    MachineTypeOption.E2_HIGH_CPU_4,
    MachineTypeOption.E2_HIGH_CPU_8,
    MachineTypeOption.E2_HIGH_CPU_16,
    MachineTypeOption.N2_STANDARD_2,
    MachineTypeOption.N2_STANDARD_4,
    MachineTypeOption.N2_STANDARD_8,
    MachineTypeOption.N2_STANDARD_16,
  ];

  return {
    workerConfig,
    isLoading,
    formData,
    machineTypeOptions,
    isDeleteDialogOpen,
    setIsDeleteDialogOpen,
    handleChange,
    handleDelete,
    handleSubmit,
    updateWorkerConfig,
    deleteWorkerConfig,
  };
};

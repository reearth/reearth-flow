import { useEffect, useState } from "react";

import { useWorkerConfig } from "@flow/lib/gql/workerConfig";

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
    if (workerConfig) {
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
  }, [workerConfig]);

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

  return {
    workerConfig,
    isLoading,
    formData,
    isDeleteDialogOpen,
    setIsDeleteDialogOpen,
    handleChange,
    handleDelete,
    handleSubmit,
    updateWorkerConfig,
    deleteWorkerConfig,
  };
};

import { FC } from "react";

import {
  Button,
  Input,
  Label,
  LoadingSkeleton,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";

import { WorkerConfigDeletionDialog } from "./components";
import useHooks from "./hooks";

const Dev: FC = () => {
  const {
    workerConfig,
    isLoading,
    formData,
    machineTypeOptions,
    isDeleteDialogOpen,
    setIsDeleteDialogOpen,
    handleChange,
    handleDelete,
    handleSubmit,
  } = useHooks();

  if (isLoading) {
    return (
      <div className="flex h-screen w-full flex-col items-center justify-center gap-6 p-6">
        <LoadingSkeleton className="h-8 w-1/3" />
      </div>
    );
  }

  return (
    <div className="flex w-full flex-col gap-6 p-6">
      <div className="flex flex-col gap-4">
        <h4 className="text-lg font-extralight">Worker Configuration</h4>
        <p className="text-sm">
          Configure worker settings for workflow execution. These settings
          control the compute resources and behavior of workflow workers.
        </p>
        <div className="h-px bg-gray-200" />
      </div>
      <form onSubmit={handleSubmit} className="space-y-6">
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="machineType" className="text-sm font-medium">
              Machine Type
            </Label>
            <Select
              key={formData.machineType ?? "empty"}
              value={formData.machineType}
              onValueChange={(value) => handleChange("machineType", value)}>
              <SelectTrigger>
                <SelectValue placeholder="Select machine type" />
              </SelectTrigger>
              <SelectContent>
                {machineTypeOptions.map((value) => (
                  <SelectItem key={value} value={value}>
                    {value}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              The GCP machine type for the worker (e.g., e2-standard-2)
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="computeCpuMilli" className="text-sm font-medium">
              Compute CPU (milli-cores)
            </Label>
            <Input
              id="computeCpuMilli"
              type="number"
              value={formData.computeCpuMilli}
              min={500}
              max={64000}
              onChange={(e) => handleChange("computeCpuMilli", e.target.value)}
              placeholder="e.g., 4000"
            />
            <p className="text-xs text-muted-foreground">
              CPU allocation in milli-cores (1000 = 1 CPU core). Must be at
              least 500 and no larger than 64000.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="computeMemoryMib" className="text-sm font-medium">
              Compute Memory (MiB)
            </Label>
            <Input
              id="computeMemoryMib"
              type="number"
              value={formData.computeMemoryMib}
              min={512}
              max={131072}
              onChange={(e) => handleChange("computeMemoryMib", e.target.value)}
              placeholder="e.g., 8192"
            />
            <p className="text-xs text-muted-foreground">
              Memory allocation in megabytes (1024 MiB = 1 GiB). Must be at
              least 512 and no larger than 131072.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="bootDiskSizeGB" className="text-sm font-medium">
              Boot Disk Size (GB)
            </Label>
            <Input
              id="bootDiskSizeGB"
              type="number"
              value={formData.bootDiskSizeGB}
              min={10}
              max={1000}
              onChange={(e) => handleChange("bootDiskSizeGB", e.target.value)}
              placeholder="e.g., 100"
            />
            <p className="text-xs text-muted-foreground">
              Boot disk size in gigabytes. Must be at least 10 GB and no larger
              than 1000 GB.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="taskCount" className="text-sm font-medium">
              Task Count
            </Label>
            <Input
              id="taskCount"
              type="number"
              value={formData.taskCount}
              min={1}
              max={20}
              onChange={(e) => handleChange("taskCount", e.target.value)}
              placeholder="e.g., 10"
            />
            <p className="text-xs text-muted-foreground">
              Number of parallel tasks. Must be at least 1 and no larger than
              20.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="maxConcurrency" className="text-sm font-medium">
              Max Concurrency
            </Label>
            <Input
              id="maxConcurrency"
              type="number"
              value={formData.maxConcurrency}
              min={1}
              max={64}
              onChange={(e) => handleChange("maxConcurrency", e.target.value)}
              placeholder="e.g., 4"
            />
            <p className="text-xs text-muted-foreground">
              Maximum concurrent operations. Must be at least 1 and no larger
              than 64.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="threadPoolSize" className="text-sm font-medium">
              Thread Pool Size
            </Label>
            <Input
              id="threadPoolSize"
              type="number"
              value={formData.threadPoolSize}
              min={1}
              max={200}
              onChange={(e) => handleChange("threadPoolSize", e.target.value)}
              placeholder="e.g., 8"
            />
            <p className="text-xs text-muted-foreground">
              Size of the worker thread pool. Must be at least 1 and no larger
              than 200.
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="channelBufferSize" className="text-sm font-medium">
              Channel Buffer Size
            </Label>
            <Input
              id="channelBufferSize"
              type="number"
              value={formData.channelBufferSize}
              min={1}
              max={8192}
              onChange={(e) =>
                handleChange("channelBufferSize", e.target.value)
              }
              placeholder="e.g., 1000"
            />
            <p className="text-xs text-muted-foreground">
              Internal channel buffer size. Must be at least 1 and no larger
              than 8192.
            </p>
          </div>

          <div className="space-y-2">
            <Label
              htmlFor="featureFlushThreshold"
              className="text-sm font-medium">
              Feature Flush Threshold
            </Label>
            <Input
              id="featureFlushThreshold"
              type="number"
              value={formData.featureFlushThreshold}
              min={1}
              max={20000}
              onChange={(e) =>
                handleChange("featureFlushThreshold", e.target.value)
              }
              placeholder="e.g., 1000"
            />
            <p className="text-xs text-muted-foreground">
              Number of features before flushing to storage. Must be at least 1
              and no larger than 20000.
            </p>
          </div>

          <div className="space-y-2">
            <Label
              htmlFor="nodeStatusPropagationDelayMilli"
              className="text-sm font-medium">
              Node Status Delay (ms)
            </Label>
            <Input
              id="nodeStatusPropagationDelayMilli"
              type="number"
              value={formData.nodeStatusPropagationDelayMilli}
              min={50}
              max={30000}
              onChange={(e) =>
                handleChange("nodeStatusPropagationDelayMilli", e.target.value)
              }
              placeholder="e.g., 100"
            />
            <p className="text-xs text-muted-foreground">
              Delay in milliseconds for node status propagation. Must be at
              least 50 and no larger than 30000.
            </p>
          </div>
        </div>

        <div className="flex justify-between gap-4">
          <Button
            type="button"
            variant="destructive"
            disabled={!workerConfig}
            onClick={() => setIsDeleteDialogOpen(true)}>
            Delete Configuration
          </Button>

          <Button type="submit" variant="default">
            Update Configuration
          </Button>
        </div>
      </form>
      {isDeleteDialogOpen && (
        <WorkerConfigDeletionDialog
          isDeleteDialogOpen={isDeleteDialogOpen}
          setIsDeleteDialogOpen={setIsDeleteDialogOpen}
          onWorkerConfigDelete={handleDelete}
        />
      )}
      {workerConfig && (
        <div className="mt-4 rounded-md bg-muted p-4">
          <p className="text-xs text-muted-foreground">
            Last updated: {new Date(workerConfig.updatedAt).toLocaleString()}
          </p>
        </div>
      )}
    </div>
  );
};

export default Dev;

const LoadingDots = () => {
  // Define the background classes in the desired order:
  const dotColors = [
    "bg-node-reader/60",
    "bg-node-transformer/60",
    "bg-node-writer/60",
    "bg-zinc-600",
    // "bg-primary/60",
    "bg-node-subworkflow/60",
  ];

  const delayClasses = [
    "delay-[0s]",
    "delay-[-0.24s]",
    "delay-[-0.48s]",
    "delay-[-0.72s]",
    "delay-[-0.96s]",
  ];

  return (
    <div className="flex items-center space-x-1">
      {dotColors.map((colorClass, index) => (
        <div
          key={index}
          className={`h-3 w-6 rounded ${colorClass} animate-wave ${delayClasses[index]}`}
        />
      ))}
    </div>
  );
};

export default LoadingDots;

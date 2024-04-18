import { cn } from "@flow/lib/utils";

const ProjectCard: React.FC<{
  className?: string;
  isSelected?: boolean;
  project: { name: string };
  onClick: () => void;
}> = ({ className, isSelected, project, onClick }) => {
  return (
    <div
      className={cn(
        className,
        isSelected ? "border-zinc-600 bg-zinc-700" : undefined,
        "flex flex-col h-[150px] bg-zinc-800/50 border border-transparent rounded-md p-2 cursor-pointer hover:bg-zinc-800",
      )}
      onClick={onClick}>
      <p className="text-zinc-300 truncate">{project.name}</p>
      <div className="flex-1 bg-[url('@flow/assets/project-screenshot.png')] bg-cover bg-center" />
    </div>
  );
};

export { ProjectCard };

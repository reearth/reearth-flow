import {
  Carousel,
  CarouselContent,
  CarouselItem,
  CarouselNext,
  CarouselPrevious,
} from "@flow/components";
import { ContentSection } from "@flow/features/Dialog/components";
import { useCurrentProject, useCurrentWorkspace, useDialogType } from "@flow/stores";
import { Project } from "@flow/types";

import { ProjectCard } from "./components";

const WelcomeDialogCenter: React.FC = () => {
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentProject, setCurrentProject] = useCurrentProject();
  const [, setDialogType] = useDialogType();

  const handleProjectSelect = (p: Project) => {
    setCurrentProject(p);
    setDialogType(undefined);
  };

  const projects = currentWorkspace?.projects;

  const renderProjects = () => {
    const pairs = [];
    const halfLength = (projects && Math.ceil(projects.length / 2)) || 0;

    for (let i = 0; i < halfLength; i++) {
      const firstItem = projects?.[i];
      const secondItem = projects?.[projects.length - 1 - i];

      pairs.push(
        <CarouselItem key={i} className="md:basis-1/2 lg:basis-1/4 gap-4 flex flex-col">
          {firstItem && (
            <ProjectCard
              isSelected={currentProject?.id === firstItem.id}
              project={firstItem}
              onClick={() => handleProjectSelect(firstItem)}
            />
          )}
          {secondItem && (
            <ProjectCard
              className={currentProject?.id === secondItem.id ? "border-zinc-700" : undefined}
              project={secondItem}
              onClick={() => handleProjectSelect(secondItem)}
            />
          )}
        </CarouselItem>,
      );
    }
    return pairs;
  };

  return (
    <ContentSection
      title={"Projects"}
      content={
        <div className="w-[640px]">
          <Carousel className="w-[630px]" opts={{ align: "start", slidesToScroll: 4, loop: true }}>
            <CarouselContent>{renderProjects()}</CarouselContent>
            <CarouselPrevious />
            <CarouselNext />
          </Carousel>
        </div>
      }
    />
  );
};

export { WelcomeDialogCenter };

import {
  Carousel,
  CarouselContent,
  CarouselItem,
  CarouselNext,
  CarouselPrevious,
} from "@flow/components";
import { ContentSection } from "@flow/features/Dialog/components/ContentSection";

import { projects } from "../../mockProjectData";

const WelcomeDialogCenter: React.FC = () => {
  const renderProjects = () => {
    const pairs = [];
    const halfLength = Math.ceil(projects.length / 2);

    for (let i = 0; i < halfLength; i++) {
      const firstItem = projects[i];
      const secondItem = projects[projects.length - 1 - i];

      pairs.push(
        <CarouselItem key={i} className="md:basis-1/2 lg:basis-1/4 gap-4 flex flex-col">
          {firstItem && <ProjectCard project={firstItem} />}
          {secondItem && <ProjectCard project={secondItem} />}
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

const ProjectCard: React.FC<{ project: { name: string } }> = ({ project }) => {
  return (
    <div className="flex flex-col h-[150px] bg-zinc-800/50 rounded-md p-2 cursor-pointer hover:bg-zinc-800">
      <p className="text-zinc-300 truncate">{project.name}</p>
      <div className="flex-1 bg-[url('@flow/assets/project-screenshot.png')] bg-cover bg-center" />
    </div>
  );
};

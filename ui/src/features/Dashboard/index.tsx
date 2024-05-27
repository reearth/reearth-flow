import { LeftSection, MainSection, Nav } from "./components";

const Dashboard: React.FC = () => {
  return (
    <div className="[&>*]:dark flex flex-col bg-zinc-800 text-zinc-300 h-[100vh]">
      <Nav />
      <div className="flex-1 m-[8px] flex gap-[8px]">
        <LeftSection />
        <MainSection />
      </div>
    </div>
  );
};

export { Dashboard };

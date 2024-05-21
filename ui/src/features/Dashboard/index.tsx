import { LeftSection, MainSection } from "./components";
import { Nav } from "./components/Nav";

const Dashboard: React.FC = () => {
  return (
    <div className="[&>*]:dark relative bg-zinc-800 pt-14 text-zinc-300 h-[100vh]">
      <Nav />
      <div className="border-t border-zinc-700 w-full" />
      <div className="h-[calc(100%-16px)] m-[8px] flex gap-[8px]">
        <LeftSection />
        <MainSection />
      </div>
    </div>
  );
};

export { Dashboard };

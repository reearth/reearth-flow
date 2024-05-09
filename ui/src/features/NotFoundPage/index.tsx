import { useNavigate } from "@tanstack/react-router";

import { Button, FlowLogo } from "@flow/components";

const NotFoundPage: React.FC = () => {
  const navigate = useNavigate();
  return (
    <div className="bg-zinc-800 h-[100vh] flex justify-center items-center">
      <div className="flex flex-col gap-10 items-center">
        <div className="flex gap-4 items-center">
          <div className="bg-red-900 p-2 rounded">
            <FlowLogo className="h-[75px] w-[75px]" />
          </div>
          <p className="text-zinc-300 text-4xl font-extralight">Not Found</p>
        </div>
        <Button variant="outline" onClick={() => navigate({ to: "/workspace" })}>
          <p className="text-zinc-300 font-extralight italic">Go to Dashboard</p>
        </Button>
      </div>
    </div>
  );
};

export default NotFoundPage;

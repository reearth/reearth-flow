import { WelcomeDialogCenter, WelcomeDialogHeader, WelcomeDialogLeft } from "./components";

const WelcomeDialogContent: React.FC = () => {
  return (
    <div className="flex justify-between gap-4">
      <WelcomeDialogLeft />
      <div className="flex-1">
        <WelcomeDialogHeader />
        <WelcomeDialogCenter />
      </div>
    </div>
  );
};

export { WelcomeDialogContent };

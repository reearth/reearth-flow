import { Button, Input, Label } from "@flow/components";
import { useT } from "@flow/providers";
import { useCurrentWorkspace } from "@flow/stores";

const GeneralSettings: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  return (
    <div>
      <p className="text-lg font-extralight">{t("General Settings")}</p>
      <div className="flex flex-col gap-6 mt-4 max-w-[600px]">
        <div className="flex flex-col gap-2">
          <Label htmlFor="workspace-name">{t("Workspace Name")}</Label>
          <Input
            id="workspace-name"
            placeholder={t("Workspace Name")}
            defaultValue={currentWorkspace?.name}
          />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="workspace-description">{t("Workspace Description")}</Label>
          <Input
            id="workspace-description"
            placeholder={t("Workspace Description")}
            defaultValue={currentWorkspace?.description}
          />
        </div>
        <Button className="self-end">{t("Save")}</Button>
      </div>
    </div>
  );
};

export { GeneralSettings };

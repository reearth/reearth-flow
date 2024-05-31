import { Button, Input, Label } from "@flow/components";
import { useT } from "@flow/providers";

const GeneralSettings: React.FC = () => {
  const t = useT();
  return (
    <div>
      <p className="text-lg font-extralight">{t("General Settings")}</p>
      <div className="flex flex-col gap-6 mt-4 max-w-[600px]">
        <div className="flex flex-col gap-2">
          <Label htmlFor="workspace-name">{t("Workspace Name")}</Label>
          <Input id="workspace-name" placeholder={t("Workspace Name")} />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="workspace-description">{t("Workspace Description")}</Label>
          <Input id="workspace-description" placeholder={t("Workspace Description")} />
        </div>
        <Button className="self-end">{t("Save")}</Button>
      </div>
    </div>
  );
};

export { GeneralSettings };

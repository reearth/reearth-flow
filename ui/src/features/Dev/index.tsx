import { FC } from "react";

import { Label, Switch } from "@flow/components";

const Dev: FC = () => {
  return (
    <div className="flex w-full flex-col gap-6 p-6">
      <div className="flex flex-col gap-4">
        <h4 className="text-lg font-extralight">Dev Configs</h4>
        <p className="text-sm">
          This page activates development-specific feature flags and is only
          available when the app is launched in development mode.
        </p>
        <div className="h-px bg-gray-200" />
      </div>
      <div className="space-y-4">
        <div>
          <Label className="text-sm font-medium">
            Vivamus gravida mauris eget tincidunt posuere.
          </Label>
          <p className="mt-2 text-sm text-muted-foreground">
            Vivamus gravida mauris eget tincidunt posuere. Vestibulum vulputate
            diam libero, ut gravida ante consectetur vitae.
          </p>
          <div className="mt-2 flex items-center space-x-3">
            <Switch />
          </div>
        </div>
      </div>
    </div>
  );
};

export default Dev;

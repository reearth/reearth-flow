import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { ContentHeader, ContentSection } from "..";

import { FieldWrapper } from "./components";

const WorkspacesDialogContent: React.FC = () => {
  const t = useT();
  return (
    <>
      <ContentHeader
        title={t("Workspaces settings")}
        // description={t("All settings related to your individual account.")}
      />
      <div className="mx-2">
        <ContentSection
          title={t("Settings section 1")}
          content={
            <div className="mt-2 flex flex-col gap-6">
              <FieldWrapper>
                <p className="mr-4">Setting 1</p>
                <Button variant="outline" size="sm">
                  Change setting
                </Button>
              </FieldWrapper>
              <FieldWrapper>
                <p className="mr-4">Setting 2</p>
                <Button variant="outline" size="sm">
                  Change setting 2
                </Button>
              </FieldWrapper>
            </div>
          }
        />
      </div>
      {/* <DialogFooter>
        <Button type="submit">Save changes</Button>
      </DialogFooter> */}
    </>
  );
};

export { WorkspacesDialogContent };

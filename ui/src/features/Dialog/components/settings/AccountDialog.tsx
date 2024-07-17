import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";

import { ContentHeader, ContentSection } from "..";

import { FieldWrapper } from "./components";

const AccountDialogContent: React.FC = () => {
  const t = useT();
  const { useGetMe } = useUser();
  const { me } = useGetMe();
  return (
    <>
      <ContentHeader title={t("Account settings")} />
      <div className="mx-2">
        <ContentSection
          title={t("Basic information")}
          content={
            <div className="mt-2 flex flex-col gap-6">
              <FieldWrapper>
                <div>
                  <p>{t("Name")}</p>
                  <p className="text-xs text-zinc-400">{me?.name}</p>
                </div>
              </FieldWrapper>
              <FieldWrapper>
                <div>
                  <p>{t("Email address")}</p>
                  <p className="text-xs text-zinc-400">{me?.email}</p>
                </div>
              </FieldWrapper>
            </div>
          }
        />
      </div>
    </>
  );
};

export { AccountDialogContent };

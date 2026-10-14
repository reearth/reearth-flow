import { CheckCircleIcon, WarningIcon } from "@phosphor-icons/react";
import type { ReactNode } from "react";

import {
  DialogContentSection,
  Input,
  Label,
  RadioGroup,
  RadioGroupItem,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  TextArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { PROJECT_ALIAS_MAX_LENGTH } from "@flow/types";
import type { ProjectVisibility } from "@flow/types";

import { LicenseSelect } from "./LicenseSelect";
import { MarkdownEditor } from "./MarkdownEditor";
import { TopicsInput } from "./TopicsInput";
import type useProjectForm from "./useProjectForm";
import type { AliasStatus } from "./useProjectForm";

type Props = {
  form: ReturnType<typeof useProjectForm>;
  // Set when editing; shows the project's default alias in the hint.
  projectId?: string;
};

const isProjectVisibility = (value: unknown): value is ProjectVisibility =>
  value === "public" || value === "private";

const ProjectFormFields: React.FC<Props> = ({ form, projectId }) => {
  const t = useT();
  const {
    values,
    licenseChoice,
    aliasStatus,
    privateDisabled,
    privateNeedsPaidPlan,
    losingPrivate,
    setValue,
    chooseLicense,
  } = form;

  return (
    <Tabs defaultValue="general" className="flex flex-col gap-4">
      <TabsList className="gap-2 self-start">
        <TabsTrigger value="general">{t("General")}</TabsTrigger>
        <TabsTrigger value="details">{t("Details")}</TabsTrigger>
      </TabsList>
      <TabsContent
        value="general"
        className="flex max-h-[60vh] flex-col gap-4 overflow-y-auto">
        <DialogContentSection>
          <Label htmlFor="project-name">{t("Project Name")}</Label>
          <Input
            id="project-name"
            placeholder={t("Project name...")}
            value={values.name}
            onChange={(e) => setValue("name", e.target.value)}
          />
        </DialogContentSection>
        <DialogContentSection>
          <Label htmlFor="project-description">
            {t("Project Description (optional)")}
          </Label>
          <TextArea
            id="project-description"
            placeholder={t("Project description...")}
            value={values.description}
            onChange={(e) => setValue("description", e.target.value)}
          />
        </DialogContentSection>
        <DialogContentSection>
          <Label htmlFor="project-alias">
            {projectId ? t("Alias") : t("Alias (optional)")}
          </Label>
          <Input
            id="project-alias"
            placeholder={projectId ? undefined : t("my-project")}
            value={values.alias}
            maxLength={PROJECT_ALIAS_MAX_LENGTH}
            autoComplete="off"
            spellCheck={false}
            onChange={(e) => setValue("alias", e.target.value)}
          />
          <AliasHint status={aliasStatus} projectId={projectId} />
        </DialogContentSection>
        <DialogContentSection>
          <Label>{t("Visibility")}</Label>
          <RadioGroup
            value={values.visibility}
            onValueChange={(v) =>
              isProjectVisibility(v) && setValue("visibility", v)
            }>
            <VisibilityOption
              value="public"
              label={t("Public")}
              description={t("Anyone can view the project.")}
            />
            <VisibilityOption
              value="private"
              label={t("Private")}
              description={t("Only members of the workspace can see it.")}
              disabled={privateDisabled}
            />
          </RadioGroup>
          {privateNeedsPaidPlan && (
            <p className="text-xs text-muted-foreground">
              {t("Private projects need a paid team plan.")}
            </p>
          )}
          {losingPrivate && (
            <p className="flex items-start gap-1 text-xs text-warning">
              <WarningIcon className="mt-px size-3.5 shrink-0" />
              {t(
                "This workspace's plan does not allow private projects. Once this project is public, it can't be made private again.",
              )}
            </p>
          )}
        </DialogContentSection>
      </TabsContent>
      <TabsContent
        value="details"
        className="flex max-h-[60vh] flex-col gap-4 overflow-y-auto">
        <DialogContentSection>
          <Label htmlFor="project-topics">{t("Topics")}</Label>
          <TopicsInput
            id="project-topics"
            value={values.topics}
            onChange={(topics) => setValue("topics", topics)}
          />
        </DialogContentSection>
        <DialogContentSection>
          {projectId ? (
            <MarkdownEditor
              id="project-license"
              label={t("License")}
              value={values.license}
              placeholder={t("Write down your license")}
              adornment={
                <LicenseSelect
                  value={licenseChoice}
                  placeholder={t("Choose a template")}
                  className="h-6 w-auto max-w-[200px] text-xs"
                  onChange={chooseLicense}
                />
              }
              onChange={(license) => setValue("license", license)}
            />
          ) : (
            <>
              <Label htmlFor="project-license">{t("License")}</Label>
              <LicenseSelect
                id="project-license"
                value={licenseChoice}
                placeholder={t("Choose a license")}
                withNoLicense
                onChange={chooseLicense}
              />
              <p className="text-xs text-muted-foreground">
                {t(
                  "We recommend selecting a license to define how others can use your work and ensure your rights as the creator are protected. You can also create a custom license later from Edit Details.",
                )}
              </p>
            </>
          )}
        </DialogContentSection>
        <DialogContentSection>
          <MarkdownEditor
            id="project-readme"
            label={t("Readme")}
            value={values.readme}
            placeholder={t("Describe this project. Markdown is supported.")}
            onChange={(readme) => setValue("readme", readme)}
          />
        </DialogContentSection>
      </TabsContent>
    </Tabs>
  );
};

const VisibilityOption: React.FC<{
  value: ProjectVisibility;
  label: string;
  description: string;
  disabled?: boolean;
}> = ({ value, label, description, disabled }) => (
  <div className={`flex items-start gap-2 ${disabled ? "opacity-50" : ""}`}>
    <RadioGroupItem
      id={`project-visibility-${value}`}
      value={value}
      disabled={disabled}
      className="mt-0.5"
    />
    <Label
      htmlFor={`project-visibility-${value}`}
      className="flex flex-col gap-0.5">
      <span>{label}</span>
      <span className="text-xs font-light text-muted-foreground">
        {description}
      </span>
    </Label>
  </div>
);

const AliasHint: React.FC<{ status: AliasStatus; projectId?: string }> = ({
  status,
  projectId,
}) => {
  const t = useT();

  let content: ReactNode;
  switch (status.kind) {
    case "default":
      content = (
        <span className="text-muted-foreground">
          {t(
            "Leave empty to use the default alias, p- followed by the project ID.",
          )}
        </span>
      );
      break;
    case "unchanged":
      content = (
        <span className="text-muted-foreground">
          {projectId
            ? t(
                "5 to 32 letters, numbers, hyphens (-) or underscores (_), unique in this workspace. The default is {{alias}}.",
                { alias: `p-${projectId}` },
              )
            : null}
        </span>
      );
      break;
    case "checking":
      content = (
        <span className="text-muted-foreground">
          {t("Checking availability...")}
        </span>
      );
      break;
    case "available":
      content = (
        <span className="flex items-center gap-1 text-success">
          <CheckCircleIcon className="size-3.5" />
          {t("Available")}
        </span>
      );
      break;
    case "taken":
      content = (
        <span className="text-destructive">
          {t("This alias is already used by another project.")}
        </span>
      );
      break;
    case "invalid":
    case "error":
      content = <span className="text-destructive">{status.message}</span>;
      break;
  }

  return <p className="min-h-4 text-xs">{content}</p>;
};

export { ProjectFormFields };

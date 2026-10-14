import { Label, RadioGroup, RadioGroupItem } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { ProjectVisibility } from "@flow/types";

type Props = {
  value: ProjectVisibility;
  privateDisabled?: boolean;
  onChange: (visibility: ProjectVisibility) => void;
};

const isProjectVisibility = (value: unknown): value is ProjectVisibility =>
  value === "public" || value === "private";

// The descriptions follow Re:Earth Visualizer's wording.
const VisibilityRadioGroup: React.FC<Props> = ({
  value,
  privateDisabled,
  onChange,
}) => {
  const t = useT();
  return (
    <RadioGroup
      value={value}
      onValueChange={(v) => isProjectVisibility(v) && onChange(v)}>
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

export { VisibilityRadioGroup };

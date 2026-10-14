import { Radio } from "@base-ui/react/radio";

import { Label, RadioGroup } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";
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
      className="grid-cols-2"
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

// Each option is a card that is outlined and tinted when chosen, so the
// choice reads at a glance in every theme.
const VisibilityOption: React.FC<{
  value: ProjectVisibility;
  label: string;
  description: string;
  disabled?: boolean;
}> = ({ value, label, description, disabled }) => (
  // Wrapping the radio, rather than pointing at it, is what lets a click
  // anywhere on the card pick it.
  <Label
    className={cn(
      "flex cursor-pointer items-start gap-3 rounded-md border p-3 transition-colors hover:bg-accent/40 has-data-checked:border-logo has-data-checked:bg-logo/10",
      disabled && "cursor-not-allowed opacity-50 hover:bg-transparent",
    )}>
    <Radio.Root
      value={value}
      disabled={disabled}
      className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border-2 border-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-hidden data-checked:border-logo data-disabled:cursor-not-allowed">
      <Radio.Indicator className="size-2 rounded-full bg-logo" />
    </Radio.Root>
    <span className="flex flex-col gap-0.5">
      <span>{label}</span>
      <span className="text-xs font-light text-muted-foreground">
        {description}
      </span>
    </span>
  </Label>
);

export { VisibilityRadioGroup };

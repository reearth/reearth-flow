import { QuestionIcon } from "@phosphor-icons/react";

import {
  Label,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@flow/components";

/**
 * The title bar above an object or array — the root form's title, or a nested
 * section's.
 */
type Props = {
  id: string;
  title: string;
  required?: boolean;
  isRoot?: boolean;
  /** Field descriptions summarised in the root title's tooltip. */
  descriptions?: Record<string, unknown>;
};

const SectionHeading: React.FC<Props> = ({
  id,
  title,
  required,
  isRoot,
  descriptions,
}) => {
  const entries = Object.entries(descriptions ?? {});

  return (
    <Label id={id}>
      <div className="my-4 mb-1 flex flex-row items-center justify-between">
        <div className="flex flex-row items-center gap-1">
          <p className={isRoot ? "font-bold" : "font-normal"}>{title}</p>
          {required && <p className="h-2 font-thin text-destructive">*</p>}
          {isRoot && entries.length > 0 && (
            <Tooltip>
              <TooltipTrigger
                render={
                  <div className="cursor-pointer p-1">
                    <QuestionIcon className="h-5 w-5" weight="thin" />
                  </div>
                }
              />
              <TooltipContent side="top" align="end" className="bg-primary">
                <div className="max-w-75 text-xs text-muted-foreground">
                  {entries.map(([key, value]) => (
                    <div key={key}>
                      <span className="font-medium">{key}:</span>{" "}
                      {String(value)}
                    </div>
                  ))}
                </div>
              </TooltipContent>
            </Tooltip>
          )}
        </div>
      </div>
      <div className="border-b" />
    </Label>
  );
};

export { SectionHeading };

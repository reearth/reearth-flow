import { QuestionIcon } from "@phosphor-icons/react";
import {
  FormContextType,
  TitleFieldProps,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import {
  Label,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@flow/components";
import { extractDescriptions } from "@flow/features/Editor/components/ParamsDialog/utils/extractDescriptions";

/** The `TitleField` is the template to use to render the title of a field
 *
 * @param props - The `TitleFieldProps` for this component
 */
const TitleFieldTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  id,
  title,
  required,
  schema,
}: TitleFieldProps<T, S, F>) => {
  // If the schema has a $schema property, it means it's the root title
  const isRootTitle = schema.$schema && schema.title === title;
  const descriptions = extractDescriptions(schema);
  const hasDescriptions = Object.keys(descriptions).length > 0;

  return (
    <Label id={id}>
      <div className="my-4 mb-1 flex flex-row items-center justify-between">
        <div className="flex flex-row items-center gap-1">
          <p className={`${isRootTitle ? "font-bold" : "font-normal"}`}>
            {title}
          </p>
          {required && <p className="h-2 font-thin text-destructive">*</p>}
        </div>
        {isRootTitle && hasDescriptions && (
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="cursor-pointer p-1">
                <QuestionIcon className="h-5 w-5" weight="thin" />
              </div>
            </TooltipTrigger>
            <TooltipContent side="top" align="end" className="bg-primary">
              <div className="max-w-[300px] text-xs text-muted-foreground">
                {Object.entries(descriptions).map(([key, value], index) => (
                  <div key={index}>
                    <span className="font-medium">{key}:</span> {String(value)}
                  </div>
                ))}
              </div>
            </TooltipContent>
          </Tooltip>
        )}
      </div>

      <div className="border-b" />
    </Label>
  );
};

export { TitleFieldTemplate };

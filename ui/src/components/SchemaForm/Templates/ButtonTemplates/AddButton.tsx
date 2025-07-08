import { PlusIcon } from "@radix-ui/react-icons";
import {
  FormContextType,
  IconButtonProps,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import { Button } from "@flow/components";
import { cn } from "@flow/lib/utils";

const AddButton = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  uiSchema,
  registry,
  ...props
}: IconButtonProps<T, S, F>) => {
  return (
    <Button
      {...props}
      size="icon"
      className={cn("ml-1", props.className)}
      aria-label="Add item">
      <PlusIcon />
    </Button>
  );
};

export { AddButton };

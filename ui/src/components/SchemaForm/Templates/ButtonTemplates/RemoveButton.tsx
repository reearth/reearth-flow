import { TrashIcon } from "@radix-ui/react-icons";
import {
  FormContextType,
  IconButtonProps,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import { Button } from "@flow/components";

const RemoveButton = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: IconButtonProps<T, S, F>,
) => {
  return (
    <Button size="icon" {...props} aria-label="Remove item">
      <TrashIcon />
    </Button>
  );
};

export { RemoveButton };

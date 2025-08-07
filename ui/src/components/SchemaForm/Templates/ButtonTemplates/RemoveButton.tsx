import { TrashIcon } from "@phosphor-icons/react";
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
    <Button className="h-6" size="icon" {...props} aria-label="Remove item">
      <TrashIcon className="fill-red-400" />
    </Button>
  );
};

export { RemoveButton };

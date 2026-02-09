import { XIcon } from "@phosphor-icons/react";
import {
  FormContextType,
  IconButtonProps,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import { Button } from "@flow/components";

const ClearButton = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: IconButtonProps<T, S, F>,
) => {
  return (
    <Button className="h-6" size="icon" {...props} aria-label="Clear">
      <XIcon />
    </Button>
  );
};

export { ClearButton };

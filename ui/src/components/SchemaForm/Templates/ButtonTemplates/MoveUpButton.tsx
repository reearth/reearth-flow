import { ArrowUpIcon } from "@radix-ui/react-icons";
import {
  FormContextType,
  IconButtonProps,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import { Button } from "@flow/components";

const MoveUpButton = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: IconButtonProps<T, S, F>,
) => {
  return (
    <Button {...props}>
      <ArrowUpIcon />
    </Button>
  );
};

export { MoveUpButton };

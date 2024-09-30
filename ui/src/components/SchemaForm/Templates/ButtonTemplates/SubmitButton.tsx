import {
  FormContextType,
  getSubmitButtonOptions,
  RJSFSchema,
  StrictRJSFSchema,
  SubmitButtonProps,
} from "@rjsf/utils";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";

const SubmitButton = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>({
  uiSchema,
}: SubmitButtonProps<T, S, F>) => {
  const t = useT();

  const {
    submitText,
    norender,
    props: submitButtonProps = {},
  } = getSubmitButtonOptions<T, S, F>(uiSchema);
  if (norender) {
    return null;
  }
  return (
    <Button type="submit" {...submitButtonProps}>
      {submitText ? submitText : t("Submit")}
    </Button>
  );
};

export { SubmitButton };

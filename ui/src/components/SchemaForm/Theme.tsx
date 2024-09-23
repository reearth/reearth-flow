import { ThemeProps, withTheme } from "@rjsf/core";
import { FormContextType, RJSFSchema, StrictRJSFSchema } from "@rjsf/utils";

import { generateTemplates } from "./Templates";
import { generateWidgets } from "./Widgets";

export function generateTheme<
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = any,
>(): ThemeProps<T, S, F> {
  return {
    templates: generateTemplates<T, S, F>(),
    widgets: generateWidgets<T, S, F>(),
  };
}

const ThemeObject = generateTheme();
const Form = withTheme(ThemeObject);

export { Form };

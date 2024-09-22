import {
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  TemplatesType,
} from "@rjsf/utils";

import { SubmitButton } from "./ButtonTemplates";
import { TitleFieldTemplate } from "./TitleFieldTemplate";

export function generateTemplates<
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = any,
>(): Partial<TemplatesType<T, S, F>> {
  return {
    TitleFieldTemplate,
    ButtonTemplates: {
      SubmitButton,
    },
  };
}

export default generateTemplates();

import {
  FormContextType,
  RegistryWidgetsType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

import { CheckboxesWidget } from "./CheckboxesWidget";
import { CheckboxWidget } from "./CheckboxWidget";
import { RadioWidget } from "./RadioWidget";
import { RangeWidget } from "./RangeWidget";
import { SelectWidget } from "./SelectWidget";
import { TextareaWidget } from "./TextareaWidget";

export function generateWidgets<
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = any,
>(): RegistryWidgetsType<T, S, F> {
  return {
    CheckboxWidget,
    CheckboxesWidget,
    RadioWidget,
    RangeWidget,
    SelectWidget,
    TextareaWidget,
  };
}

export default generateWidgets();

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
  F extends FormContextType = FormContextType,
>(): RegistryWidgetsType<T, S, F> {
  return {
    // Standard widgets that RJSF expects by default
    CheckboxWidget,
    CheckboxesWidget,
    RadioWidget,
    RangeWidget,
    SelectWidget,
    TextareaWidget,
    // Note: Basic inputs (text, number, color) are now handled by BaseInputTemplate
  };
}

const GeneratedWidgets = generateWidgets();
export default GeneratedWidgets;

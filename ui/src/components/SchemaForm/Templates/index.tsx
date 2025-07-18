import {
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  SubmitButtonProps,
  TemplatesType,
} from "@rjsf/utils";
import { ComponentType } from "react";

import { ArrayFieldItemTemplate } from "./ArrayFieldItemTemplate";
import { ArrayFieldTemplate } from "./ArrayFieldTemplate";
import { BaseInputTemplate } from "./BaseInputTemplate";
import {
  AddButton,
  CopyButton,
  MoveDownButton,
  MoveUpButton,
  RemoveButton,
} from "./ButtonTemplates";
import { DescriptionFieldTemplate } from "./DescriptionFieldTemplate";
import { ErrorListTemplate } from "./ErrorListTemplate";
import { FieldErrorTemplate } from "./FieldErrorTemplate";
import { FieldHelpTemplate } from "./FieldHelpTemplate";
import { FieldTemplate } from "./FieldTemplate";
import { ObjectFieldTemplate } from "./ObjectFieldTemplate";
import { TitleFieldTemplate } from "./TitleFieldTemplate";

type SchemaFormTemplates<
  T,
  S extends StrictRJSFSchema,
  F extends FormContextType,
> = Omit<TemplatesType<T, S, F>, "ButtonTemplates"> & {
  ButtonTemplates: Omit<
    TemplatesType<T, S, F>["ButtonTemplates"],
    "SubmitButton"
  > & {
    SubmitButton?: ComponentType<SubmitButtonProps<T, S, F>>;
  };
};

export function generateTemplates<
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(): Partial<SchemaFormTemplates<T, S, F>> {
  return {
    TitleFieldTemplate,
    DescriptionFieldTemplate,
    BaseInputTemplate,
    FieldTemplate,
    ArrayFieldItemTemplate,
    ArrayFieldTemplate,
    ErrorListTemplate,
    FieldErrorTemplate,
    FieldHelpTemplate,
    ObjectFieldTemplate,
    ButtonTemplates: {
      // SubmitButton intentionally omitted - handled outside SchemaForm
      AddButton,
      CopyButton,
      MoveDownButton,
      MoveUpButton,
      RemoveButton,
    },
  };
}

export default generateTemplates();

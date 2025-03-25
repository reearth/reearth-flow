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
  // SubmitButton,
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

type ModifiedButtonTemplates<
  T,
  S extends StrictRJSFSchema,
  F extends FormContextType,
> = Omit<TemplatesType<T, S, F>["ButtonTemplates"], "SubmitButton"> & {
  SubmitButton?: ComponentType<SubmitButtonProps<T, S, F>>;
};

type ModifiedTemplatesType<
  T,
  S extends StrictRJSFSchema,
  F extends FormContextType,
> = Omit<TemplatesType<T, S, F>, "ButtonTemplates"> & {
  ButtonTemplates: ModifiedButtonTemplates<T, S, F>;
};

export function generateTemplates<
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(): Partial<ModifiedTemplatesType<T, S, F>> {
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
      // SubmitButton,
      AddButton,
      CopyButton,
      MoveDownButton,
      MoveUpButton,
      RemoveButton,
    },
  };
}

export default generateTemplates();

import {
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  TemplatesType,
} from "@rjsf/utils";

import { ArrayFieldItemTemplate } from "./ArrayFieldItemTemplate";
import { ArrayFieldTemplate } from "./ArrayFieldTemplate";
import { BaseInputTemplate } from "./BaseInputTemplate";
import {
  SubmitButton,
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

export function generateTemplates<
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = any,
>(): Partial<TemplatesType<T, S, F>> {
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
      SubmitButton,
      AddButton,
      CopyButton,
      MoveDownButton,
      MoveUpButton,
      RemoveButton,
    },
  };
}

export default generateTemplates();

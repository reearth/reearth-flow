import {
  FieldTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  getTemplate,
  getUiOptions,
} from "@rjsf/utils";

import { Tooltip, TooltipContent, TooltipTrigger } from "@flow/components";
import { AwarenessUser } from "@flow/types";

import { ExtendedFormContext } from "./BaseInputTemplate";

/** The `FieldTemplate` component is the template used by `SchemaField` to render any field. It renders the field
 * content, (label, description, children, errors and help) inside of a `WrapIfAdditional` component.
 *
 * @param props - The `FieldTemplateProps` for this component
 */
const FieldTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: FieldTemplateProps<T, S, F>,
) => {
  const {
    id,
    children,
    classNames,
    style,
    disabled,
    displayLabel,
    hidden,
    label,
    onRemoveProperty,
    onKeyRename,
    onKeyRenameBlur,
    readonly,
    required,
    errors,
    help,
    schema,
    uiSchema,
    registry,
  } = props;
  const uiOptions = getUiOptions<T, S, F>(uiSchema);
  const WrapIfAdditionalTemplate = getTemplate(
    "WrapIfAdditionalTemplate",
    registry,
    uiOptions,
  );

  const { fieldFocusMap, onFieldFocus } =
    (registry.formContext as ExtendedFormContext) ?? {};
  const focusedUsers = fieldFocusMap?.[id] ?? [];

  if (hidden) {
    return <div className="hidden">{children}</div>;
  }

  return (
    <WrapIfAdditionalTemplate
      classNames={classNames}
      style={style}
      disabled={disabled}
      id={id}
      label={label}
      onKeyRename={onKeyRename}
      onKeyRenameBlur={onKeyRenameBlur}
      onRemoveProperty={onRemoveProperty}
      readonly={readonly}
      required={required}
      schema={schema}
      uiSchema={uiSchema}
      registry={registry}>
      <div
        className="my-1.5"
        onFocus={(e) => {
          e.stopPropagation();
          onFieldFocus?.(id);
        }}
        onBlur={(e) => {
          e.stopPropagation();
          if (!e.currentTarget.contains(e.relatedTarget as Node)) {
            onFieldFocus?.(null);
          }
        }}>
        {focusedUsers.map((user: AwarenessUser) => (
          <Tooltip key={user.userName}>
            <TooltipTrigger asChild>
              <div
                className="flex h-4 w-4 shrink-0 cursor-default items-center justify-center rounded-full"
                style={{ backgroundColor: user.color }}>
                <span className="text-[9px] font-medium text-white select-none">
                  {user.userName.charAt(0).toUpperCase()}
                </span>
              </div>
            </TooltipTrigger>
            <TooltipContent side="top" className="bg-primary">
              <p className="text-xs text-muted-foreground">
                <span className="font-medium text-foreground">
                  {user.userName}
                </span>
              </p>
            </TooltipContent>
          </Tooltip>
        ))}
        {displayLabel ? (
          <div className="flex flex-1 items-center gap-6">
            <div className="flex flex-row gap-1">
              <p className="shrink-0 font-light">{label}</p>
              {required && <p className="h-2 font-thin text-destructive">*</p>}
            </div>
            <div className="flex-1">{children}</div>
          </div>
        ) : (
          children
        )}
        {errors && (
          <div className="mt-1 text-xs text-destructive" role="alert">
            {errors}
          </div>
        )}
        {help && (
          <div className="mt-1 text-xs text-muted-foreground">{help}</div>
        )}
      </div>
    </WrapIfAdditionalTemplate>
  );
};

export { FieldTemplate };

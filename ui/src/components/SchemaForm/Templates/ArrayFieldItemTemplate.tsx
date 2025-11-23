import {
  ArrayFieldItemTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

const ArrayFieldItemTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: ArrayFieldItemTemplateProps<T, S, F>,
) => {
  const {
    children,
    disabled,
    hasToolbar,
    buttonsProps,
    readonly,
    registry,
    schema,
    uiSchema,
  } = props;

  const {
    hasCopy,
    hasMoveDown,
    hasMoveUp,
    hasRemove,
    onCopyItem,
    onRemoveItem,
    onMoveUpItem,
    onMoveDownItem,
  } = buttonsProps;

  const { CopyButton, MoveDownButton, MoveUpButton, RemoveButton } =
    registry.templates.ButtonTemplates;

  return (
    <div className="relative flex flex-col items-center rounded-md pt-2 pl-2">
      <div
        className={`w-full ${!schema.required ? "flex justify-between gap-2" : ""}`}>
        <div className="flex-1">{children}</div>
        {hasToolbar && (
          <div
            className={`${schema.required ? "absolute top-3.5 right-0" : ""} flex items-center gap-1`}>
            {(hasMoveUp || hasMoveDown) && (
              <MoveUpButton
                disabled={readonly || disabled || !hasMoveUp}
                onClick={onMoveUpItem}
                uiSchema={uiSchema}
                registry={registry}
              />
            )}
            {(hasMoveUp || hasMoveDown) && (
              <MoveDownButton
                disabled={readonly || disabled || !hasMoveDown}
                onClick={onMoveDownItem}
                uiSchema={uiSchema}
                registry={registry}
              />
            )}
            {hasCopy && (
              <CopyButton
                disabled={readonly || disabled}
                onClick={onCopyItem}
                uiSchema={uiSchema}
                registry={registry}
              />
            )}
            {hasRemove && (
              <RemoveButton
                disabled={readonly || disabled}
                onClick={onRemoveItem}
                uiSchema={uiSchema}
                registry={registry}
              />
            )}
          </div>
        )}
      </div>
      <div className="w-full border-b border-primary" />
    </div>
  );
};

export { ArrayFieldItemTemplate };

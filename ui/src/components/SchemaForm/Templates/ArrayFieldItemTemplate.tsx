import {
  ArrayFieldTemplateItemType,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from "@rjsf/utils";

const ArrayFieldItemTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(
  props: ArrayFieldTemplateItemType<T, S, F>,
) => {
  const {
    children,
    disabled,
    hasToolbar,
    hasCopy,
    hasMoveDown,
    hasMoveUp,
    hasRemove,
    index,
    onCopyIndexClick,
    onDropIndexClick,
    onReorderClick,
    readonly,
    registry,
    uiSchema,
  } = props;

  const { CopyButton, MoveDownButton, MoveUpButton, RemoveButton } =
    registry.templates.ButtonTemplates;

  return (
    <div className="relative mt-4 ml-4 flex flex-col items-center gap-2 rounded-md py-1">
      <div className="w-full">{children}</div>
      {hasToolbar && (
        <div className="absolute top-0 right-0 flex flex-row gap-1">
          {(hasMoveUp || hasMoveDown) && (
            <MoveUpButton
              disabled={readonly || disabled || !hasMoveUp}
              onClick={onReorderClick(index, index - 1)}
              uiSchema={uiSchema}
              registry={registry}
            />
          )}
          {(hasMoveUp || hasMoveDown) && (
            <MoveDownButton
              disabled={readonly || disabled || !hasMoveDown}
              onClick={onReorderClick(index, index + 1)}
              uiSchema={uiSchema}
              registry={registry}
            />
          )}
          {hasCopy && (
            <CopyButton
              disabled={readonly || disabled}
              onClick={onCopyIndexClick(index)}
              uiSchema={uiSchema}
              registry={registry}
            />
          )}
          {hasRemove && (
            <RemoveButton
              disabled={readonly || disabled}
              onClick={onDropIndexClick(index)}
              uiSchema={uiSchema}
              registry={registry}
            />
          )}
        </div>
      )}
      <div className="w-full border-b border-primary" />
    </div>
  );
};

export { ArrayFieldItemTemplate };

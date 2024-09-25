import {
  ErrorListProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
  TranslatableString,
} from "@rjsf/utils";

import { Card, CardContent, CardHeader } from "@flow/components";

const ErrorListTemplate = <
  T = any,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = any,
>({
  errors,
  registry,
}: ErrorListProps<T, S, F>) => {
  const { translateString } = registry;
  return (
    <Card className="mb-4 border-destructive">
      <CardHeader>{translateString(TranslatableString.ErrorsLabel)}</CardHeader>
      <CardContent>
        <div>
          {errors.map((error, i: number) => {
            return (
              <div key={i} className="border-none">
                <span>{error.stack}</span>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
};

export { ErrorListTemplate };

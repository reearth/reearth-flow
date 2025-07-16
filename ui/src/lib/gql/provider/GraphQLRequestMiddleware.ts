import { RequestMiddleware } from "graphql-request";
import { isArray, set } from "lodash-es";

const isExtractableFile = <ValueType>(value: ValueType) => {
  return (
    (typeof File !== "undefined" && value instanceof File) ||
    (typeof Blob !== "undefined" && value instanceof Blob) ||
    (typeof Buffer !== "undefined" && value instanceof Buffer) ||
    (typeof value === `object` &&
      value !== null &&
      `pipe` in value &&
      typeof value.pipe === `function`)
  );
};

//@ts-expect-error ignoring errors from "is object"
const isPlainObject = <T>(value: T): value is object =>
  //@ts-expect-error ignoring errors that constructor is not defined
  value && [undefined, Object].includes(value.constructor);

const recursiveExtractFiles = (
  variableKey: string,
  variableValue: any,
  prefix: string,
): any => {
  if (isExtractableFile(variableValue)) {
    return [
      {
        variableKey: [`${prefix}.${variableKey}`],
        file: variableValue,
      },
    ];
  }

  if (
    isArray(variableValue) &&
    variableValue.every((item) => isExtractableFile(item))
  ) {
    return variableValue.map((file, fileIndex) => {
      return {
        variableKey: [`${prefix}.${variableKey}.${fileIndex}`],
        file,
      };
    });
  }

  if (isPlainObject(variableValue)) {
    const ggg = Object.entries(variableValue)
      .map(([key, value]: any) =>
        recursiveExtractFiles(key, value, `${prefix}.${variableKey}`),
      )
      .flat();

    return ggg;
  }

  return [];
};

// headersWAuth is used to pass the auth token due to an issue with
// graphql-request's implementation of requestMiddleware - https://github.com/graffle-js/graffle/issues/1349 @KaWaite
export const requestMiddleware =
  (headersWAuth: Record<string, string>): RequestMiddleware =>
  async (request) => {
    const files = Object.entries(
      (request.variables as { input: unknown })?.["input"] || {},
    ).flatMap(([variableKey, variableValue]) => {
      return recursiveExtractFiles(
        variableKey,
        variableValue,
        "variables.input",
      );
    });

    if (!files.length) {
      return {
        ...request,
        ...headersWAuth,
      };
    }

    const form = new FormData();
    const parsedBody = JSON.parse(request.body as string);
    for (const file of files) {
      //remove file here to reduce request size
      set(parsedBody, file.variableKey[0], null);
    }
    form.append("operations", JSON.stringify(parsedBody));

    const map = files.reduce((accumulator, { variableKey }, index) => {
      return {
        ...accumulator,
        [index.toString()]: variableKey,
      };
    }, {});

    form.append("map", JSON.stringify(map));

    for (let index = 0; index < files.length; index++) {
      const file = files[index];
      form.append(index.toString(), file.file);
    }

    const newHeaders = { ...request.headers, ...headersWAuth } as Record<
      string,
      string
    >;
    delete newHeaders["Content-Type"];

    return {
      ...request,
      body: form,
      headers: {
        ...newHeaders,
      },
    };
  };

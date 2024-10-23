// import { once } from "events";

import { RequestMiddleware } from "graphql-request";
import { isArray, set } from "lodash-es";

// import { CreateDeploymentMutationVariables } from "../__gen__/plugins/graphql-request";

// const toBlob = async (data: any) => {
//   if ("pipe" in data) {
//     const stream = data as NodeJS.ReadableStream;
//     if (!stream) throw new Error("");

//     const chunks: any[] = [];
//     const handler = (data: any) => {
//       chunks.push(data);
//     };
//     stream.on("data", handler);
//     await once(stream, "end");
//     stream.removeListener("data", handler);

//     return new Blob(chunks);
//   }

//   return new Blob([data]);
// };

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

//@ts-expect-error laksdjflaksdjflk
const isPlainObject = <T>(value: T): value is object =>
  //@ts-expect-error laksdjflaksdjflk
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

export const requestMiddleware: RequestMiddleware = async (request) => {
  const files = Object.entries(
    (request.variables as { input: unknown })?.["input"] || {},
  ).flatMap(([variableKey, variableValue]) => {
    return recursiveExtractFiles(variableKey, variableValue, "variables.input");
  });

  if (!files.length) {
    return request;
  }

  const form = new FormData();
  const parsedBody = JSON.parse(request.body as string);
  for (const file of files) {
    //remove file here to reduce request size
    set(parsedBody, file.variableKey[0], null);
  }
  console.log("parsed body: ", parsedBody);
  form.append("operations", JSON.stringify(parsedBody));

  const map = files.reduce((accumulator, { variableKey }, index) => {
    return {
      ...accumulator,
      [index.toString()]: variableKey,
    };
  }, {});

  console.log("map: ", map);

  form.append("map", JSON.stringify(map));

  for (let index = 0; index < files.length; index++) {
    const file = files[index];
    form.append(index.toString(), file.file);
    // form.append(index.toString(), await toBlob(file.file));
  }

  const { "Content-Type": _, ...newHeaders } = request.headers as Record<
    string,
    string
  >;

  form.forEach((value, key) => {
    console.log(`!!!! Key: ${key}, Value:`, value);
  });

  return {
    ...request,
    body: form,
    headers: newHeaders,
  };
};
// export const requestMiddleware: RequestMiddleware = (request) => {
//   console.log("request", request);
//   const files = Object.entries(request.variables["input"] || {}).flatMap(
//     ([variableKey, variableValue]) => {
//       if (isExtractableFile(variableValue)) {
//         console.log("IS EXTRACTABLE FILE");
//         return [
//           {
//             variableKey: [`variables.${variableKey}`],
//             file: variableValue,
//           },
//         ];
//       }

//       if (
//         isArray(variableValue) &&
//         variableValue.every((item) => isExtractableFile(item))
//       ) {
//         console.log("IS ARRAY OF EXTRACTABLE FILES");
//         return variableValue.map((file, fileIndex) => {
//           return {
//             variableKey: [`variables.${variableKey}.${fileIndex}`],
//             file,
//           };
//         });
//       }
//       console.log("ASLKFJLKSJDF", request.variables);

//       return [];
//     },
//   );

//   if (!files.length) {
//     return request;
//   }

//   const form = new FormData();
//   form.append("operations", request.body as string);

//   const map = files.reduce((accumulator, { variableKey }, index) => {
//     return {
//       ...accumulator,
//       [index.toString()]: variableKey,
//     };
//   }, {});

//   form.append("map", JSON.stringify(map));

//   for (let index = 0; index < files.length; index++) {
//     const file = files[index];
//     form.append(index.toString(), file.file);
//   }

//   const { "Content-Type": contentType, ...newHeaders } =
//     request.headers as Record<string, string>;

//   form.forEach((value, key) => {
//     console.log(`!!!! Key: ${key}, Value:`, value);
//   });

//   return {
//     ...request,
//     body: form,
//     headers: newHeaders,
//   };
// };

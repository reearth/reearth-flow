type TextVar = "string";
type NumberVar = "number";
type BooleanVar = "boolean";
type ArrayVar = "array";

export type VarType = TextVar | NumberVar | BooleanVar | ArrayVar;

export type ProjectVar = {
  id: string;
  name: string;
  required: boolean;
  definition: any;
  type: VarType; // TODO: use ParameterType
};

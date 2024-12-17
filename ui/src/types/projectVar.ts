type TextVar = "string";
type NumberVar = "number";
type BooleanVar = "boolean";
type ArrayVar = "array";

export type VarType = TextVar | NumberVar | BooleanVar | ArrayVar;

export type ProjectVar = {
  id: string;
  key: string;
  value: any;
  type: VarType;
};

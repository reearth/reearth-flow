import { ApiResponse } from "./api";

export type Transformer = {
  name: string;
  description: string;
  type: string;
  categories: string[];
};

export type TransformersSegregated = Record<string, Transformer[] | undefined>;

export type Segregated = Record<string, TransformersSegregated>;

export type GetTransformers = {
  transformers?: Transformer[];
  isLoading: boolean;
} & ApiResponse;

export type GetTransformer = {
  transformer?: Transformer;
  isLoading: boolean;
} & ApiResponse;

export type GetTransformerSegregated = {
  transformers?: Segregated;
  isLoading: boolean;
} & ApiResponse;

import { createContext } from "react";

import type { VariableNameProblems } from "@flow/utils";

// The name problems for the table's name inputs. Passed by context rather
// than through the column definitions: new columns remount every cell, which
// takes focus out of the input being typed in.
export const NameProblemsContext = createContext<
  VariableNameProblems | undefined
>(undefined);

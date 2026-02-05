import { useMemo } from "react";
import type { A2UISchema } from "../components/A2UIRenderer";
import { getSchema } from "../schemas/schemaLoader";

export function useSchema(name: string): A2UISchema | undefined {
  return useMemo(() => getSchema(name), [name]);
}

import { A2UIViewer } from "@copilotkit/a2ui-renderer";
import type { v0_8 } from "@a2ui/lit";

export interface A2UISchema {
  root: string;
  components: v0_8.Types.ComponentInstance[];
  defaultData?: Record<string, unknown>;
}

interface Props {
  schema: A2UISchema;
  data?: Record<string, unknown>;
  onAction?: (action: v0_8.Types.UserAction) => void;
}

export function A2UIRenderer({ schema, data, onAction }: Props) {
  const mergedData = { ...schema.defaultData, ...data };

  return (
    <A2UIViewer
      root={schema.root}
      components={schema.components}
      data={mergedData}
      onAction={onAction}
    />
  );
}

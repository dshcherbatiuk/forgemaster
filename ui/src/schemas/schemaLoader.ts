import type { A2UISchema } from "../components/A2UIRenderer";

const schemaModules = import.meta.glob<{ default: A2UISchema }>("./*.json", {
  eager: true,
});

const schemaCache = new Map<string, A2UISchema>();

for (const [path, module] of Object.entries(schemaModules)) {
  const name = path.replace("./", "").replace(".json", "");
  schemaCache.set(name, module.default);
}

export function getSchema(name: string): A2UISchema | undefined {
  return schemaCache.get(name);
}

export function getAvailableSchemas(): string[] {
  return Array.from(schemaCache.keys());
}

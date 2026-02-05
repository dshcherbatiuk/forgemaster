import type { A2UISchema } from "../components/A2UIRenderer";

interface ComponentFile {
  id: string;
  components: A2UISchema["components"];
}

interface ComposableSchema {
  root: string;
  includes: string[];
  layout: A2UISchema["components"];
  defaultData?: Record<string, unknown>;
}

// Load all component files from components/ directory
const componentModules = import.meta.glob<{ default: ComponentFile }>(
  "./components/*.json",
  { eager: true }
);

// Load all schema files from schemas/ directory (non-component)
const schemaModules = import.meta.glob<{ default: ComposableSchema | A2UISchema }>(
  "./*.json",
  { eager: true }
);

// Build component registry
const componentRegistry = new Map<string, ComponentFile>();
for (const [path, module] of Object.entries(componentModules)) {
  const name = path.replace("./components/", "").replace(".json", "");
  componentRegistry.set(name, module.default);
}

// Build schema cache with resolved components
const schemaCache = new Map<string, A2UISchema>();

function isComposableSchema(schema: unknown): schema is ComposableSchema {
  return (
    typeof schema === "object" &&
    schema !== null &&
    "includes" in schema &&
    Array.isArray((schema as ComposableSchema).includes)
  );
}

function resolveSchema(schema: ComposableSchema | A2UISchema): A2UISchema {
  if (!isComposableSchema(schema)) {
    return schema as A2UISchema;
  }

  // Merge all included components
  const allComponents: A2UISchema["components"] = [...schema.layout];

  for (const includeName of schema.includes) {
    const component = componentRegistry.get(includeName);
    if (component) {
      allComponents.push(...component.components);
    } else {
      console.warn(`Component not found: ${includeName}`);
    }
  }

  return {
    root: schema.root,
    components: allComponents,
    defaultData: schema.defaultData,
  };
}

// Initialize schema cache
for (const [path, module] of Object.entries(schemaModules)) {
  const name = path.replace("./", "").replace(".json", "");
  schemaCache.set(name, resolveSchema(module.default));
}

export function getSchema(name: string): A2UISchema | undefined {
  return schemaCache.get(name);
}

export function getAvailableSchemas(): string[] {
  return Array.from(schemaCache.keys());
}

export function getAvailableComponents(): string[] {
  return Array.from(componentRegistry.keys());
}

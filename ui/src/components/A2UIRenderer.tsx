import { useEffect, useRef } from "react";
import "@a2ui/lit";

interface Props {
  schema: object;
  onAction?: (action: string, data: unknown) => void;
}

export function A2UIRenderer({ schema, onAction }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (containerRef.current) {
      const renderer = document.createElement("a2ui-renderer");
      renderer.setAttribute("schema", JSON.stringify(schema));

      if (onAction) {
        renderer.addEventListener("action", (e: Event) => {
          const customEvent = e as CustomEvent;
          onAction(customEvent.detail.action, customEvent.detail.data);
        });
      }

      containerRef.current.innerHTML = "";
      containerRef.current.appendChild(renderer);
    }
  }, [schema, onAction]);

  return <div ref={containerRef} />;
}

const BLOCKED_SVG_ELEMENTS = new Set([
  "script",
  "foreignObject",
  "iframe",
  "object",
  "embed",
]);

export function mountTrustedSvg(
  container: HTMLElement,
  svgSource: string,
): void {
  const parser = new DOMParser();
  const document = parser.parseFromString(svgSource, "image/svg+xml");
  const parserError = document.querySelector("parsererror");
  const svg = document.documentElement;

  if (parserError || svg.tagName.toLowerCase() !== "svg") {
    throw new Error("Renderer output is not a valid SVG document");
  }

  for (const element of Array.from(svg.querySelectorAll("*"))) {
    const tagName = element.tagName.toLowerCase();
    if (BLOCKED_SVG_ELEMENTS.has(tagName)) {
      throw new Error(`Renderer SVG contains blocked element: ${tagName}`);
    }
    for (const attribute of Array.from(element.attributes)) {
      const name = attribute.name.toLowerCase();
      const value = attribute.value.trim().toLowerCase();
      if (name.startsWith("on") || value.startsWith("javascript:")) {
        throw new Error(
          `Renderer SVG contains blocked attribute: ${attribute.name}`,
        );
      }
    }
  }

  const imported = container.ownerDocument.importNode(svg, true);
  container.replaceChildren(imported);
}

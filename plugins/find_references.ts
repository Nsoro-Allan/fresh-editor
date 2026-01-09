/// <reference path="./lib/fresh.d.ts" />

import {
  ResultsPanel,
  ResultItem,
  createStaticProvider,
  getRelativePath,
} from "./lib/results-panel.ts";

const editor = getEditor();

/**
 * Find References Plugin
 *
 * Displays LSP find references results using the ResultsPanel abstraction
 * with VS Code-inspired Provider pattern.
 *
 * Key features:
 * - Provider pattern for live data channel
 * - syncWithEditor for bidirectional cursor sync
 * - groupBy: "file" for organized display
 */

// Maximum number of results to display
const MAX_RESULTS = 100;

// Reference item structure from LSP
interface ReferenceLocation {
  file: string;
  line: number;
  column: number;
}

// Line text cache for previews
const lineCache: Map<string, string[]> = new Map();

// Create a static provider (Find References is a snapshot, not live data)
const provider = createStaticProvider<ResultItem>();

// Create the panel with Provider pattern
const panel = new ResultsPanel(editor, "references", provider, {
  title: "References", // Will be updated when showing
  syncWithEditor: true, // Enable bidirectional cursor sync!
  groupBy: "file", // Group by file for better organization
  ratio: 0.7,
  onSelect: (item) => {
    if (item.location) {
      panel.openInSource(
        item.location.file,
        item.location.line,
        item.location.column
      );
      const displayPath = getRelativePath(editor, item.location.file);
      editor.setStatus(`Jumped to ${displayPath}:${item.location.line}`);
    }
  },
  onClose: () => {
    lineCache.clear();
  },
});

/**
 * Load line text for references (for preview display)
 */
async function loadLineTexts(
  references: ReferenceLocation[]
): Promise<Map<string, string>> {
  const lineTexts = new Map<string, string>();

  // Group references by file
  const fileRefs: Map<string, ReferenceLocation[]> = new Map();
  for (const ref of references) {
    if (!fileRefs.has(ref.file)) {
      fileRefs.set(ref.file, []);
    }
    fileRefs.get(ref.file)!.push(ref);
  }

  // Load each file and extract lines
  for (const [filePath, refs] of fileRefs) {
    try {
      let lines = lineCache.get(filePath);
      if (!lines) {
        const content = await editor.readFile(filePath);
        lines = content.split("\n");
        lineCache.set(filePath, lines);
      }

      for (const ref of refs) {
        const lineIndex = ref.line - 1;
        if (lineIndex >= 0 && lineIndex < lines.length) {
          const key = `${ref.file}:${ref.line}:${ref.column}`;
          lineTexts.set(key, lines[lineIndex]);
        }
      }
    } catch {
      // If file can't be read, skip
    }
  }

  return lineTexts;
}

/**
 * Convert LSP references to ResultItems for display
 */
function referencesToItems(
  references: ReferenceLocation[],
  lineTexts: Map<string, string>
): ResultItem[] {
  return references.map((ref, index) => {
    const displayPath = getRelativePath(editor, ref.file);
    const key = `${ref.file}:${ref.line}:${ref.column}`;
    const lineText = lineTexts.get(key) || "";
    const trimmedLine = lineText.trim();

    // Format label as "line:col"
    const label = `${ref.line}:${ref.column}`;

    // Preview text
    const maxPreviewLen = 60;
    const preview =
      trimmedLine.length > maxPreviewLen
        ? trimmedLine.slice(0, maxPreviewLen - 3) + "..."
        : trimmedLine;

    return {
      // Unique ID for reveal/sync
      id: `ref-${index}-${ref.file}-${ref.line}-${ref.column}`,
      label: label,
      description: preview,
      location: {
        file: ref.file,
        line: ref.line,
        column: ref.column,
      },
    };
  });
}

/**
 * Show references panel with the given results
 */
async function showReferences(
  symbol: string,
  references: ReferenceLocation[]
): Promise<void> {
  // Limit results
  const limitedRefs = references.slice(0, MAX_RESULTS);

  // Clear and reload line cache
  lineCache.clear();
  const lineTexts = await loadLineTexts(limitedRefs);

  // Convert to ResultItems
  const items = referencesToItems(limitedRefs, lineTexts);

  // Update panel title dynamically
  const count = references.length;
  const limitNote = count > MAX_RESULTS ? ` (first ${MAX_RESULTS})` : "";
  (panel as { options: { title: string } }).options.title =
    `References to '${symbol}': ${count}${limitNote}`;

  // Update provider with new items (triggers panel refresh if open)
  provider.updateItems(items);

  // Show panel if not already open
  if (!panel.isOpen) {
    await panel.show();
  }
}

// Handle lsp_references hook
globalThis.on_lsp_references = function (data: {
  symbol: string;
  locations: ReferenceLocation[];
}): void {
  editor.debug(`Received ${data.locations.length} references for '${data.symbol}'`);

  if (data.locations.length === 0) {
    editor.setStatus(`No references found for '${data.symbol}'`);
    return;
  }

  showReferences(data.symbol, data.locations);
};

// Register the hook handler
editor.on("lsp_references", "on_lsp_references");

// Export close function for command palette
globalThis.hide_references_panel = function (): void {
  panel.close();
};

// Register commands
editor.registerCommand(
  "%cmd.show_references",
  "%cmd.show_references_desc",
  "show_references_panel",
  "normal"
);

editor.registerCommand(
  "%cmd.hide_references",
  "%cmd.hide_references_desc",
  "hide_references_panel",
  "normal"
);

// Plugin initialization
editor.setStatus("Find References plugin ready");
editor.debug("Find References plugin initialized (Provider pattern v2)");

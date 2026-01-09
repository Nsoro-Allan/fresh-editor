/// <reference path="./fresh.d.ts" />

import type { Location, RGB } from "./types.ts";

/**
 * ResultsPanel v2 - VS Code-inspired Provider Pattern
 *
 * Key architecture principles (lessons from VS Code's TreeDataProvider):
 * 1. Don't pass arrays, pass Providers - creates a live data channel
 * 2. Standardize the Item shape - Core handles sync automatically
 * 3. Event-driven updates - Provider emits events, Panel refreshes
 *
 * @example
 * ```typescript
 * // Create a provider that emits change events
 * class MyProvider implements ResultsProvider<ResultItem> {
 *   private items: ResultItem[] = [];
 *   private emitter = new EventEmitter<void>();
 *   readonly onDidChangeResults = this.emitter.event;
 *
 *   updateItems(newItems: ResultItem[]) {
 *     this.items = newItems;
 *     this.emitter.fire(); // Notify panel to refresh
 *   }
 *
 *   provideResults() {
 *     return this.items;
 *   }
 * }
 *
 * const provider = new MyProvider();
 * const panel = new ResultsPanel(editor, "references", provider, {
 *   title: "References",
 *   syncWithEditor: true,
 *   onSelect: (item) => {
 *     if (item.location) {
 *       panel.openInSource(item.location.file, item.location.line, item.location.column);
 *     }
 *   },
 * });
 *
 * // Later: update data
 * provider.updateItems(newReferences);
 * ```
 */

// ============================================================================
// Event System (Simplified VS Code-style)
// ============================================================================

/**
 * A function that can be called to unsubscribe from an event
 */
export type Disposable = () => void;

/**
 * An event that can be subscribed to
 */
export type Event<T> = (listener: (e: T) => void) => Disposable;

/**
 * Simple event emitter for Provider → Panel communication
 */
export class EventEmitter<T> {
  private listeners: Array<(e: T) => void> = [];

  /**
   * The event that others can subscribe to
   */
  readonly event: Event<T> = (listener) => {
    this.listeners.push(listener);
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index >= 0) {
        this.listeners.splice(index, 1);
      }
    };
  };

  /**
   * Fire the event, notifying all listeners
   */
  fire(data: T): void {
    for (const listener of this.listeners) {
      try {
        listener(data);
      } catch (e) {
        // Don't let one listener break others
        console.error("Event listener error:", e);
      }
    }
  }

  /**
   * Fire without data (for void events)
   */
  fireVoid(): void {
    this.fire(undefined as T);
  }
}

// ============================================================================
// Core Interfaces
// ============================================================================

/**
 * Standard shape for any item in a results list.
 *
 * By enforcing a standard `location` property, the Core can implement
 * "Sync Cursor" logic once, globally, rather than asking every plugin
 * to write custom sync callbacks.
 */
export interface ResultItem {
  /** Unique identifier for this item (used for reveal/selection) */
  id: string;

  /** Primary text shown for this item */
  label: string;

  /** Secondary text (e.g., code preview) */
  description?: string;

  /**
   * Location in source file - CRITICAL for Core-managed features:
   * - Bidirectional cursor sync (syncWithEditor)
   * - Navigation (Enter to jump)
   */
  location?: Location;

  /** Severity for visual styling (error/warning/info badge) */
  severity?: "error" | "warning" | "info" | "hint";

  /** Custom data attached to this item */
  metadata?: unknown;
}

/**
 * The Provider acts as the bridge between Plugin Logic and UI.
 *
 * This matches VS Code's TreeDataProvider pattern where:
 * - Provider owns the data and business logic
 * - Panel owns the UI rendering and interaction
 */
export interface ResultsProvider<T extends ResultItem = ResultItem> {
  /**
   * Data Retrieval: Core calls this when it needs the current items.
   * Can be sync or async.
   */
  provideResults(): T[] | Promise<T[]>;

  /**
   * Reactivity: Plugin fires this to tell Core "My data changed, please refresh".
   * Matches VS Code's 'onDidChangeTreeData' pattern.
   *
   * If omitted, the panel won't auto-refresh; you must call panel.refresh() manually.
   */
  onDidChangeResults?: Event<void>;

  /**
   * Optional filtering logic for complex custom filtering.
   * If omitted, Core does simple substring matching on label.
   */
  filter?(item: T, query: string): boolean;
}

// ============================================================================
// Panel Options
// ============================================================================

/**
 * Options for creating a ResultsPanel
 */
export interface ResultsPanelOptions<T extends ResultItem = ResultItem> {
  /** Title shown at top of panel */
  title: string;

  /**
   * Bidirectional Sync: If true, Core automatically highlights items
   * matching the active editor's cursor position, based on the item's
   * `location` property. No custom callback needed.
   */
  syncWithEditor?: boolean;

  /**
   * Grouping strategy for items.
   * - 'file': Group by file path (default for location-based items)
   * - 'severity': Group by severity level
   * - 'none': Flat list, no grouping
   */
  groupBy?: "file" | "severity" | "none";

  /** Split ratio (default 0.7 = source keeps 70%) */
  ratio?: number;

  /** Called when user presses Enter on an item */
  onSelect?: (item: T, index: number) => void;

  /** Called when user presses Escape */
  onClose?: () => void;

  /** Called when cursor moves to a new item (for preview updates) */
  onCursorMove?: (item: T, index: number) => void;
}

// ============================================================================
// Colors
// ============================================================================

const colors = {
  selected: [80, 80, 120] as RGB,
  location: [150, 255, 150] as RGB,
  help: [150, 150, 150] as RGB,
  title: [200, 200, 255] as RGB,
  error: [255, 100, 100] as RGB,
  warning: [255, 200, 100] as RGB,
  info: [100, 200, 255] as RGB,
  hint: [150, 150, 150] as RGB,
  fileHeader: [180, 180, 255] as RGB,
};

// ============================================================================
// Internal State
// ============================================================================

interface PanelState<T extends ResultItem> {
  isOpen: boolean;
  bufferId: number | null;
  splitId: number | null;
  sourceSplitId: number | null;
  cachedContent: string;
  cursorLine: number;
  items: T[];
  // Maps panel line -> item index (for sync)
  lineToItemIndex: Map<number, number>;
}

// ============================================================================
// ResultsPanel Class
// ============================================================================

/**
 * ResultsPanel - manages a results list panel with Provider pattern
 */
export class ResultsPanel<T extends ResultItem = ResultItem> {
  private state: PanelState<T> = {
    isOpen: false,
    bufferId: null,
    splitId: null,
    sourceSplitId: null,
    cachedContent: "",
    cursorLine: 1,
    items: [],
    lineToItemIndex: new Map(),
  };

  private readonly modeName: string;
  private readonly panelName: string;
  private readonly namespace: string;
  private readonly handlerPrefix: string;

  private providerDisposable: Disposable | null = null;
  private cursorSyncDisposable: Disposable | null = null;

  /**
   * Create a new ResultsPanel with a Provider
   *
   * @param editor - The editor API instance
   * @param id - Unique identifier for this panel (e.g., "references", "diagnostics")
   * @param provider - The data provider
   * @param options - Panel configuration
   */
  constructor(
    private readonly editor: EditorAPI,
    private readonly id: string,
    private readonly provider: ResultsProvider<T>,
    private readonly options: ResultsPanelOptions<T>
  ) {
    this.modeName = `${id}-results`;
    this.panelName = `*${id.charAt(0).toUpperCase() + id.slice(1)}*`;
    this.namespace = id;
    this.handlerPrefix = `_results_panel_${id}`;

    // Define mode with minimal keybindings (navigation inherited from "normal")
    editor.defineMode(
      this.modeName,
      "normal",
      [
        ["Return", `${this.handlerPrefix}_select`],
        ["Escape", `${this.handlerPrefix}_close`],
      ],
      true
    );

    // Register global handlers
    this.registerHandlers();

    // Auto-subscribe to provider changes (the "VS Code Way")
    if (this.provider.onDidChangeResults) {
      this.providerDisposable = this.provider.onDidChangeResults(() => {
        if (this.state.isOpen) {
          this.refresh();
        }
      });
    }
  }

  // ==========================================================================
  // Public API
  // ==========================================================================

  get isOpen(): boolean {
    return this.state.isOpen;
  }

  get bufferId(): number | null {
    return this.state.bufferId;
  }

  get sourceSplitId(): number | null {
    return this.state.sourceSplitId;
  }

  /**
   * Show the panel, fetching items from the provider
   */
  async show(): Promise<void> {
    // Save source context if not already open
    if (!this.state.isOpen) {
      this.state.sourceSplitId = this.editor.getActiveSplitId();
    }

    // Fetch items from provider
    this.state.items = await Promise.resolve(this.provider.provideResults());

    // Build entries
    const entries = this.buildEntries();
    this.state.cachedContent = entries.map((e) => e.text).join("");
    this.state.cursorLine = this.findFirstItemLine();

    try {
      const result = await this.editor.createVirtualBufferInSplit({
        name: this.panelName,
        mode: this.modeName,
        read_only: true,
        entries: entries,
        ratio: this.options.ratio ?? 0.7,
        direction: "horizontal",
        panel_id: this.id,
        show_line_numbers: false,
        show_cursors: true,
        editing_disabled: true,
      });

      if (result.buffer_id !== null) {
        this.state.bufferId = result.buffer_id;
        this.state.splitId = result.split_id ?? null;
        this.state.isOpen = true;
        this.applyHighlighting();

        // Enable bidirectional cursor sync if requested
        if (this.options.syncWithEditor) {
          this.enableCursorSync();
        }

        const count = this.state.items.length;
        this.editor.setStatus(
          `${this.options.title}: ${count} item${count !== 1 ? "s" : ""}`
        );
      } else {
        this.editor.setStatus(`Failed to open ${this.panelName}`);
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      this.editor.setStatus(`Failed to open panel: ${msg}`);
      this.editor.debug(`ResultsPanel error: ${msg}`);
    }
  }

  /**
   * Refresh the panel by re-fetching from the provider
   */
  async refresh(): Promise<void> {
    if (!this.state.isOpen || this.state.bufferId === null) {
      return;
    }

    this.state.items = await Promise.resolve(this.provider.provideResults());

    const entries = this.buildEntries();
    this.state.cachedContent = entries.map((e) => e.text).join("");

    this.editor.setVirtualBufferContent(this.state.bufferId, entries);
    this.applyHighlighting();

    const count = this.state.items.length;
    this.editor.setStatus(
      `${this.options.title}: ${count} item${count !== 1 ? "s" : ""}`
    );
  }

  /**
   * Close the panel and clean up
   */
  close(): void {
    if (!this.state.isOpen) {
      return;
    }

    // Capture values before clearing
    const splitId = this.state.splitId;
    const bufferId = this.state.bufferId;
    const sourceSplitId = this.state.sourceSplitId;

    // Disable cursor sync
    if (this.cursorSyncDisposable) {
      this.cursorSyncDisposable();
      this.cursorSyncDisposable = null;
    }

    // Clear state
    this.state.isOpen = false;
    this.state.bufferId = null;
    this.state.splitId = null;
    this.state.sourceSplitId = null;
    this.state.cachedContent = "";
    this.state.cursorLine = 1;
    this.state.items = [];
    this.state.lineToItemIndex.clear();

    // Close split and buffer
    if (splitId !== null) {
      this.editor.closeSplit(splitId);
    }
    if (bufferId !== null) {
      this.editor.closeBuffer(bufferId);
    }

    // Focus source
    if (sourceSplitId !== null) {
      this.editor.focusSplit(sourceSplitId);
    }

    // Call user callback
    if (this.options.onClose) {
      this.options.onClose();
    }

    this.editor.setStatus(`${this.panelName} closed`);
  }

  /**
   * Reveal an item by ID (scroll to and highlight)
   */
  reveal(itemId: string, options?: { focus?: boolean; select?: boolean }): void {
    if (!this.state.isOpen || this.state.bufferId === null) return;

    const index = this.state.items.findIndex((item) => item.id === itemId);
    if (index === -1) return;

    // Find the panel line for this item
    for (const [line, idx] of this.state.lineToItemIndex) {
      if (idx === index) {
        this.state.cursorLine = line;

        // Move cursor to this line
        const byteOffset = this.lineToByteOffset(line);
        this.editor.setBufferCursor(this.state.bufferId, byteOffset);
        this.applyHighlighting();

        if (options?.focus) {
          this.focusPanel();
        }
        break;
      }
    }
  }

  /**
   * Open a file in the source split and jump to location
   */
  openInSource(file: string, line: number, column: number): void {
    if (this.state.sourceSplitId === null) return;

    this.editor.focusSplit(this.state.sourceSplitId);
    this.editor.openFile(file, line, column);
  }

  /**
   * Focus the source split
   */
  focusSource(): void {
    if (this.state.sourceSplitId !== null) {
      this.editor.focusSplit(this.state.sourceSplitId);
    }
  }

  /**
   * Focus the panel split
   */
  focusPanel(): void {
    if (this.state.splitId !== null) {
      this.editor.focusSplit(this.state.splitId);
    }
  }

  /**
   * Get the currently selected item
   */
  getSelectedItem(): T | null {
    const index = this.state.lineToItemIndex.get(this.state.cursorLine);
    if (index !== undefined && index < this.state.items.length) {
      return this.state.items[index];
    }
    return null;
  }

  /**
   * Dispose the panel and all subscriptions
   */
  dispose(): void {
    this.close();
    if (this.providerDisposable) {
      this.providerDisposable();
      this.providerDisposable = null;
    }
  }

  // ==========================================================================
  // Private Methods
  // ==========================================================================

  private registerHandlers(): void {
    const self = this;

    // Select handler (Enter)
    (globalThis as Record<string, unknown>)[`${this.handlerPrefix}_select`] =
      function (): void {
        if (!self.state.isOpen) return;

        const item = self.getSelectedItem();
        if (item && self.options.onSelect) {
          const index = self.state.items.indexOf(item);
          self.options.onSelect(item, index);
        } else if (!item) {
          self.editor.setStatus("No item selected");
        }
      };

    // Close handler (Escape)
    (globalThis as Record<string, unknown>)[`${this.handlerPrefix}_close`] =
      function (): void {
        self.close();
      };

    // Panel cursor movement handler
    (globalThis as Record<string, unknown>)[
      `${this.handlerPrefix}_cursor_moved`
    ] = function (data: {
      buffer_id: number;
      cursor_id: number;
      old_position: number;
      new_position: number;
      line: number;
    }): void {
      if (!self.state.isOpen || self.state.bufferId === null) return;
      if (data.buffer_id !== self.state.bufferId) return;

      self.state.cursorLine = data.line;
      self.applyHighlighting();

      // Get the item at this line
      const itemIndex = self.state.lineToItemIndex.get(data.line);
      if (itemIndex !== undefined && itemIndex < self.state.items.length) {
        const item = self.state.items[itemIndex];
        self.editor.setStatus(`Item ${itemIndex + 1}/${self.state.items.length}`);

        if (self.options.onCursorMove) {
          self.options.onCursorMove(item, itemIndex);
        }
      }
    };

    // Register cursor movement handler
    this.editor.on("cursor_moved", `${this.handlerPrefix}_cursor_moved`);
  }

  /**
   * Enable bidirectional cursor sync with source files
   */
  private enableCursorSync(): void {
    const self = this;
    const handlerName = `${this.handlerPrefix}_source_cursor`;

    // Handler for cursor movement in SOURCE files
    (globalThis as Record<string, unknown>)[handlerName] = function (data: {
      buffer_id: number;
      cursor_id: number;
      old_position: number;
      new_position: number;
      line: number;
    }): void {
      if (!self.state.isOpen || self.state.bufferId === null) return;

      // Ignore cursor moves in the panel itself
      if (data.buffer_id === self.state.bufferId) return;

      // Get the file path for this buffer
      const filePath = self.editor.getBufferPath(data.buffer_id);
      if (!filePath) return;

      // Find an item that matches this file and line
      const matchingIndex = self.state.items.findIndex((item) => {
        if (!item.location) return false;
        return (
          item.location.file === filePath && item.location.line === data.line
        );
      });

      if (matchingIndex >= 0) {
        const item = self.state.items[matchingIndex];
        // Reveal this item in the panel (without stealing focus)
        self.reveal(item.id, { focus: false, select: true });
      }
    };

    // Register the handler
    this.editor.on("cursor_moved", handlerName);

    // Store disposable to unregister later
    this.cursorSyncDisposable = () => {
      // Note: Fresh doesn't have an "off" method, so we just make the handler a no-op
      (globalThis as Record<string, unknown>)[handlerName] = () => {};
    };
  }

  private buildEntries(): TextPropertyEntry[] {
    const entries: TextPropertyEntry[] = [];
    this.state.lineToItemIndex.clear();

    let currentLine = 1;

    // Title line
    entries.push({
      text: `${this.options.title}\n`,
      properties: { type: "title" },
    });
    currentLine++;

    if (this.state.items.length === 0) {
      entries.push({
        text: "  No results\n",
        properties: { type: "empty" },
      });
      currentLine++;
    } else if (this.options.groupBy === "file") {
      // Group by file
      const byFile = new Map<string, Array<{ item: T; index: number }>>();

      for (let i = 0; i < this.state.items.length; i++) {
        const item = this.state.items[i];
        const file = item.location?.file ?? "(no file)";
        if (!byFile.has(file)) {
          byFile.set(file, []);
        }
        byFile.get(file)!.push({ item, index: i });
      }

      for (const [file, itemsInFile] of byFile) {
        // File header
        const fileName = file.split("/").pop() ?? file;
        entries.push({
          text: `\n${fileName}:\n`,
          properties: { type: "file-header", file },
        });
        currentLine += 2;

        // Items in this file
        for (const { item, index } of itemsInFile) {
          entries.push(this.buildItemEntry(item, index));
          this.state.lineToItemIndex.set(currentLine, index);
          currentLine++;
        }
      }
    } else {
      // Flat list
      for (let i = 0; i < this.state.items.length; i++) {
        const item = this.state.items[i];
        entries.push(this.buildItemEntry(item, i));
        this.state.lineToItemIndex.set(currentLine, i);
        currentLine++;
      }
    }

    // Help footer
    entries.push({
      text: "\n",
      properties: { type: "blank" },
    });
    entries.push({
      text: "Enter:select | Esc:close\n",
      properties: { type: "help" },
    });

    return entries;
  }

  private buildItemEntry(item: T, _index: number): TextPropertyEntry {
    const severityIcon =
      item.severity === "error"
        ? "[E]"
        : item.severity === "warning"
          ? "[W]"
          : item.severity === "info"
            ? "[I]"
            : item.severity === "hint"
              ? "[H]"
              : "";

    const prefix = severityIcon ? `${severityIcon} ` : "  ";
    const desc = item.description ? `  ${item.description}` : "";

    let line = `${prefix}${item.label}${desc}`;
    const maxLen = 100;
    if (line.length > maxLen) {
      line = line.slice(0, maxLen - 3) + "...";
    }

    return {
      text: `${line}\n`,
      properties: {
        type: "item",
        id: item.id,
        location: item.location,
        severity: item.severity,
        metadata: item.metadata,
      },
    };
  }

  private findFirstItemLine(): number {
    // Find the first line that has an item
    for (const [line] of this.state.lineToItemIndex) {
      return line;
    }
    return 2; // Default to line after title
  }

  private lineToByteOffset(lineNumber: number): number {
    const lines = this.state.cachedContent.split("\n");
    let offset = 0;
    for (let i = 0; i < lineNumber - 1 && i < lines.length; i++) {
      offset += lines[i].length + 1;
    }
    return offset;
  }

  private applyHighlighting(): void {
    if (this.state.bufferId === null) return;

    const bufferId = this.state.bufferId;
    this.editor.clearNamespace(bufferId, this.namespace);

    if (!this.state.cachedContent) return;

    const lines = this.state.cachedContent.split("\n");
    let byteOffset = 0;

    for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
      const line = lines[lineIdx];
      const lineStart = byteOffset;
      const lineEnd = byteOffset + line.length;
      const lineNumber = lineIdx + 1;
      const isCurrentLine = lineNumber === this.state.cursorLine;
      const isItemLine = this.state.lineToItemIndex.has(lineNumber);

      // Highlight current line if it's an item line
      if (isCurrentLine && isItemLine && line.trim() !== "") {
        this.editor.addOverlay(
          bufferId,
          this.namespace,
          lineStart,
          lineEnd,
          colors.selected[0],
          colors.selected[1],
          colors.selected[2],
          true,
          true,
          false
        );
      }

      // Title line
      if (lineNumber === 1) {
        this.editor.addOverlay(
          bufferId,
          this.namespace,
          lineStart,
          lineEnd,
          colors.title[0],
          colors.title[1],
          colors.title[2],
          true,
          true,
          false
        );
      }

      // File header (ends with : but isn't title)
      if (line.endsWith(":") && lineNumber > 1 && !line.startsWith(" ")) {
        this.editor.addOverlay(
          bufferId,
          this.namespace,
          lineStart,
          lineEnd,
          colors.fileHeader[0],
          colors.fileHeader[1],
          colors.fileHeader[2],
          false,
          true,
          false
        );
      }

      // Severity icon highlighting
      const iconMatch = line.match(/^\[([EWIH])\]/);
      if (iconMatch) {
        const iconEnd = lineStart + 3;
        let color: RGB;
        switch (iconMatch[1]) {
          case "E":
            color = colors.error;
            break;
          case "W":
            color = colors.warning;
            break;
          case "I":
            color = colors.info;
            break;
          case "H":
            color = colors.hint;
            break;
          default:
            color = colors.hint;
        }

        this.editor.addOverlay(
          bufferId,
          this.namespace,
          lineStart,
          iconEnd,
          color[0],
          color[1],
          color[2],
          false,
          true,
          false
        );
      }

      // Help line (dimmed)
      if (line.startsWith("Enter:") || line.includes("|")) {
        this.editor.addOverlay(
          bufferId,
          this.namespace,
          lineStart,
          lineEnd,
          colors.help[0],
          colors.help[1],
          colors.help[2],
          false,
          true,
          false
        );
      }

      byteOffset += line.length + 1;
    }
  }
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Get the relative path for display
 */
export function getRelativePath(editor: EditorAPI, filePath: string): string {
  const cwd = editor.getCwd();
  if (filePath.startsWith(cwd)) {
    return filePath.slice(cwd.length + 1);
  }
  return filePath;
}

/**
 * Create a simple static provider from an array of items.
 * Useful for one-shot results like "Find References".
 */
export function createStaticProvider<T extends ResultItem>(
  initialItems: T[] = []
): ResultsProvider<T> & { updateItems: (items: T[]) => void } {
  let items = initialItems;
  const emitter = new EventEmitter<void>();

  return {
    provideResults: () => items,
    onDidChangeResults: emitter.event,
    updateItems: (newItems: T[]) => {
      items = newItems;
      emitter.fireVoid();
    },
  };
}

# Plugin Utilities Library

## Example Plugins

The `plugins/` directory contains several example plugins:

- **`welcome.ts`** - Simple command registration and status messages
- **`todo_highlighter.ts`** - Uses overlays and hooks to highlight keywords efficiently
- **`git_grep.ts`** - Spawns external process and displays results in a virtual buffer

## Plugin Utilities Library

The `plugins/lib/` directory provides reusable utilities that abstract common plugin patterns. Import them with:

```typescript
import { PanelManager, NavigationController, VirtualBufferFactory } from "@plugins/lib";
```

### PanelManager

Manages the lifecycle of result panels (open, close, update, toggle):

```typescript
import { PanelManager } from "@plugins/lib";

const panel = new PanelManager({
  name: "*Search Results*",
  mode: "search-results",
  panelId: "search",
  ratio: 0.3,
  keybindings: [
    ["Return", "search_goto"],
    ["q", "close_buffer"]
  ]
});

// Show results
await panel.open(entries);

// Update with new results
await panel.update(newEntries);

// Toggle visibility
await panel.toggle(entries);

// Check state
if (panel.isOpen()) { ... }
```

### NavigationController

Handles list navigation with selection tracking and visual highlighting:

```typescript
import { NavigationController } from "@plugins/lib";

const nav = new NavigationController({
  bufferId: myBufferId,
  highlightPrefix: "mylist",
  color: { r: 100, g: 100, b: 255 }
});

// Move selection
nav.moveUp();
nav.moveDown();
nav.moveToTop();
nav.moveToBottom();

// Get current selection
const index = nav.getSelectedIndex();
const location = nav.getSelectedLocation();

// Cleanup
nav.clearHighlights();
```

### VirtualBufferFactory

Simplified creation of virtual buffers with less boilerplate:

```typescript
import { VirtualBufferFactory } from "@plugins/lib";

const bufferId = await VirtualBufferFactory.create({
  name: "*Output*",
  mode: "output-mode",
  entries: [
    { text: "Line 1\n", properties: { id: 1 } },
    { text: "Line 2\n", properties: { id: 2 } }
  ],
  readOnly: true,
  ratio: 0.25,
  panelId: "output"
});
```

### Types

The library also exports common types:

```typescript
import type { RGB, Location, PanelOptions, NavigationOptions } from "@plugins/lib";
```

See the source files in `plugins/lib/` for full API details.

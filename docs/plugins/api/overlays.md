# Overlays and Virtual Text API

## Overlay Operations

### `addOverlay`

Add a visual overlay to a buffer range. Overlays persist until explicitly removed.
Use namespaces for batch removal (e.g., "spell", "todo").
Multiple overlays can apply to the same range; colors blend.

```typescript
addOverlay(buffer_id: number, namespace: string, start: number, end: number, r: number, g: number, b: number, bg_r: number, bg_g: number, bg_b: number, underline: boolean, bold: boolean, italic: boolean, extend_to_line_end: boolean): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | Target buffer ID |
| `namespace` | `string` | Optional namespace for grouping (use clearNamespace for batch removal) |
| `start` | `number` | Start byte offset |
| `end` | `number` | End byte offset |
| `r` | `number` | Red (0-255) |
| `g` | `number` | Green (0-255) |
| `b` | `number` | Blue (0-255) |
| `bg_r` | `number` | - |
| `bg_g` | `number` | - |
| `bg_b` | `number` | - |
| `underline` | `boolean` | Add underline decoration |
| `bold` | `boolean` | Use bold text |
| `italic` | `boolean` | Use italic text |
| `extend_to_line_end` | `boolean` | Extend background to end of visual line |

#### `removeOverlay`

Remove a specific overlay by its handle

```typescript
removeOverlay(buffer_id: number, handle: string): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | The buffer ID |
| `handle` | `string` | The overlay handle to remove |

#### `clearOverlaysInRange`

Clear all overlays that overlap with a byte range

```typescript
clearOverlaysInRange(buffer_id: number, start: number, end: number): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | The buffer ID |
| `start` | `number` | Start byte position (inclusive) |
| `end` | `number` | End byte position (exclusive) |

#### `clearAllOverlays`

Remove all overlays from a buffer

```typescript
clearAllOverlays(buffer_id: number): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | The buffer ID |

#### `addVirtualText`

Add virtual text (inline decoration) at a position

```typescript
addVirtualText(buffer_id: number, virtual_text_id: string, position: number, text: string, r: number, g: number, b: number, before: boolean, use_bg: boolean): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | The buffer ID |
| `virtual_text_id` | `string` | Unique identifier for this virtual text |
| `position` | `number` | Byte position to insert at |
| `text` | `string` | The virtual text to display |
| `r` | `number` | Red color component (0-255) |
| `g` | `number` | Green color component (0-255) |
| `b` | `number` | Blue color component (0-255) |
| `before` | `boolean` | Whether to insert before (true) or after (false) the position |
| `use_bg` | `boolean` | Whether to use the color as background (true) or foreground (false) |

#### `removeVirtualText`

Remove virtual text by ID

```typescript
removeVirtualText(buffer_id: number, virtual_text_id: string): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | The buffer ID |
| `virtual_text_id` | `string` | The virtual text ID to remove |

#### `removeVirtualTextsByPrefix`

Remove all virtual texts with IDs starting with a prefix

```typescript
removeVirtualTextsByPrefix(buffer_id: number, prefix: string): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | The buffer ID |
| `prefix` | `string` | The prefix to match virtual text IDs against |

#### `clearVirtualTexts`

Remove all virtual texts from a buffer

```typescript
clearVirtualTexts(buffer_id: number): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | The buffer ID |

#### `clearVirtualTextNamespace`

Clear all virtual texts in a namespace

```typescript
clearVirtualTextNamespace(buffer_id: number, namespace: string): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | The buffer ID |
| `namespace` | `string` | The namespace to clear (e.g., "git-blame") |

#### `refreshLines`

Force a refresh of line display for a buffer

```typescript
refreshLines(buffer_id: number): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | The buffer ID |

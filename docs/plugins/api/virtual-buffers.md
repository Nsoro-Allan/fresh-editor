# Virtual Buffers and Composite Buffers API

## Virtual Buffer Operations

### `createVirtualBufferInSplit`

Create a virtual buffer in a new horizontal split below current pane
Use for results panels, diagnostics, logs, etc. The panel_id enables
idempotent updates: if a panel with that ID exists, its content is replaced
instead of creating a new split. Define the mode with defineMode first.

```typescript
createVirtualBufferInSplit(options: CreateVirtualBufferOptions): Promise<CreateVirtualBufferResult>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `options` | `CreateVirtualBufferOptions` | Buffer configuration |

**Example:**

```typescript
// First define the mode with keybindings
editor.defineMode("search-results", "special", [
["Return", "search_goto"],
["q", "close_buffer"]
], true);

// Then create the buffer
const id = await editor.createVirtualBufferInSplit({
name: "*Search*",
mode: "search-results",
read_only: true,
entries: [
{ text: "src/main.rs:42: match\n", properties: { file: "src/main.rs", line: 42 } }
],
ratio: 0.3,
panel_id: "search"
});
```

### `createVirtualBufferInExistingSplit`

Create a virtual buffer in an existing split

```typescript
createVirtualBufferInExistingSplit(options: CreateVirtualBufferInExistingSplitOptions): Promise<number>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `options` | `CreateVirtualBufferInExistingSplitOptions` | Configuration for the virtual buffer |

### `createVirtualBuffer`

Create a virtual buffer in the current split as a new tab
This is useful for help panels, documentation, etc. that should open
alongside other buffers rather than in a separate split.

```typescript
createVirtualBuffer(options: CreateVirtualBufferInCurrentSplitOptions): Promise<number>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `options` | `CreateVirtualBufferInCurrentSplitOptions` | Configuration for the virtual buffer |

### `defineMode`

Define a buffer mode with keybindings

```typescript
defineMode(name: string, parent: string, bindings: [string, string][], read_only: boolean): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | Mode name (e.g., "diagnostics-list") |
| `parent` | `string` | Parent mode name for inheritance (e.g., "special"), or null |
| `bindings` | `[string, string][]` | Array of [key_string, command_name] pairs |
| `read_only` | `boolean` | Whether buffers in this mode are read-only |

**Example:**

```typescript
editor.defineMode("diagnostics-list", "special", [
["Return", "diagnostics_goto"],
["q", "close_buffer"]
], true);
```

### `showBuffer`

Switch the current split to display a buffer

```typescript
showBuffer(buffer_id: number): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | ID of the buffer to show |

### `closeBuffer`

Close a buffer and remove it from all splits

```typescript
closeBuffer(buffer_id: number): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | ID of the buffer to close |

### `focusSplit`

Focus a specific split

```typescript
focusSplit(split_id: number): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `split_id` | `number` | ID of the split to focus |

### `setSplitBuffer`

Set the buffer displayed in a specific split

```typescript
setSplitBuffer(split_id: number, buffer_id: number): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `split_id` | `number` | ID of the split |
| `buffer_id` | `number` | ID of the buffer to display in the split |

### `closeSplit`

Close a split (if not the last one)

```typescript
closeSplit(split_id: number): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `split_id` | `number` | ID of the split to close |

### `getTextPropertiesAtCursor`

Get text properties at the cursor position in a buffer

```typescript
getTextPropertiesAtCursor(buffer_id: number): Record<string, unknown>[]
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | ID of the buffer to query |

**Example:**

```typescript
const props = editor.getTextPropertiesAtCursor(bufferId);
if (props.length > 0 && props[0].location) {
editor.openFile(props[0].location.file, props[0].location.line, 0);
}
```

### `setVirtualBufferContent`

Set the content of a virtual buffer with text properties

```typescript
setVirtualBufferContent(buffer_id: number, entries: TextPropertyEntry[]): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `buffer_id` | `number` | ID of the virtual buffer |
| `entries` | `TextPropertyEntry[]` | Array of text entries with properties |


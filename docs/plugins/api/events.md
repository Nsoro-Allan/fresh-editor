# Events and Hooks API

## Event/Hook Operations

### `on`

Subscribe to an editor event
Handler must be a global function name (not a closure).
Multiple handlers can be registered for the same event.
Events: "buffer_save", "cursor_moved", "buffer_modified", etc.

```typescript
on(event_name: string, handler_name: string): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `event_name` | `string` | Event to subscribe to |
| `handler_name` | `string` | Name of globalThis function to call with event data |

**Example:**

```typescript
globalThis.onSave = (data) => {
editor.setStatus(`Saved: ${data.path}`);
};
editor.on("buffer_save", "onSave");
```

#### `off`

Unregister an event handler

```typescript
off(event_name: string, handler_name: string): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `event_name` | `string` | Name of the event |
| `handler_name` | `string` | Name of the handler to remove |

#### `getHandlers`

Get list of registered handlers for an event

```typescript
getHandlers(event_name: string): string[]
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `event_name` | `string` | Name of the event |

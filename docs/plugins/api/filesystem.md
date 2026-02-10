# Filesystem, Path, and Environment API

## File System Operations

### `readFile`

Read entire file contents as UTF-8 string
Throws if file doesn't exist, isn't readable, or isn't valid UTF-8.
For binary files, this will fail. For large files, consider memory usage.

```typescript
readFile(path: string): Promise<string>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | File path (absolute or relative to cwd) |

#### `writeFile`

Write string content to a NEW file (fails if file exists)
Creates a new file with the given content. Fails if the file already exists
to prevent plugins from accidentally overwriting user data.

```typescript
writeFile(path: string, content: string): Promise<void>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | Destination path (absolute or relative to cwd) |
| `content` | `string` | UTF-8 string to write |

#### `fileExists`

Check if a path exists (file, directory, or symlink)
Does not follow symlinks; returns true for broken symlinks.
Use fileStat for more detailed information.

```typescript
fileExists(path: string): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | Path to check (absolute or relative to cwd) |

#### `fileStat`

Get metadata about a file or directory
Follows symlinks. Returns exists=false for non-existent paths
rather than throwing. Size is in bytes; directories may report 0.

```typescript
fileStat(path: string): FileStat
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | Path to stat (absolute or relative to cwd) |

#### `readDir`

List directory contents
Returns unsorted entries with type info. Entry names are relative
to the directory (use pathJoin to construct full paths).
Throws on permission errors or if path is not a directory.

```typescript
readDir(path: string): DirEntry[]
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | Directory path (absolute or relative to cwd) |

**Example:**

```typescript
const entries = editor.readDir("/home/user");
for (const e of entries) {
const fullPath = editor.pathJoin("/home/user", e.name);
}
```

### Environment Operations

#### `getEnv`

Get an environment variable

```typescript
getEnv(name: string): string
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | Name of environment variable |

#### `getCwd`

Returns the editor's working directory set at startup. Use as base for resolving relative paths.

```typescript
getCwd(): string
```

### Path Operations

#### `pathJoin`

Join path segments using the OS path separator
Handles empty segments and normalizes separators.
If a segment is absolute, previous segments are discarded.

```typescript
pathJoin(parts: string[]): string
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `parts` | `string[]` | Path segments to join |

**Example:**

```typescript
pathJoin("/home", "user", "file.txt") // "/home/user/file.txt"
pathJoin("relative", "/absolute") // "/absolute"
```

#### `pathDirname`

Get the parent directory of a path
Returns empty string for root paths or paths without parent.
Does not resolve symlinks or check existence.

```typescript
pathDirname(path: string): string
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | File or directory path |

**Example:**

```typescript
pathDirname("/home/user/file.txt") // "/home/user"
pathDirname("/") // ""
```

#### `pathBasename`

Get the final component of a path
Returns empty string for root paths.
Does not strip file extension; use pathExtname for that.

```typescript
pathBasename(path: string): string
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | File or directory path |

**Example:**

```typescript
pathBasename("/home/user/file.txt") // "file.txt"
pathBasename("/home/user/") // "user"
```

#### `pathExtname`

Get the file extension including the dot
Returns empty string if no extension. Only returns the last extension
for files like "archive.tar.gz" (returns ".gz").

```typescript
pathExtname(path: string): string
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | File path |

**Example:**

```typescript
pathExtname("file.txt") // ".txt"
pathExtname("archive.tar.gz") // ".gz"
pathExtname("Makefile") // ""
```

#### `pathIsAbsolute`

Check if a path is absolute
On Unix: starts with "/". On Windows: starts with drive letter or UNC path.

```typescript
pathIsAbsolute(path: string): boolean
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | Path to check |

### Event/Hook Operations
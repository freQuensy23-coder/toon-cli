# TOON Format Specification (v3.0)

**Token-Oriented Object Notation** — a compact, human-readable serialization format
designed to minimize token count for LLM consumption. Lossless replacement for JSON.

## Primitives

### Strings
- **Unquoted** by default: `name: Alice`
- **Quoted** (double quotes) when value:
  - Is empty
  - Looks like a number, boolean, or null (`true`, `false`, `null`, `123`, `1.5`)
  - Has leading/trailing whitespace
  - Contains special characters: `:` `"` `\` `[` `]` `{` `}` `,` `|` or control chars
  - Contains newlines
- Escape sequences (only these 5): `\\` `\"` `\n` `\r` `\t`

### Numbers
- Canonical decimal form: no exponent notation, no trailing zeros
- `-0` normalized to `0`
- Integer and float both supported: `42`, `3.14`, `-7`

### Booleans
- `true` / `false`

### Null
- `null`

## Objects

Key-value pairs, one per line:
```
name: Alice
age: 30
active: true
```

Nested objects use 2-space indentation — parent key ends with `:` alone:
```
user:
  name: Alice
  address:
    city: Berlin
    country: Germany
```

Empty object — key with colon, no children, no value:
```
metadata:
```

## Arrays

All arrays declare their length in brackets: `[N]`.

### Primitive Arrays (inline)
Values comma-separated on the same line:
```
tags[3]: admin,ops,dev
numbers[4]: 1,2,3,4
```

### Tabular Arrays (uniform object arrays)
Header declares length and field names. Each row is comma-separated values:
```
users[2]{id,name,role}:
  1,Alice,admin
  2,Bob,user
```

Rules:
- All objects in the array must have the **same keys** in the **same order**
- All values must be primitives (no nested objects/arrays in cells)
- Row count must match declared length `[N]`

### List Arrays (mixed/non-uniform)
Hyphen prefix, like YAML:
```
items[3]:
  - 42
  - hello
  - true
```

List arrays of objects:
```
entries[2]:
  -
    name: Alice
    age: 30
  -
    name: Bob
    age: 25
```

### Empty Arrays
```
items[0]:
```

### Nested Arrays
Arrays inside list items:
```
matrix[2]:
  - [3]: 1,2,3
  - [3]: 4,5,6
```

## Delimiters

Three options, declared in the array header:
- **Comma** (default): `[N]` or `[N,]`
- **Tab**: `[N\t]`
- **Pipe**: `[N|]`

## Key Folding (optional)

Chains of single-key nested objects collapse to dotted paths:
```
data.metadata.version: 2
```
Instead of:
```
data:
  metadata:
    version: 2
```

## Root Forms

A TOON document can be:
- **Root object** (most common) — key-value pairs at depth 0
- **Root array** — `[N]:` at depth 0
- **Root primitive** — single value

## File Conventions
- Extension: `.toon`
- Media type: `text/toon`
- Encoding: UTF-8 (always)

## JSON-to-TOON Conversion Rules

1. Parse JSON into a value tree
2. For each value, choose the most compact TOON representation:
   - Object → indented key-value pairs
   - Array of primitives → inline `key[N]: v1,v2,...`
   - Array of uniform objects (same keys, all primitive values) → tabular
   - Array of non-uniform items or items with nested structures → list format
   - Primitives → direct value encoding
3. Apply string quoting rules to all string values and keys
4. Output with 2-space indentation

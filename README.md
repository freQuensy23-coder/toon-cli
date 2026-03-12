# toon-cli

Blazing-fast JSON-to-TOON converter. Written in Rust with simd-json.

**TOON** (Token-Oriented Object Notation) is a compact format that reduces token count for LLM consumption — typically 35-50% smaller than JSON.

## Install

```bash
cargo install --path .
```

## Usage

```bash
# From file
toon-cli data.json

# From stdin (pipe-friendly)
curl -s https://api.example.com/users | toon-cli

# Write to file
toon-cli data.json -o data.toon

# Inline
echo '{"name":"Alice","age":30}' | toon-cli
```

## Example

```
$ echo '[{"id":1,"name":"Alice","role":"admin"},{"id":2,"name":"Bob","role":"user"}]' | toon-cli

[2]{id,name,role}:
  1,Alice,admin
  2,Bob,user
```

JSON (76 bytes) → TOON (42 bytes) — **45% smaller**.

## Performance

| Input | Size | Time | Throughput |
|-------|------|------|------------|
| 100k rows tabular | 10.2 MB | 47ms | ~217 MB/s |
| 10k rows tabular | 1.0 MB | 2.9ms | ~345 MB/s |
| Mixed API response | 400 B | 462ns | ~824 MB/s |

Optimizations: simd-json parsing, itoa/ryu number formatting, lookup-table quoting, pre-computed indents, bulk-copy string escaping.

## TOON Format

See [TOON_SPEC.md](TOON_SPEC.md) for the full format specification.

Key features:
- **Objects**: `key: value` with 2-space indent nesting
- **Tabular arrays**: `[N]{fields}:` header + CSV rows (biggest savings)
- **Primitive arrays**: `key[N]: v1,v2,v3` inline
- **List arrays**: `- item` for mixed/nested content

## License

MIT

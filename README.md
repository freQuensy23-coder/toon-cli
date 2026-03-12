# toon-cli

Fast JSON to [TOON](https://toonformat.dev) converter. Rust, simd-json.

TOON is a compact serialization format for LLM contexts — same data, fewer tokens.

## Install

```bash
cargo install --path .
```

Or grab a binary from [Releases](https://github.com/freQuensy23-coder/toon-cli/releases).

## Usage

```bash
# file
toon-cli data.json

# pipe
curl -s api.example.com/users | toon-cli

# output to file
toon-cli data.json -o data.toon
```

## What it does

```
$ echo '[{"id":1,"name":"Alice","role":"admin"},{"id":2,"name":"Bob","role":"user"}]' | toon-cli

[2]{id,name,role}:
  1,Alice,admin
  2,Bob,user
```

76 bytes JSON → 42 bytes TOON.

## Speed

10 MB JSON (100k rows) converts in ~47ms. See [TOON_SPEC.md](TOON_SPEC.md) for format details.

## License

MIT

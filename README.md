# toon-cli

Fast JSON to [TOON](https://toonformat.dev) converter. Rust, simd-json.

TOON is a compact serialization format for LLM contexts — same data, fewer tokens.

## Install

**Homebrew** (macOS / Linux):
```bash
brew install freQuensy23-coder/tap/toon-cli
```

**Cargo**:
```bash
cargo install --path .
```

**Binary**: grab from [Releases](https://github.com/freQuensy23-coder/toon-cli/releases).

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

10 MB JSON (100k rows) converts in ~47ms. Output is byte-identical to the reference [toon-js](https://github.com/toon-format/toon) implementation.

Run `benches/compare.sh` to verify against Node.js SDK. See [TOON_SPEC.md](TOON_SPEC.md) for format details.

## License

MIT

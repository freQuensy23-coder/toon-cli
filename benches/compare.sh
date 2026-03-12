#!/bin/bash
# Compare toon-cli (Rust) output against @toon-format/toon (Node.js) reference
# Usage: ./benches/compare.sh [json_file]
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
INPUT="${1:-$SCRIPT_DIR/mock_data.json}"
WORK="$(mktemp -d)"
trap "rm -rf $WORK" EXIT

echo "=== toon-cli vs toon-js comparison ==="
echo "Input: $INPUT ($(wc -c < "$INPUT" | tr -d ' ') bytes)"
echo ""

# Build Rust
echo "Building toon-cli..."
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml" 2>/dev/null

# Install Node SDK
echo "Installing @toon-format/toon..."
(cd "$WORK" && npm init -y &>/dev/null && npm install @toon-format/toon &>/dev/null)

# Generate Rust output + time
echo "Running toon-cli..."
RUST_START=$(python3 -c "import time; print(time.perf_counter())")
"$ROOT_DIR/target/release/toon-cli" "$INPUT" -o "$WORK/rust.toon"
RUST_END=$(python3 -c "import time; print(time.perf_counter())")
RUST_MS=$(python3 -c "print(f'{($RUST_END - $RUST_START) * 1000:.2f}')")

# Generate Node output + time
echo "Running toon-js..."
cat > "$WORK/bench.mjs" << 'NODESCRIPT'
import { encode } from '@toon-format/toon';
import { readFileSync, writeFileSync } from 'fs';
const json = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const start = performance.now();
const toon = encode(json);
const elapsed = performance.now() - start;
writeFileSync(process.argv[3], toon + '\n');
console.log(elapsed.toFixed(2));
NODESCRIPT
NODE_MS=$(cd "$WORK" && node bench.mjs "$INPUT" "$WORK/node.toon")

# Compare
echo ""
echo "=== Results ==="
echo "Rust:  $RUST_MS ms (full pipeline: read + parse + encode + write)"
echo "Node:  $NODE_MS ms (encode only)"
echo ""

RUST_SIZE=$(wc -c < "$WORK/rust.toon" | tr -d ' ')
NODE_SIZE=$(wc -c < "$WORK/node.toon" | tr -d ' ')
INPUT_SIZE=$(wc -c < "$INPUT" | tr -d ' ')
echo "Input:  $INPUT_SIZE bytes"
echo "Rust:   $RUST_SIZE bytes"
echo "Node:   $NODE_SIZE bytes"

if diff -q "$WORK/rust.toon" "$WORK/node.toon" >/dev/null 2>&1; then
    HASH=$(md5 -q "$WORK/rust.toon" 2>/dev/null || md5sum "$WORK/rust.toon" | cut -d' ' -f1)
    echo ""
    echo "RESULT: IDENTICAL (md5: $HASH)"
else
    echo ""
    echo "RESULT: DIFFER"
    echo "First differences:"
    diff "$WORK/rust.toon" "$WORK/node.toon" | head -20
    exit 1
fi

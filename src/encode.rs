use simd_json::OwnedValue;
use simd_json::prelude::*;
use std::fmt::Write;

/// Encode a JSON value tree into TOON format.
/// Returns the full TOON string (no trailing newline — caller adds if needed).
pub fn encode(value: &OwnedValue) -> String {
    let mut buf = String::with_capacity(4096);
    encode_root(value, &mut buf);
    buf
}

fn encode_root(value: &OwnedValue, buf: &mut String) {
    match value {
        OwnedValue::Object(_) => {
            encode_object_fields(value, buf, 0);
        }
        OwnedValue::Array(arr) => {
            encode_root_array(arr.as_slice(), buf);
        }
        _ => {
            encode_primitive(value, buf, QuoteCtx::Value);
        }
    }
}

fn encode_root_array(arr: &[OwnedValue], buf: &mut String) {
    let len = arr.len();
    if len == 0 {
        buf.push_str("[0]:");
        return;
    }
    match classify_array(arr) {
        ArrayKind::Primitives => {
            write!(buf, "[{}]: ", len).unwrap();
            encode_inline_values(arr, buf);
        }
        ArrayKind::Tabular(keys) => {
            write!(buf, "[{}]{{{}}}:", len, keys.join(",")).unwrap();
            for item in arr {
                buf.push('\n');
                push_indent(buf, 1);
                encode_tabular_row(item, &keys, buf);
            }
        }
        ArrayKind::List => {
            write!(buf, "[{}]:", len).unwrap();
            for item in arr {
                buf.push('\n');
                encode_list_item(item, buf, 1);
            }
        }
    }
}

fn encode_object_fields(value: &OwnedValue, buf: &mut String, depth: usize) {
    if let OwnedValue::Object(obj) = value {
        let mut first = true;
        for (key, val) in obj.iter() {
            if !first {
                buf.push('\n');
            }
            first = false;
            push_indent(buf, depth);
            encode_key_value(key, val, buf, depth);
        }
    }
}

fn encode_key_value(key: &str, val: &OwnedValue, buf: &mut String, depth: usize) {
    match val {
        OwnedValue::Object(obj) => {
            encode_key(key, buf);
            buf.push(':');
            if !obj.is_empty() {
                buf.push('\n');
                encode_object_fields(val, buf, depth + 1);
            }
        }
        OwnedValue::Array(arr) => {
            encode_array_field(key, arr.as_slice(), buf, depth);
        }
        _ => {
            encode_key(key, buf);
            buf.push_str(": ");
            encode_primitive(val, buf, QuoteCtx::Value);
        }
    }
}

fn encode_array_field(key: &str, arr: &[OwnedValue], buf: &mut String, depth: usize) {
    let len = arr.len();
    if len == 0 {
        encode_key(key, buf);
        buf.push_str("[0]:");
        return;
    }
    match classify_array(arr) {
        ArrayKind::Primitives => {
            encode_key(key, buf);
            write!(buf, "[{}]: ", len).unwrap();
            encode_inline_values(arr, buf);
        }
        ArrayKind::Tabular(keys) => {
            encode_key(key, buf);
            write!(buf, "[{}]{{{}}}:", len, keys.join(",")).unwrap();
            for item in arr {
                buf.push('\n');
                push_indent(buf, depth + 1);
                encode_tabular_row(item, &keys, buf);
            }
        }
        ArrayKind::List => {
            encode_key(key, buf);
            write!(buf, "[{}]:", len).unwrap();
            for item in arr {
                buf.push('\n');
                encode_list_item(item, buf, depth + 1);
            }
        }
    }
}

fn encode_inline_values(arr: &[OwnedValue], buf: &mut String) {
    for (i, v) in arr.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        encode_primitive(v, buf, QuoteCtx::InlineArray);
    }
}

fn encode_tabular_row(item: &OwnedValue, keys: &[String], buf: &mut String) {
    if let OwnedValue::Object(obj) = item {
        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                buf.push(',');
            }
            if let Some(val) = obj.get(key.as_str()) {
                encode_primitive(val, buf, QuoteCtx::TabularCell);
            } else {
                buf.push_str("null");
            }
        }
    }
}

fn encode_list_item(item: &OwnedValue, buf: &mut String, depth: usize) {
    push_indent(buf, depth);
    match item {
        OwnedValue::Object(obj) => {
            buf.push('-');
            if obj.is_empty() {
                return;
            }
            buf.push('\n');
            encode_object_fields(item, buf, depth + 1);
        }
        OwnedValue::Array(arr) => {
            buf.push_str("- ");
            let arr = arr.as_slice();
            let len = arr.len();
            if len == 0 {
                buf.push_str("[0]:");
                return;
            }
            match classify_array(arr) {
                ArrayKind::Primitives => {
                    write!(buf, "[{}]: ", len).unwrap();
                    encode_inline_values(arr, buf);
                }
                ArrayKind::Tabular(keys) => {
                    write!(buf, "[{}]{{{}}}:", len, keys.join(",")).unwrap();
                    for sub in arr {
                        buf.push('\n');
                        push_indent(buf, depth + 2);
                        encode_tabular_row(sub, &keys, buf);
                    }
                }
                ArrayKind::List => {
                    write!(buf, "[{}]:", len).unwrap();
                    for sub in arr {
                        buf.push('\n');
                        encode_list_item(sub, buf, depth + 2);
                    }
                }
            }
        }
        _ => {
            buf.push_str("- ");
            encode_primitive(item, buf, QuoteCtx::Value);
        }
    }
}

// ─── Primitive encoding ──────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum QuoteCtx {
    Value,       // standalone value or object field value
    InlineArray, // element inside a primitive inline array
    TabularCell, // cell inside a tabular row
}

fn encode_primitive(val: &OwnedValue, buf: &mut String, ctx: QuoteCtx) {
    match val {
        OwnedValue::Static(s) => match s {
            simd_json::StaticNode::Null => buf.push_str("null"),
            simd_json::StaticNode::Bool(b) => {
                buf.push_str(if *b { "true" } else { "false" });
            }
            simd_json::StaticNode::I64(n) => write!(buf, "{}", n).unwrap(),
            simd_json::StaticNode::U64(n) => write!(buf, "{}", n).unwrap(),
            simd_json::StaticNode::F64(f) => encode_float(*f, buf),
            #[allow(unreachable_patterns)]
            _ => buf.push_str("null"),
        },
        OwnedValue::String(s) => encode_string(s, buf, ctx),
        _ => buf.push_str("null"), // arrays/objects shouldn't reach here
    }
}

fn encode_float(f: f64, buf: &mut String) {
    if f == 0.0 && f.is_sign_negative() {
        buf.push('0');
        return;
    }
    // Use ryu for fast float formatting
    let mut b = ryu::Buffer::new();
    let s = b.format(f);
    // ryu may produce scientific notation — normalize
    if s.contains('e') || s.contains('E') {
        // Fallback to standard formatting
        write!(buf, "{}", f).unwrap();
    } else {
        buf.push_str(s);
    }
}

fn encode_string(s: &str, buf: &mut String, ctx: QuoteCtx) {
    if needs_quoting(s, ctx) {
        buf.push('"');
        for c in s.chars() {
            match c {
                '"' => buf.push_str("\\\""),
                '\\' => buf.push_str("\\\\"),
                '\n' => buf.push_str("\\n"),
                '\r' => buf.push_str("\\r"),
                '\t' => buf.push_str("\\t"),
                _ => buf.push(c),
            }
        }
        buf.push('"');
    } else {
        buf.push_str(s);
    }
}

fn needs_quoting(s: &str, ctx: QuoteCtx) -> bool {
    if s.is_empty() {
        return true;
    }
    // Leading/trailing whitespace
    let bytes = s.as_bytes();
    if bytes[0] == b' ' || bytes[bytes.len() - 1] == b' ' {
        return true;
    }
    // Looks like number, bool, or null
    if looks_like_literal(s) {
        return true;
    }
    // In inline array or tabular context, commas need quoting
    if ctx == QuoteCtx::InlineArray || ctx == QuoteCtx::TabularCell {
        if s.contains(',') {
            return true;
        }
    }
    // Special chars
    for c in s.chars() {
        match c {
            ':' | '"' | '\\' | '[' | ']' | '{' | '}' | '|' => return true,
            '\n' | '\r' | '\t' => return true,
            c if c.is_control() => return true,
            _ => {}
        }
    }
    false
}

fn looks_like_literal(s: &str) -> bool {
    if s == "true" || s == "false" || s == "null" {
        return true;
    }
    looks_like_number(s)
}

fn looks_like_number(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let bytes = s.as_bytes();
    let mut i = 0;
    if bytes[i] == b'-' {
        i += 1;
        if i >= bytes.len() {
            return false;
        }
    }
    if i >= bytes.len() {
        return false;
    }
    // Must start with digit
    if !bytes[i].is_ascii_digit() {
        return false;
    }
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        if i >= bytes.len() || !bytes[i].is_ascii_digit() {
            return false;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    // Allow exponent notation to also be caught
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        if i >= bytes.len() || !bytes[i].is_ascii_digit() {
            return false;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    i == bytes.len()
}

// ─── Key encoding ────────────────────────────────────────────

fn encode_key(key: &str, buf: &mut String) {
    // Keys follow similar quoting rules but in key position
    if key_needs_quoting(key) {
        buf.push('"');
        for c in key.chars() {
            match c {
                '"' => buf.push_str("\\\""),
                '\\' => buf.push_str("\\\\"),
                '\n' => buf.push_str("\\n"),
                '\r' => buf.push_str("\\r"),
                '\t' => buf.push_str("\\t"),
                _ => buf.push(c),
            }
        }
        buf.push('"');
    } else {
        buf.push_str(key);
    }
}

fn key_needs_quoting(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    for c in s.chars() {
        match c {
            ':' | '"' | '\\' | '[' | ']' | '{' | '}' | ',' | '|' => return true,
            '\n' | '\r' | '\t' => return true,
            ' ' => return true,
            c if c.is_control() => return true,
            _ => {}
        }
    }
    false
}

// ─── Array classification ────────────────────────────────────

enum ArrayKind {
    Primitives,
    Tabular(Vec<String>),
    List,
}

fn classify_array(arr: &[OwnedValue]) -> ArrayKind {
    if arr.is_empty() {
        return ArrayKind::Primitives;
    }

    // Check if all elements are primitives (not objects, not arrays)
    let all_primitives = arr.iter().all(|v| is_primitive(v));
    if all_primitives {
        // Check if all the same type — if mixed types, use list format
        if is_uniform_primitive_types(arr) {
            return ArrayKind::Primitives;
        } else {
            return ArrayKind::List;
        }
    }

    // Check if all elements are objects with same keys and all primitive values → tabular
    if let Some(keys) = try_tabular(arr) {
        return ArrayKind::Tabular(keys);
    }

    ArrayKind::List
}

fn is_primitive(v: &OwnedValue) -> bool {
    matches!(v, OwnedValue::Static(_) | OwnedValue::String(_))
}

fn is_uniform_primitive_types(arr: &[OwnedValue]) -> bool {
    // All same "kind" — all numbers, all strings, all bools, all null, or all primitives
    // For TOON, primitive arrays can have mixed primitive types BUT
    // mixed types (string + number) should still be inline if they're all primitives.
    // Actually, looking at the spec more carefully, mixed primitive types go to list format.
    // Let me check the test: [1,"hello",true] → list format.
    // So we need to check if types are uniform.

    let first_kind = prim_kind(&arr[0]);
    arr.iter().all(|v| prim_kind(v) == first_kind)
}

#[derive(PartialEq)]
enum PrimKind {
    Number,
    String,
    Bool,
    Null,
}

fn prim_kind(v: &OwnedValue) -> PrimKind {
    match v {
        OwnedValue::String(_) => PrimKind::String,
        OwnedValue::Static(s) => match s {
            simd_json::StaticNode::Null => PrimKind::Null,
            simd_json::StaticNode::Bool(_) => PrimKind::Bool,
            _ => PrimKind::Number,
        },
        _ => PrimKind::Null,
    }
}

fn try_tabular(arr: &[OwnedValue]) -> Option<Vec<String>> {
    // All must be objects
    let first_obj = match &arr[0] {
        OwnedValue::Object(o) => o,
        _ => return None,
    };

    // Extract key order from first object
    let keys: Vec<String> = first_obj.keys().map(|k| k.to_string()).collect();
    if keys.is_empty() {
        return None;
    }

    // All values in first must be primitive
    if !first_obj.values().all(|v| is_primitive(v)) {
        return None;
    }

    // All other objects must have exact same keys in same order, all primitive values
    for item in &arr[1..] {
        let obj = match item {
            OwnedValue::Object(o) => o,
            _ => return None,
        };
        let item_keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        if item_keys.len() != keys.len() {
            return None;
        }
        for (a, b) in keys.iter().zip(item_keys.iter()) {
            if a.as_str() != *b {
                return None;
            }
        }
        if !obj.values().all(|v| is_primitive(v)) {
            return None;
        }
    }

    Some(keys)
}

// ─── Helpers ─────────────────────────────────────────────────

#[inline]
fn push_indent(buf: &mut String, depth: usize) {
    for _ in 0..depth {
        buf.push_str("  ");
    }
}

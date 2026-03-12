use simd_json::OwnedValue;

// ─── Lookup table for bytes that need quoting in value context ──
static NEEDS_QUOTE_VALUE: [bool; 256] = {
    let mut t = [false; 256];
    t[b':' as usize] = true;
    t[b'"' as usize] = true;
    t[b'\\' as usize] = true;
    t[b'[' as usize] = true;
    t[b']' as usize] = true;
    t[b'{' as usize] = true;
    t[b'}' as usize] = true;
    t[b'|' as usize] = true;
    let mut i = 0u8;
    while i < 32 {
        t[i as usize] = true;
        i += 1;
    }
    t[127] = true;
    t
};

// Key quoting lookup — same as value but also includes space and comma
static NEEDS_QUOTE_KEY: [bool; 256] = {
    let mut t = NEEDS_QUOTE_VALUE;
    t[b' ' as usize] = true;
    t[b',' as usize] = true;
    t
};

/// Encode a JSON value tree into TOON format.
pub fn encode(value: &OwnedValue) -> String {
    let mut buf = String::with_capacity(4096);
    encode_root(value, &mut buf);
    buf
}

#[inline]
fn encode_root(value: &OwnedValue, buf: &mut String) {
    match value {
        OwnedValue::Object(_) => encode_object_fields(value, buf, 0),
        OwnedValue::Array(arr) => encode_root_array(arr.as_slice(), buf),
        _ => encode_primitive(value, buf, QC_VALUE),
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
            push_array_len(buf, len);
            buf.push_str(": ");
            encode_inline_values(arr, buf);
        }
        ArrayKind::Tabular(keys) => {
            push_tabular_header(buf, len, &keys);
            for item in arr {
                buf.push('\n');
                push_indent(buf, 1);
                encode_tabular_row(item, &keys, buf);
            }
        }
        ArrayKind::List => {
            push_array_len(buf, len);
            buf.push(':');
            for item in arr {
                buf.push('\n');
                encode_list_item_value(item, buf, 1);
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
            encode_primitive(val, buf, QC_VALUE);
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
            push_array_len(buf, len);
            buf.push_str(": ");
            encode_inline_values(arr, buf);
        }
        ArrayKind::Tabular(keys) => {
            encode_key(key, buf);
            push_tabular_header(buf, len, &keys);
            let d1 = depth + 1;
            for item in arr {
                buf.push('\n');
                push_indent(buf, d1);
                encode_tabular_row(item, &keys, buf);
            }
        }
        ArrayKind::List => {
            encode_key(key, buf);
            push_array_len(buf, len);
            buf.push(':');
            for item in arr {
                buf.push('\n');
                encode_list_item_value(item, buf, depth + 1);
            }
        }
    }
}

#[inline]
fn encode_inline_values(arr: &[OwnedValue], buf: &mut String) {
    let mut first = true;
    for v in arr {
        if !first {
            buf.push(',');
        }
        first = false;
        encode_primitive(v, buf, QC_ARRAY);
    }
}

#[inline]
fn encode_tabular_row(item: &OwnedValue, keys: &[String], buf: &mut String) {
    if let OwnedValue::Object(obj) = item {
        let mut first = true;
        for key in keys {
            if !first {
                buf.push(',');
            }
            first = false;
            if let Some(val) = obj.get(key.as_str()) {
                encode_primitive(val, buf, QC_ARRAY);
            } else {
                buf.push_str("null");
            }
        }
    }
}

// ─── List item encoding (matching toon-js reference) ─────────
//
// For objects: first key-value goes on same line as "- ", rest at depth+1
// Example:
//   - name: Alice
//     age: 30
//     addr:
//       city: NY

fn encode_list_item_value(item: &OwnedValue, buf: &mut String, depth: usize) {
    match item {
        OwnedValue::Object(obj) => {
            encode_object_as_list_item(obj, item, buf, depth);
        }
        OwnedValue::Array(arr) => {
            let arr = arr.as_slice();
            if all_primitives(arr) {
                // Inline primitive array as list item
                push_indent(buf, depth);
                buf.push_str("- ");
                let len = arr.len();
                if len == 0 {
                    buf.push_str("[0]:");
                } else {
                    push_array_len(buf, len);
                    buf.push_str(": ");
                    encode_inline_values(arr, buf);
                }
            } else {
                push_indent(buf, depth);
                buf.push_str("- ");
                let len = arr.len();
                if len == 0 {
                    buf.push_str("[0]:");
                } else {
                    push_array_len(buf, len);
                    buf.push(':');
                    for sub in arr {
                        buf.push('\n');
                        encode_list_item_value(sub, buf, depth + 1);
                    }
                }
            }
        }
        _ => {
            push_indent(buf, depth);
            buf.push_str("- ");
            encode_primitive(item, buf, QC_VALUE);
        }
    }
}

fn encode_object_as_list_item(
    obj: &simd_json::owned::Object,
    _full_value: &OwnedValue,
    buf: &mut String,
    depth: usize,
) {
    if obj.is_empty() {
        push_indent(buf, depth);
        buf.push('-');
        return;
    }

    let mut iter = obj.iter();
    let (first_key, first_val) = iter.next().unwrap();

    // Emit "- " prefix + first key-value on same line
    push_indent(buf, depth);
    buf.push_str("- ");
    encode_first_field(first_key, first_val, buf, depth);

    // Remaining fields at depth + 1
    for (key, val) in iter {
        buf.push('\n');
        push_indent(buf, depth + 1);
        encode_key_value(key, val, buf, depth + 1);
    }
}

/// Encode the first field of a list-item object (on the same line as "- ")
fn encode_first_field(key: &str, val: &OwnedValue, buf: &mut String, depth: usize) {
    match val {
        OwnedValue::Object(obj) => {
            // - key:
            //     nested_key: value
            encode_key(key, buf);
            buf.push(':');
            if !obj.is_empty() {
                buf.push('\n');
                encode_object_fields(val, buf, depth + 2);
            }
        }
        OwnedValue::Array(arr) => {
            let arr_slice = arr.as_slice();
            let len = arr_slice.len();
            if len == 0 {
                encode_key(key, buf);
                buf.push_str("[0]:");
            } else if all_primitives(arr_slice) {
                encode_key(key, buf);
                push_array_len(buf, len);
                buf.push_str(": ");
                encode_inline_values(arr_slice, buf);
            } else if all_objects(arr_slice) {
                if let Some(keys) = try_tabular(arr_slice) {
                    // Tabular as first field
                    encode_key(key, buf);
                    push_tabular_header(buf, len, &keys);
                    for item in arr_slice {
                        buf.push('\n');
                        push_indent(buf, depth + 2);
                        encode_tabular_row(item, &keys, buf);
                    }
                } else {
                    // Non-tabular object array
                    encode_key(key, buf);
                    push_array_len(buf, len);
                    buf.push(':');
                    for item in arr_slice {
                        buf.push('\n');
                        encode_list_item_value(item, buf, depth + 2);
                    }
                }
            } else {
                // Mixed array
                encode_key(key, buf);
                push_array_len(buf, len);
                buf.push(':');
                for item in arr_slice {
                    buf.push('\n');
                    encode_list_item_value(item, buf, depth + 2);
                }
            }
        }
        _ => {
            encode_key(key, buf);
            buf.push_str(": ");
            encode_primitive(val, buf, QC_VALUE);
        }
    }
}

// ─── Primitive encoding ──────────────────────────────────────

const QC_VALUE: u8 = 0;
const QC_ARRAY: u8 = 1;

#[inline]
fn encode_primitive(val: &OwnedValue, buf: &mut String, ctx: u8) {
    match val {
        OwnedValue::Static(s) => match s {
            simd_json::StaticNode::Null => buf.push_str("null"),
            simd_json::StaticNode::Bool(b) => {
                buf.push_str(if *b { "true" } else { "false" });
            }
            simd_json::StaticNode::I64(n) => push_i64(buf, *n),
            simd_json::StaticNode::U64(n) => push_u64(buf, *n),
            simd_json::StaticNode::F64(f) => encode_float(*f, buf),
            #[allow(unreachable_patterns)]
            _ => buf.push_str("null"),
        },
        OwnedValue::String(s) => encode_string(s, buf, ctx),
        _ => buf.push_str("null"),
    }
}

#[inline]
fn push_i64(buf: &mut String, n: i64) {
    let mut b = itoa::Buffer::new();
    buf.push_str(b.format(n));
}

#[inline]
fn push_u64(buf: &mut String, n: u64) {
    let mut b = itoa::Buffer::new();
    buf.push_str(b.format(n));
}

#[inline]
fn push_usize(buf: &mut String, n: usize) {
    let mut b = itoa::Buffer::new();
    buf.push_str(b.format(n));
}

#[inline]
fn encode_float(f: f64, buf: &mut String) {
    if f == 0.0 && f.is_sign_negative() {
        buf.push('0');
        return;
    }
    // If it's a whole number, output as integer (matching toon-js)
    if f.fract() == 0.0 && f.is_finite() && f.abs() < (i64::MAX as f64) {
        push_i64(buf, f as i64);
        return;
    }
    let mut b = ryu::Buffer::new();
    let s = b.format(f);
    if memchr::memchr2(b'e', b'E', s.as_bytes()).is_some() {
        use std::fmt::Write;
        write!(buf, "{}", f).unwrap();
    } else {
        buf.push_str(s);
    }
}

#[inline]
fn encode_string(s: &str, buf: &mut String, ctx: u8) {
    if needs_quoting(s, ctx) {
        buf.push('"');
        escape_into(s, buf);
        buf.push('"');
    } else {
        buf.push_str(s);
    }
}

#[inline]
fn escape_into(s: &str, buf: &mut String) {
    let bytes = s.as_bytes();
    let mut start = 0;
    for (i, &b) in bytes.iter().enumerate() {
        let esc = match b {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\n' => "\\n",
            b'\r' => "\\r",
            b'\t' => "\\t",
            _ => continue,
        };
        if start < i {
            buf.push_str(&s[start..i]);
        }
        buf.push_str(esc);
        start = i + 1;
    }
    if start < bytes.len() {
        buf.push_str(&s[start..]);
    }
}

#[inline]
fn needs_quoting(s: &str, ctx: u8) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return true;
    }
    if bytes[0] == b' ' || bytes[bytes.len() - 1] == b' ' {
        return true;
    }
    if looks_like_literal(bytes) {
        return true;
    }
    for &b in bytes {
        if NEEDS_QUOTE_VALUE[b as usize] {
            return true;
        }
        if ctx == QC_ARRAY && b == b',' {
            return true;
        }
    }
    false
}

#[inline]
fn looks_like_literal(bytes: &[u8]) -> bool {
    match bytes.len() {
        4 if bytes == b"true" || bytes == b"null" => return true,
        5 if bytes == b"false" => return true,
        _ => {}
    }
    looks_like_number(bytes)
}

#[inline]
fn looks_like_number(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    if bytes[i] == b'-' {
        i += 1;
        if i >= bytes.len() {
            return false;
        }
    }
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

#[inline]
fn encode_key(key: &str, buf: &mut String) {
    if key_needs_quoting(key) {
        buf.push('"');
        escape_into(key, buf);
        buf.push('"');
    } else {
        buf.push_str(key);
    }
}

#[inline]
fn key_needs_quoting(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return true;
    }
    for &b in bytes {
        if NEEDS_QUOTE_KEY[b as usize] {
            return true;
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

#[inline]
fn classify_array(arr: &[OwnedValue]) -> ArrayKind {
    debug_assert!(!arr.is_empty());

    // Check if ALL elements are primitives (mixed types OK, matching toon-js)
    if all_primitives(arr) {
        return ArrayKind::Primitives;
    }

    // Check if all elements are arrays of primitives
    if all_primitive_arrays(arr) {
        return ArrayKind::List; // array-of-arrays → list with inline sub-arrays
    }

    // Check if all elements are objects → try tabular
    if all_objects(arr) {
        if let Some(keys) = try_tabular(arr) {
            return ArrayKind::Tabular(keys);
        }
    }

    ArrayKind::List
}

#[inline]
fn is_primitive(v: &OwnedValue) -> bool {
    matches!(v, OwnedValue::Static(_) | OwnedValue::String(_))
}

#[inline]
fn all_primitives(arr: &[OwnedValue]) -> bool {
    arr.iter().all(|v| is_primitive(v))
}

#[inline]
fn all_objects(arr: &[OwnedValue]) -> bool {
    arr.iter().all(|v| matches!(v, OwnedValue::Object(_)))
}

#[inline]
fn all_primitive_arrays(arr: &[OwnedValue]) -> bool {
    arr.iter().all(|v| {
        if let OwnedValue::Array(a) = v {
            all_primitives(a.as_slice())
        } else {
            false
        }
    })
}

fn try_tabular(arr: &[OwnedValue]) -> Option<Vec<String>> {
    let first_obj = match &arr[0] {
        OwnedValue::Object(o) => o,
        _ => return None,
    };

    let keys: Vec<String> = first_obj.keys().map(|k| k.to_string()).collect();
    if keys.is_empty() {
        return None;
    }

    for v in first_obj.values() {
        if !is_primitive(v) {
            return None;
        }
    }

    let key_count = keys.len();
    for item in &arr[1..] {
        let obj = match item {
            OwnedValue::Object(o) => o,
            _ => return None,
        };
        if obj.len() != key_count {
            return None;
        }
        for (a, (b, v)) in keys.iter().zip(obj.iter()) {
            if a.as_str() != b {
                return None;
            }
            if !is_primitive(v) {
                return None;
            }
        }
    }

    Some(keys)
}

// ─── Helpers ─────────────────────────────────────────────────

static INDENTS: [&str; 17] = [
    "",
    "  ",
    "    ",
    "      ",
    "        ",
    "          ",
    "            ",
    "              ",
    "                ",
    "                  ",
    "                    ",
    "                      ",
    "                        ",
    "                          ",
    "                            ",
    "                              ",
    "                                ",
];

#[inline]
fn push_indent(buf: &mut String, depth: usize) {
    if depth < INDENTS.len() {
        buf.push_str(INDENTS[depth]);
    } else {
        for _ in 0..depth {
            buf.push_str("  ");
        }
    }
}

#[inline]
fn push_array_len(buf: &mut String, len: usize) {
    buf.push('[');
    push_usize(buf, len);
    buf.push(']');
}

#[inline]
fn push_tabular_header(buf: &mut String, len: usize, keys: &[String]) {
    buf.push('[');
    push_usize(buf, len);
    buf.push_str("]{");
    let mut first = true;
    for k in keys {
        if !first {
            buf.push(',');
        }
        first = false;
        buf.push_str(k);
    }
    buf.push_str("}:");
}

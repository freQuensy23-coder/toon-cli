use simd_json::OwnedValue;

// ─── Lookup table for bytes that need quoting in value context ──
// Set bit for: : " \ [ ] { } | and control chars (0..32) plus DEL (127)
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
    // Control chars
    let mut i = 0u8;
    while i < 32 {
        t[i as usize] = true;
        i += 1;
    }
    t[127] = true;
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
                buf.push_str("  ");
                encode_tabular_row(item, &keys, buf);
            }
        }
        ArrayKind::List => {
            push_array_len(buf, len);
            buf.push(':');
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
                encode_list_item(item, buf, depth + 1);
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
                    push_array_len(buf, len);
                    buf.push_str(": ");
                    encode_inline_values(arr, buf);
                }
                ArrayKind::Tabular(keys) => {
                    push_tabular_header(buf, len, &keys);
                    let d2 = depth + 2;
                    for sub in arr {
                        buf.push('\n');
                        push_indent(buf, d2);
                        encode_tabular_row(sub, &keys, buf);
                    }
                }
                ArrayKind::List => {
                    push_array_len(buf, len);
                    buf.push(':');
                    for sub in arr {
                        buf.push('\n');
                        encode_list_item(sub, buf, depth + 2);
                    }
                }
            }
        }
        _ => {
            buf.push_str("- ");
            encode_primitive(item, buf, QC_VALUE);
        }
    }
}

// ─── Primitive encoding ──────────────────────────────────────

// QuoteContext as u8 for zero-overhead dispatch
const QC_VALUE: u8 = 0;
const QC_ARRAY: u8 = 1; // both inline array and tabular cell

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

/// Fast escape: only 5 sequences (\\, \", \n, \r, \t).
/// Scans for runs of safe bytes and copies them in bulk.
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
            // SAFETY: we're slicing on ASCII byte boundaries
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
    // Leading/trailing space
    if bytes[0] == b' ' || bytes[bytes.len() - 1] == b' ' {
        return true;
    }
    // Looks like literal
    if looks_like_literal(bytes) {
        return true;
    }
    // Scan bytes — all ASCII special chars are single-byte
    for &b in bytes {
        if NEEDS_QUOTE_VALUE[b as usize] {
            return true;
        }
        // In array/tabular context, comma needs quoting
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

// Key quoting lookup — same as value but also includes space and comma
static NEEDS_QUOTE_KEY: [bool; 256] = {
    let mut t = NEEDS_QUOTE_VALUE;
    t[b' ' as usize] = true;
    t[b',' as usize] = true;
    t
};

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

    // Fast path: check first element
    match &arr[0] {
        OwnedValue::Object(_) => {
            if let Some(keys) = try_tabular(arr) {
                ArrayKind::Tabular(keys)
            } else {
                ArrayKind::List
            }
        }
        OwnedValue::Array(_) => ArrayKind::List,
        _ => {
            // All primitives — check uniform type
            let first_kind = prim_kind_fast(&arr[0]);
            for v in &arr[1..] {
                if !is_primitive(v) || prim_kind_fast(v) != first_kind {
                    return ArrayKind::List;
                }
            }
            ArrayKind::Primitives
        }
    }
}

#[inline]
fn is_primitive(v: &OwnedValue) -> bool {
    matches!(v, OwnedValue::Static(_) | OwnedValue::String(_))
}

// Faster discriminant check — just 2 bits
#[inline]
fn prim_kind_fast(v: &OwnedValue) -> u8 {
    match v {
        OwnedValue::String(_) => 0,
        OwnedValue::Static(s) => match s {
            simd_json::StaticNode::Null => 1,
            simd_json::StaticNode::Bool(_) => 2,
            _ => 3, // number
        },
        _ => 4,
    }
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

    // All values in first must be primitive
    for v in first_obj.values() {
        if !is_primitive(v) {
            return None;
        }
    }

    let key_count = keys.len();
    // Check remaining objects
    for item in &arr[1..] {
        let obj = match item {
            OwnedValue::Object(o) => o,
            _ => return None,
        };
        if obj.len() != key_count {
            return None;
        }
        // Check keys match and values are primitive
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

// Pre-computed indent strings for depths 0..16
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

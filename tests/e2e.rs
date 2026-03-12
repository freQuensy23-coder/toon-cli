use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn toon() -> Command {
    Command::cargo_bin("toon-cli").unwrap()
}

// ─── Primitives ─────────────────────────────────────────────

#[test]
fn root_string() {
    toon()
        .write_stdin(r#""hello world""#)
        .assert()
        .success()
        .stdout("hello world\n");
}

#[test]
fn root_number_int() {
    toon()
        .write_stdin("42")
        .assert()
        .success()
        .stdout("42\n");
}

#[test]
fn root_number_float() {
    toon()
        .write_stdin("3.14")
        .assert()
        .success()
        .stdout("3.14\n");
}

#[test]
fn root_bool_true() {
    toon()
        .write_stdin("true")
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn root_bool_false() {
    toon()
        .write_stdin("false")
        .assert()
        .success()
        .stdout("false\n");
}

#[test]
fn root_null() {
    toon()
        .write_stdin("null")
        .assert()
        .success()
        .stdout("null\n");
}

#[test]
fn root_negative_zero() {
    toon()
        .write_stdin("-0")
        .assert()
        .success()
        .stdout("0\n");
}

#[test]
fn root_string_needs_quoting_number_like() {
    toon()
        .write_stdin(r#""123""#)
        .assert()
        .success()
        .stdout("\"123\"\n");
}

#[test]
fn root_string_needs_quoting_bool_like() {
    toon()
        .write_stdin(r#""true""#)
        .assert()
        .success()
        .stdout("\"true\"\n");
}

#[test]
fn root_string_needs_quoting_null_like() {
    toon()
        .write_stdin(r#""null""#)
        .assert()
        .success()
        .stdout("\"null\"\n");
}

#[test]
fn root_string_empty() {
    toon()
        .write_stdin(r#""""#)
        .assert()
        .success()
        .stdout("\"\"\n");
}

#[test]
fn root_string_with_special_chars() {
    toon()
        .write_stdin(r#""hello: world""#)
        .assert()
        .success()
        .stdout("\"hello: world\"\n");
}

#[test]
fn root_string_with_newline() {
    toon()
        .write_stdin(r#""line1\nline2""#)
        .assert()
        .success()
        .stdout("\"line1\\nline2\"\n");
}

#[test]
fn root_string_with_leading_space() {
    toon()
        .write_stdin(r#"" hello""#)
        .assert()
        .success()
        .stdout("\" hello\"\n");
}

#[test]
fn root_string_with_trailing_space() {
    toon()
        .write_stdin(r#""hello ""#)
        .assert()
        .success()
        .stdout("\"hello \"\n");
}

// ─── Simple Objects ─────────────────────────────────────────

#[test]
fn simple_flat_object() {
    toon()
        .write_stdin(r#"{"name":"Alice","age":30,"active":true}"#)
        .assert()
        .success()
        .stdout("name: Alice\nage: 30\nactive: true\n");
}

#[test]
fn empty_object() {
    // Root empty object → empty output (no keys)
    toon()
        .write_stdin(r#"{}"#)
        .assert()
        .success()
        .stdout("\n");
}

#[test]
fn nested_object() {
    toon()
        .write_stdin(r#"{"user":{"name":"Alice","address":{"city":"Berlin"}}}"#)
        .assert()
        .success()
        .stdout(
            "user:\n  name: Alice\n  address:\n    city: Berlin\n",
        );
}

#[test]
fn object_with_null_value() {
    toon()
        .write_stdin(r#"{"name":"Alice","bio":null}"#)
        .assert()
        .success()
        .stdout("name: Alice\nbio: null\n");
}

#[test]
fn object_with_empty_string_value() {
    toon()
        .write_stdin(r#"{"name":"","age":30}"#)
        .assert()
        .success()
        .stdout("name: \"\"\nage: 30\n");
}

#[test]
fn deeply_nested_object() {
    toon()
        .write_stdin(r#"{"a":{"b":{"c":{"d":"deep"}}}}"#)
        .assert()
        .success()
        .stdout("a:\n  b:\n    c:\n      d: deep\n");
}

// ─── Primitive Arrays ───────────────────────────────────────

#[test]
fn root_primitive_array_strings() {
    toon()
        .write_stdin(r#"["a","b","c"]"#)
        .assert()
        .success()
        .stdout("[3]: a,b,c\n");
}

#[test]
fn root_primitive_array_numbers() {
    toon()
        .write_stdin(r#"[1,2,3]"#)
        .assert()
        .success()
        .stdout("[3]: 1,2,3\n");
}

#[test]
fn root_empty_array() {
    toon()
        .write_stdin(r#"[]"#)
        .assert()
        .success()
        .stdout("[0]:\n");
}

#[test]
fn object_with_primitive_array() {
    toon()
        .write_stdin(r#"{"tags":["admin","ops","dev"]}"#)
        .assert()
        .success()
        .stdout("tags[3]: admin,ops,dev\n");
}

#[test]
fn object_with_number_array() {
    toon()
        .write_stdin(r#"{"scores":[10,20,30]}"#)
        .assert()
        .success()
        .stdout("scores[3]: 10,20,30\n");
}

#[test]
fn object_with_empty_array() {
    toon()
        .write_stdin(r#"{"items":[]}"#)
        .assert()
        .success()
        .stdout("items[0]:\n");
}

// ─── Tabular Arrays ────────────────────────────────────────

#[test]
fn tabular_array_simple() {
    toon()
        .write_stdin(
            r#"{"users":[{"id":1,"name":"Alice","role":"admin"},{"id":2,"name":"Bob","role":"user"}]}"#,
        )
        .assert()
        .success()
        .stdout("users[2]{id,name,role}:\n  1,Alice,admin\n  2,Bob,user\n");
}

#[test]
fn root_tabular_array() {
    toon()
        .write_stdin(
            r#"[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]"#,
        )
        .assert()
        .success()
        .stdout("[2]{id,name}:\n  1,Alice\n  2,Bob\n");
}

#[test]
fn tabular_with_null_values() {
    toon()
        .write_stdin(
            r#"[{"a":1,"b":null},{"a":2,"b":"hi"}]"#,
        )
        .assert()
        .success()
        .stdout("[2]{a,b}:\n  1,null\n  2,hi\n");
}

#[test]
fn tabular_with_bool_values() {
    toon()
        .write_stdin(
            r#"[{"name":"Alice","active":true},{"name":"Bob","active":false}]"#,
        )
        .assert()
        .success()
        .stdout("[2]{name,active}:\n  Alice,true\n  Bob,false\n");
}

#[test]
fn tabular_with_string_needing_quotes() {
    toon()
        .write_stdin(
            r#"[{"k":"hello, world"},{"k":"simple"}]"#,
        )
        .assert()
        .success()
        .stdout("[2]{k}:\n  \"hello, world\"\n  simple\n");
}

// ─── List Arrays (non-uniform / nested) ─────────────────────

#[test]
fn list_array_mixed_types() {
    toon()
        .write_stdin(r#"[1,"hello",true]"#)
        .assert()
        .success()
        .stdout("[3]:\n  - 1\n  - hello\n  - true\n");
}

#[test]
fn list_array_of_objects_different_keys() {
    toon()
        .write_stdin(
            r#"[{"a":1},{"b":2}]"#,
        )
        .assert()
        .success()
        .stdout("[2]:\n  -\n    a: 1\n  -\n    b: 2\n");
}

#[test]
fn list_array_objects_with_nested_values() {
    toon()
        .write_stdin(
            r#"[{"name":"Alice","addr":{"city":"NY"}},{"name":"Bob","addr":{"city":"LA"}}]"#,
        )
        .assert()
        .success()
        .stdout("[2]:\n  -\n    name: Alice\n    addr:\n      city: NY\n  -\n    name: Bob\n    addr:\n      city: LA\n");
}

#[test]
fn array_of_arrays() {
    toon()
        .write_stdin(r#"[[1,2],[3,4]]"#)
        .assert()
        .success()
        .stdout("[2]:\n  - [2]: 1,2\n  - [2]: 3,4\n");
}

#[test]
fn single_element_tabular() {
    toon()
        .write_stdin(r#"[{"id":1,"name":"Alice"}]"#)
        .assert()
        .success()
        .stdout("[1]{id,name}:\n  1,Alice\n");
}

// ─── Complex / Real-world ───────────────────────────────────

#[test]
fn api_response_shape() {
    let json = r#"{
        "status": "ok",
        "count": 2,
        "data": [
            {"id": 1, "email": "a@b.com", "score": 95.5},
            {"id": 2, "email": "c@d.com", "score": 87.3}
        ]
    }"#;
    toon()
        .write_stdin(json)
        .assert()
        .success()
        .stdout(
            "status: ok\ncount: 2\ndata[2]{id,email,score}:\n  1,a@b.com,95.5\n  2,c@d.com,87.3\n",
        );
}

#[test]
fn object_with_mixed_array_and_scalar() {
    let json = r#"{"title":"Report","tags":["a","b"],"version":3}"#;
    toon()
        .write_stdin(json)
        .assert()
        .success()
        .stdout("title: Report\ntags[2]: a,b\nversion: 3\n");
}

#[test]
fn empty_nested_object() {
    toon()
        .write_stdin(r#"{"config":{}}"#)
        .assert()
        .success()
        .stdout("config:\n");
}

// ─── CLI Features ───────────────────────────────────────────

#[test]
fn reads_from_file_arg() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, r#"{{"name":"Alice","age":30}}"#).unwrap();
    toon()
        .arg(f.path())
        .assert()
        .success()
        .stdout("name: Alice\nage: 30\n");
}

#[test]
fn reads_from_stdin_when_no_file() {
    toon()
        .write_stdin(r#"{"x":1}"#)
        .assert()
        .success()
        .stdout("x: 1\n");
}

#[test]
fn writes_to_output_file() {
    let input = NamedTempFile::new().unwrap();
    std::fs::write(input.path(), r#"{"a":1}"#).unwrap();
    let output = NamedTempFile::new().unwrap();
    toon()
        .arg(input.path())
        .arg("-o")
        .arg(output.path())
        .assert()
        .success();
    let content = std::fs::read_to_string(output.path()).unwrap();
    assert_eq!(content, "a: 1\n");
}

#[test]
fn invalid_json_gives_error() {
    toon()
        .write_stdin("{invalid}")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn help_flag() {
    toon()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("JSON to TOON"));
}

#[test]
fn version_flag() {
    toon()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

// ─── Pipe-friendly behavior ────────────────────────────────

#[test]
fn large_json_pipe() {
    // Simulate a large array piped in
    let mut items = Vec::new();
    for i in 0..100 {
        items.push(format!(r#"{{"id":{},"val":"item{}"}}"#, i, i));
    }
    let json = format!("[{}]", items.join(","));
    let result = toon()
        .write_stdin(json)
        .assert()
        .success();
    let stdout = String::from_utf8(result.get_output().stdout.clone()).unwrap();
    assert!(stdout.starts_with("[100]{id,val}:\n"));
    assert!(stdout.contains("0,item0"));
    assert!(stdout.contains("99,item99"));
}

// ─── Edge cases ─────────────────────────────────────────────

#[test]
fn string_with_comma_in_primitive_array() {
    // Strings with commas need quoting in arrays
    toon()
        .write_stdin(r#"["a,b","c"]"#)
        .assert()
        .success()
        .stdout("[2]: \"a,b\",c\n");
}

#[test]
fn string_with_quotes() {
    toon()
        .write_stdin(r#"{"msg":"He said \"hi\""}"#)
        .assert()
        .success()
        .stdout("msg: \"He said \\\"hi\\\"\"\n");
}

#[test]
fn number_no_trailing_zeros() {
    toon()
        .write_stdin(r#"{"val":1.0}"#)
        .assert()
        .success()
        .stdout("val: 1.0\n");
}

#[test]
fn tabular_not_eligible_mixed_keys() {
    // Objects with different keys → list format
    toon()
        .write_stdin(r#"[{"a":1,"b":2},{"a":3,"c":4}]"#)
        .assert()
        .success()
        .stdout("[2]:\n  -\n    a: 1\n    b: 2\n  -\n    a: 3\n    c: 4\n");
}

#[test]
fn tabular_not_eligible_nested_value() {
    // Objects with nested object values → list format
    toon()
        .write_stdin(r#"[{"a":1,"b":{"x":1}},{"a":2,"b":{"x":2}}]"#)
        .assert()
        .success()
        .stdout("[2]:\n  -\n    a: 1\n    b:\n      x: 1\n  -\n    a: 2\n    b:\n      x: 2\n");
}

#[test]
fn unicode_string() {
    toon()
        .write_stdin(r#"{"greeting":"Привет мир"}"#)
        .assert()
        .success()
        .stdout("greeting: Привет мир\n");
}

#[test]
fn very_long_number() {
    toon()
        .write_stdin(r#"{"big":99999999999999}"#)
        .assert()
        .success()
        .stdout("big: 99999999999999\n");
}

#[test]
fn nested_array_in_object() {
    toon()
        .write_stdin(r#"{"matrix":[[1,2],[3,4]]}"#)
        .assert()
        .success()
        .stdout("matrix[2]:\n  - [2]: 1,2\n  - [2]: 3,4\n");
}

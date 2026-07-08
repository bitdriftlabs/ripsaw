use chrono::{DateTime, Datelike, TimeZone, Utc};
use criterion::{Criterion, criterion_group, criterion_main};
use regex::Regex;

use std::env;
use std::path::PathBuf;
use vrl::compiler::prelude::*;
use vrl::{bench_function, btreemap, func_args, value};

use crate::value::Value;

criterion_group!(
    name = benches;
    // encapsulates CI noise we saw in
    // https://github.com/vectordotdev/vector/pull/6408
    config = Criterion::default().noise_threshold(0.05);
    targets = array,
              // TODO: Cannot pass a Path to bench_function:
              //del,
              //exists
              //unnest
              assert,
              r#bool,
              camelcase,
              ceil,
              chunks,
              compact,
              contains,
              downcase,
              ends_with,
              find,
              flatten,
              floor,
              float,
              format_int,
              format_number,
              format_timestamp,
              get,
              includes,
              int,
              is_array,
              is_boolean,
              is_empty,
              is_float,
              is_integer,
              is_ipv4,
              is_ipv6,
              is_json,
              is_null,
              is_nullish,
              is_object,
              is_regex,
              is_string,
              is_timestamp,
              join,
              keys,
              kebabcase,
              length,
              r#match,
              match_any,
              match_array,
              merge,
              r#mod,
              object,
              object_from_array,
              parse_int,
              pascalcase,
              push,
              remove,
              replace,
              round,
              set,
              screamingsnakecase,
              snakecase,
              sieve,
              slice,
              split,
              starts_with,
              string,
              strip_whitespace,
              strlen,
              tally,
              tally_value,
              timestamp,
              to_bool,
              to_float,
              to_int,
              to_regex,
              to_string,
              truncate,
              unflatten,
              unique,
              upcase,
              values,
              zip,
);
criterion_main!(benches);

bench_function! {
    append => ripsaw::stdlib::Append;

    arrays {
        args: func_args![value: value!([1, 2, 3]), items: value!([4, 5, 6])],
        want: Ok(value!([1, 2, 3, 4, 5, 6])),
    }
}

bench_function! {
    array => ripsaw::stdlib::Array;

    array {
        args: func_args![value: value!([1,2,3])],
        want: Ok(value!([1,2,3])),
    }
}

bench_function! {
    assert => ripsaw::stdlib::Assert;

    literal {
        args: func_args![condition: value!(true), message: "must be true"],
        want: Ok(value!(true)),
    }
}

bench_function! {
    r#bool => ripsaw::stdlib::Boolean;

    r#bool {
        args: func_args![value: value!(true)],
        want: Ok(value!(true)),
    }
}

bench_function! {
    ceil => ripsaw::stdlib::Ceil;

    literal {
        args: func_args![value: 1234.56725, precision: 4],
        want: Ok(1234.5673),
    }
}

bench_function! {
    chunks => ripsaw::stdlib::Chunks;

    literal {
        args: func_args![value: "abcdefgh", chunk_size: 4],
        want: Ok(value!(["abcd", "efgh"])),
    }
}

bench_function! {
    compact => ripsaw::stdlib::Compact;

    array {
        args: func_args![
            value: value!([null, 1, "" ]),
        ],
        want: Ok(value!([ 1 ])),
    }

    map {
        args: func_args![
            value: value!({ "key1": null, "key2":  1, "key3": "" }),
        ],
        want: Ok(value!({ "key2": 1 })),
    }
}

bench_function! {
    contains => ripsaw::stdlib::Contains;

    case_sensitive {
        args: func_args![value: "abcdefg", substring: "cde", case_sensitive: true],
        want: Ok(value!(true)),
    }

    case_insensitive {
        args: func_args![value: "abcdefg", substring: "CDE", case_sensitive: false],
        want: Ok(value!(true)),
    }
}

bench_function! {
    downcase => ripsaw::stdlib::Downcase;

    literal {
        args: func_args![value: "FOO"],
        want: Ok("foo")
    }
}

bench_function! {
    ends_with => ripsaw::stdlib::EndsWith;

    case_sensitive {
        args: func_args![value: "abcdefg", substring: "efg", case_sensitive: true],
        want: Ok(value!(true)),
    }

    case_insensitive {
        args: func_args![value: "abcdefg", substring: "EFG", case_sensitive: false],
        want: Ok(value!(true)),
    }
}

bench_function! {
    find => ripsaw::stdlib::Find;

    str_matching {
        args: func_args![value: "foobarfoo", pattern: "bar"],
        want: Ok(value!(3)),
    }

    str_too_long {
        args: func_args![value: "foo", pattern: "foobar"],
        want: Ok(value!(-1)),
    }

    regex_matching_start {
        args: func_args![value: "foobar", pattern: Value::Regex(Regex::new("fo+z?").unwrap().into())],
        want: Ok(value!(0)),
    }
}

bench_function! {
    flatten => ripsaw::stdlib::Flatten;

    nested_map {
        args: func_args![value: value!({parent: {child1: 1, child2: 2}, key: "val"})],
        want: Ok(value!({"parent.child1": 1, "parent.child2": 2, key: "val"})),
    }

    nested_array {
        args: func_args![value: value!([42, [43, 44]])],
        want: Ok(value!([42, 43, 44])),
    }

    map_and_array {
        args: func_args![value: value!({
            "parent": {
                "child1": [1, [2, 3]],
                "child2": {"grandchild1": 1, "grandchild2": [1, [2, 3], 4]},
            },
            "key": "val",
        })],
        want: Ok(value!({
            "parent.child1": [1, [2, 3]],
            "parent.child2.grandchild1": 1,
            "parent.child2.grandchild2": [1, [2, 3], 4],
            "key": "val",
        })),
    }
}

bench_function! {
    float => ripsaw::stdlib::Float;

    float {
        args: func_args![value: value!(1.2)],
        want: Ok(value!(1.2)),
    }
}

bench_function! {
    floor  => ripsaw::stdlib::Floor;

    literal {
        args: func_args![value: 1234.56725, precision: 4],
        want: Ok(1234.5672),
    }
}

bench_function! {
    format_int => ripsaw::stdlib::FormatInt;

    decimal {
        args: func_args![value: 42],
        want: Ok("42"),
    }

    hexadecimal {
        args: func_args![value: 42, base: 16],
        want: Ok(value!("2a")),
    }
}

bench_function! {
    format_number => ripsaw::stdlib::FormatNumber;

    literal {
        args: func_args![
            value: 11222333444.56789,
            scale: 3,
            decimal_separator: ",",
            grouping_separator: "."
        ],
        want: Ok("11.222.333.444,567"),
    }
}

bench_function! {
    format_timestamp => ripsaw::stdlib::FormatTimestamp;

    iso_6801 {
        args: func_args![value: Utc.timestamp_opt(10, 0).single().expect("invalid timestamp"), format: "%+"],
        want: Ok("1970-01-01T00:00:10+00:00"),
    }
}

bench_function! {
    includes => ripsaw::stdlib::Includes;

    mixed_included_string {
        args: func_args![value: value!(["foo", 1, true, [1,2,3]]), item: value!("foo")],
        want: Ok(value!(true)),
    }
}

bench_function! {
    set => ripsaw::stdlib::Set;

    single {
        args: func_args![value: value!({ "foo": "bar" }), path: vec!["baz"], data: true],
        want: Ok(value!({ "foo": "bar", "baz": true })),
    }

    nested {
        args: func_args![value: value!({ "foo": { "bar": "baz" } }), path: vec!["foo", "bar", "qux"], data: 42],
        want: Ok(value!({ "foo": { "bar": { "qux": 42 } } })),
    }

    indexing {
        args: func_args![value: value!([0, 42, 91]), path: vec![3], data: 1],
        want: Ok(value!([0, 42, 91, 1])),
    }
}

bench_function! {
    int => ripsaw::stdlib::Integer;

    int {
        args: func_args![value: value!(1)],
        want: Ok(value!(1)),
    }
}

bench_function! {
    is_array => ripsaw::stdlib::IsArray;

    string {
        args: func_args![value: "foobar"],
        want: Ok(false),
    }

    array {
        args: func_args![value: value!([1, 2, 3])],
        want: Ok(true),
    }
}

bench_function! {
    is_boolean => ripsaw::stdlib::IsBoolean;

    string {
        args: func_args![value: "foobar"],
        want: Ok(false),
    }

    boolean {
        args: func_args![value: true],
        want: Ok(true),
    }
}

bench_function! {
    is_empty => ripsaw::stdlib::IsEmpty;

    empty_array {
        args: func_args![value: value!([])],
        want: Ok(true),
    }

    non_empty_array {
        args: func_args![value: value!([1, 2, 3])],
        want: Ok(false),
    }

    empty_object {
        args: func_args![value: value!({})],
        want: Ok(true),
    }

    non_empty_object {
        args: func_args![value: value!({"foo": "bar"})],
        want: Ok(false),
    }

    string {
        args: func_args![value: "foo"],
        want: Ok(false),
    }
}

bench_function! {
    is_float => ripsaw::stdlib::IsFloat;

    array {
        args: func_args![value: value!([1, 2, 3])],
        want: Ok(false),
    }

    float {
        args: func_args![value: 0.577],
        want: Ok(true),
    }
}

bench_function! {
    is_integer => ripsaw::stdlib::IsInteger;

    integer {
        args: func_args![value: 1701],
        want: Ok(true),
    }

    object {
        args: func_args![value: value!({"foo": "bar"})],
        want: Ok(false),
    }
}

bench_function! {
    is_ipv4 => ripsaw::stdlib::IsIpv4;

    not_string {
        args: func_args![value: 42],
        want: Ok(false),
    }

    ipv4 {
        args: func_args![value: "192.168.0.1"],
        want: Ok(true),
    }

    invalid_ipv4 {
        args: func_args![value: "192.168.0.299"],
        want: Ok(false),
    }

    ipv6 {
        args: func_args![value: "2404:6800:4003:c02::64"],
        want: Ok(false),
    }
}

bench_function! {
    is_ipv6 => ripsaw::stdlib::IsIpv6;

    not_string {
        args: func_args![value: 42],
        want: Ok(false),
    }

    ipv4 {
        args: func_args![value: "192.168.0.1"],
        want: Ok(false),
    }

    ipv6 {
        args: func_args![value: "2404:6800:4003:c02::64"],
        want: Ok(true),
    }

    invalid_ipv6 {
        args: func_args![value: "2404:6800:goat:c02::64"],
        want: Ok(false),
    }
}

bench_function! {
    is_json => ripsaw::stdlib::IsJson;

    map {
        args: func_args![value: r#"{"key": "value"}"#],
        want: Ok(true),
    }

    invalid_map {
        args: func_args![value: r#"{"key": "value""#],
        want: Ok(false),
    }

    exact_variant {
        args: func_args![value: r#"{"key": "value""#, variant: "object"],
        want: Ok(true),
    }
}

bench_function! {
    is_null => ripsaw::stdlib::IsNull;

    string {
        args: func_args![value: "foobar"],
        want: Ok(false),
    }

    null {
        args: func_args![value: value!(null)],
        want: Ok(true),
    }
}

bench_function! {
    is_nullish => ripsaw::stdlib::IsNullish;

    whitespace {
        args: func_args![value: "         "],
        want: Ok(true),
    }

    hyphen {
        args: func_args![value: "-"],
        want: Ok(true),
    }

    null {
        args: func_args![value: value!(null)],
        want: Ok(true),
    }

    not_empty {
        args: func_args![value: "foo"],
        want: Ok(false),
    }
}

bench_function! {
    is_object => ripsaw::stdlib::IsObject;

    integer {
        args: func_args![value: 1701],
        want: Ok(false),
    }

    object {
        args: func_args![value: value!({"foo": "bar"})],
        want: Ok(true),
    }
}

bench_function! {
    is_regex => ripsaw::stdlib::IsRegex;

    regex {
        args: func_args![value: value!(Regex::new(r"\d+").unwrap())],
        want: Ok(true),
    }

    object {
        args: func_args![value: value!({"foo": "bar"})],
        want: Ok(false),
    }
}

bench_function! {
    is_string => ripsaw::stdlib::IsString;

    string {
        args: func_args![value: "foobar"],
        want: Ok(true),
    }

    array {
        args: func_args![value: value!([1, 2, 3])],
        want: Ok(false),
    }
}

bench_function! {
    is_timestamp => ripsaw::stdlib::IsTimestamp;

    string {
        args: func_args![value: Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap()],
        want: Ok(true),
    }

    array {
        args: func_args![value: value!([1, 2, 3])],
        want: Ok(false),
    }
}

bench_function! {
    join => ripsaw::stdlib::Join;

    literal {
        args: func_args![value: value!(["hello", "world"]), separator: " "],
        want: Ok("hello world"),
    }
}

bench_function! {
    keys => ripsaw::stdlib::Keys;

    literal {
        args: func_args![value: value!({"key1": "val1", "key2": "val2"})],
        want: Ok(value!(["key1", "key2"])),
    }
}

bench_function! {
    length => ripsaw::stdlib::Length;

    map {
        args: func_args![value: value!({foo: "bar", baz: true, baq: [1, 2, 3]})],
        want: Ok(3),
    }

    array {
        args: func_args![value: value!([1, 2, 3, 4, true, "hello"])],
        want: Ok(value!(6)),
    }

    string {
        args: func_args![value: "hello world"],
        want: Ok(value!(11))
    }
}

// TODO: Ensure tracing is enabled
bench_function! {
    get => ripsaw::stdlib::Get;

    single {
        args: func_args![value: value!({ "foo": "bar" }), path: vec!["foo"]],
        want: Ok("bar"),
    }

    nested {
        args: func_args![value: value!({ "foo": { "bar": "baz" } }), path: vec!["foo", "bar"]],
        want: Ok("baz"),
    }

    indexing {
        args: func_args![value: value!([0, 42, 91]), path: vec![-2]],
        want: Ok(42),
    }
}

bench_function! {
    r#match => ripsaw::stdlib::Match;

    simple {
        args: func_args![value: "foo 2 bar", pattern: Regex::new("foo \\d bar").unwrap()],
        want: Ok(true),
    }
}

bench_function! {
    match_any => ripsaw::stdlib::MatchAny;

    simple {
        args: func_args![value: "foo 2 bar", patterns: vec![Regex::new(r"foo \d bar").unwrap()]],
        want: Ok(true),
    }
}

bench_function! {
    match_array => ripsaw::stdlib::MatchArray;

    single_match {
        args: func_args![
            value: value!(["foo 1 bar"]),
            pattern: Regex::new(r"foo \d bar").unwrap(),
        ],
        want: Ok(true),
    }

    no_match {
        args: func_args![
            value: value!(["foo x bar"]),
            pattern: Regex::new(r"foo \d bar").unwrap(),
        ],
        want: Ok(false),
    }

    some_match {
        args: func_args![
            value: value!(["foo 2 bar", "foo 3 bar", "foo 4 bar", "foo 5 bar"]),
            pattern: Regex::new(r"foo \d bar").unwrap(),
        ],
        want: Ok(true),
    }

    all_match {
        args: func_args![
            value: value!(["foo 2 bar", "foo 3 bar", "foo 4 bar", "foo 5 bar"]),
            pattern: Regex::new(r"foo \d bar").unwrap(),
            all: value!(true)
        ],
        want: Ok(true),
    }

    not_all_match {
        args: func_args![
            value: value!(["foo 2 bar", "foo 3 bar", "foo 4 bar", "foo x bar"]),
            pattern: Regex::new(r"foo \d bar").unwrap(),
            all: value!(true)
        ],
        want: Ok(false),
    }
}

bench_function! {
    r#mod => ripsaw::stdlib::Mod;

    simple {
        args: func_args![
            value: value!(5),
            modulus: value!(2),
        ],
        want: Ok(value!(1))
    }
}

bench_function! {
    merge => ripsaw::stdlib::Merge;

    simple {
        args: func_args![
            to: value!({ "key1": "val1" }),
            from: value!({ "key2": "val2" }),
        ],
        want: Ok(value!({
            "key1": "val1",
            "key2": "val2",
        }))
    }

    shallow {
        args: func_args![
            to: value!({
                "key1": "val1",
                "child": { "grandchild1": "val1" },
            }),
            from: value!({
                "key2": "val2",
                "child": { "grandchild2": "val2" },
            })
        ],
        want: Ok(value!({
            "key1": "val1",
            "key2": "val2",
            "child": { "grandchild2": "val2" },
        }))
    }

    deep {
        args: func_args![
            to: value!({
                "key1": "val1",
                "child": { "grandchild1": "val1" },
            }),
            from: value!({
                "key2": "val2",
                "child": { "grandchild2": "val2" },
            }),
            deep: true
        ],
        want: Ok(value!({
            "key1": "val1",
            "key2": "val2",
            "child": {
                "grandchild1": "val1",
                "grandchild2": "val2",
            },
        }))
    }
}

bench_function! {
    object => ripsaw::stdlib::Object;

    object {
        args: func_args![value: value!({"foo": "bar"})],
        want: Ok(value!({"foo": "bar"})),
    }
}

bench_function! {
    object_from_array => ripsaw::stdlib::ObjectFromArray;

    default {
        args: func_args![values: value!([["zero",null], ["one",true], ["two","foo"], ["three",3]])],
        want: Ok(value!({"zero":null, "one":true, "two":"foo", "three":3})),
    }

    values_and_keys {
        args: func_args![
            keys: value!(["zero", "one", "two", "three"]),
            values: value!([null, true, "foo", 3]),
        ],
        want: Ok(value!({"zero":null, "one":true, "two":"foo", "three":3})),
    }
}

bench_function! {
    parse_int => ripsaw::stdlib::ParseInt;

    decimal {
        args: func_args![value: "-42"],
        want: Ok(-42),
    }

    hexidecimal {
        args: func_args![value: "0x2a"],
        want: Ok(42),
    }

    explicit_hexidecimal {
        args: func_args![value: "2a", base: 16],
        want: Ok(42),
    }
}

bench_function! {
    push => ripsaw::stdlib::Push;

    literal {
        args: func_args![value: value!([11, false, 42.5]), item: "foo"],
        want: Ok(value!([11, false, 42.5, "foo"])),
    }
}

bench_function! {
    remove => ripsaw::stdlib::Remove;

    single {
        args: func_args![value: value!({ "foo": "bar", "baz": true }), path: vec!["foo"]],
        want: Ok(value!({ "baz": true })),
    }

    nested {
        args: func_args![value: value!({ "foo": { "bar": "baz" } }), path: vec!["foo", "bar"]],
        want: Ok(value!({ "foo": {} })),
    }

    indexing {
        args: func_args![value: value!([0, 42, 91]), path: vec![-2]],
        want: Ok(vec![0, 91]),
    }
}

bench_function! {
    replace => ripsaw::stdlib::Replace;

    string {
        args: func_args![
            value: "I like apples and bananas",
            pattern: "a",
            with: "o",
        ],
        want: Ok("I like opples ond bononos")
    }

    regex {
        args: func_args![
            value: "I like apples and bananas",
            pattern: Regex::new("[a]").unwrap(),
            with: "o",
        ],
        want: Ok("I like opples ond bononos")
    }
}

bench_function! {
    round => ripsaw::stdlib::Round;

    integer {
        args: func_args![value: 1234.56789, precision: 4],
        want: Ok(1234.5679)
    }

    float {
        args: func_args![value: 1234, precision: 4],
        want: Ok(1234)
    }
}

bench_function! {
    sieve => ripsaw::stdlib::Sieve;

    regex {
        args: func_args![value: value!("test123%456.فوائد.net."), permitted_characters: regex::Regex::new("[a-z.0-9]").unwrap(), replace_single: "X", replace_repeated: "<REMOVED>"],
        want: Ok(value!("test123X456.<REMOVED>.net.")),
    }

    string {
        args: func_args![value: value!("37ccx6a5uf52a7dv2hfxgpmltji09x6xkg0zv6yxsoi4kqs9atmjh7k50dcjb7z.فوائد.net."), permitted_characters: "acx.", replace_single: "0", replace_repeated: "<REMOVED>"],
        want: Ok(value!("<REMOVED>ccx0a<REMOVED>a<REMOVED>x<REMOVED>x0x<Removed>x<Removed>a<Removed>c<REMOVED>.<REMOVED>.<REMOVED>.")),
    }
}

bench_function! {
    slice => ripsaw::stdlib::Slice;

    literal {
        args: func_args![
            value: "Supercalifragilisticexpialidocious",
            start: 5,
            end: 9,
        ],
        want: Ok("cali")
    }
}

bench_function! {
    split => ripsaw::stdlib::Split;

    string {
        args: func_args![value: "foo,bar,baz", pattern: ","],
        want: Ok(value!(["foo", "bar", "baz"]))
    }

    regex {
        args: func_args![value: "foo,bar,baz", pattern: Regex::new("[,]").unwrap()],
        want: Ok(value!(["foo", "bar", "baz"]))
    }
}

bench_function! {
    starts_with  => ripsaw::stdlib::StartsWith;

    case_sensitive {
        args: func_args![value: "abcdefg", substring: "abc", case_sensitive: true],
        want: Ok(value!(true)),
    }

    case_insensitive {
        args: func_args![value: "abcdefg", substring: "ABC", case_sensitive: false],
        want: Ok(value!(true)),
    }
}

bench_function! {
    string => ripsaw::stdlib::String;

    string {
        args: func_args![value: "2"],
        want: Ok("2")
    }
}

bench_function! {
    strip_ansi_escape_codes => ripsaw::stdlib::StripAnsiEscapeCodes;

    literal {
        args: func_args![value: "\x1b[46mfoo\x1b[0m bar"],
        want: Ok("foo bar")
    }
}

bench_function! {
    strip_whitespace => ripsaw::stdlib::StripWhitespace;

    literal {
        args: func_args![
            value:" \u{3000}\u{205F}\u{202F}\u{A0}\u{9} ❤❤ hi there ❤❤  \u{9}\u{A0}\u{202F}\u{205F}\u{3000}"
        ],
        want: Ok("❤❤ hi there ❤❤")
    }
}

bench_function! {
    strlen => ripsaw::stdlib::Strlen;

    literal {
        args: func_args![value: "ñandú"],
        want: Ok(5)
    }
}

bench_function! {
    tag_types_externally => ripsaw::stdlib::TagTypesExternally;

    tag_bytes {
        args: func_args![value: "foo"],
        want: Ok(btreemap! {
            "string" => "foo",
        }),
    }

    tag_integer {
        args: func_args![value: 123],
        want: Ok(btreemap! {
            "integer" => 123
        }),
    }

    tag_float {
        args: func_args![value: 123.45],
        want: Ok(btreemap! {
            "float" => 123.45
        }),
    }

    tag_boolean {
        args: func_args![value: true],
        want: Ok(btreemap! {
            "boolean" => true
        }),
    }

    tag_map {
        args: func_args![value: btreemap! {"foo" => "bar"}],
        want: Ok(btreemap! {
            "foo" => btreemap! {
                "string" => "bar"
            }
        }),
    }

    tag_array {
        args: func_args![value: vec!["foo"]],
        want: Ok(vec![
            btreemap! {
                "string" => "foo"
            },
        ]),
    }

    tag_timestamp {
        args: func_args![value: Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap()],
        want: Ok(btreemap! {
            "timestamp" => Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap()
        }),
    }

    tag_regex {
        args: func_args![value: Regex::new(".*").unwrap()],
        want: Ok(btreemap! {
            "regex" => Regex::new(".*").unwrap()
        }),
    }

    tag_null {
        args: func_args![value: Value::Null],
        want: Ok(Value::Null),
    }
}

bench_function! {
    tally => ripsaw::stdlib::Tally;

    default {
        args: func_args![
            value: value!(["bar", "foo", "baz", "foo"]),
        ],
        want: Ok(value!({"bar": 1, "foo": 2, "baz": 1})),
    }
}

bench_function! {
    tally_value => ripsaw::stdlib::TallyValue;

    default {
        args: func_args![
            array: value!(["bar", "foo", "baz", "foo"]),
            value: "foo",
        ],
        want: Ok(value!(2)),
    }
}

bench_function! {
    timestamp => ripsaw::stdlib::Timestamp;

    timestamp {
        args: func_args![value: Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap()],
        want: Ok(value!(Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap())),
    }
}

bench_function! {
    to_bool => ripsaw::stdlib::ToBool;

    string {
        args: func_args![value: "true"],
        want: Ok(true)
    }

    r#bool {
        args: func_args![value: true],
        want: Ok(true)
    }

    int {
        args: func_args![value: 20],
        want: Ok(true)
    }

    null {
        args: func_args![value: value!(null)],
        want: Ok(false)
    }
}

bench_function! {
    to_float => ripsaw::stdlib::ToFloat;

    string {
        args: func_args![value: "2.0"],
        want: Ok(2.0)
    }

    r#bool {
        args: func_args![value: true],
        want: Ok(1.0)
    }

    float {
        args: func_args![value: 1.0],
        want: Ok(1.0)
    }

    null {
        args: func_args![value: value!(null)],
        want: Ok(0.0)
    }
}

bench_function! {
    to_int => ripsaw::stdlib::ToInt;

    string {
        args: func_args![value: "2"],
        want: Ok(2)
    }

    r#bool {
        args: func_args![value: true],
        want: Ok(1)
    }

    int {
        args: func_args![value: 1],
        want: Ok(1)
    }

    null {
        args: func_args![value: value!(null)],
        want: Ok(0)
    }
}

bench_function! {
    to_regex => ripsaw::stdlib::ToRegex;

    regex {
        args: func_args![value: "^foo.*bar.*baz"],
        want: Ok(Regex::new("^foo.*bar.*baz").expect("regex is valid"))
    }
}

bench_function! {
    to_string => ripsaw::stdlib::ToString;

    string {
        args: func_args![value: "2"],
        want: Ok("2")
    }

    r#bool {
        args: func_args![value: true],
        want: Ok("true")
    }

    int {
        args: func_args![value: 1],
        want: Ok("1")
    }

    null {
        args: func_args![value: value!(null)],
        want: Ok("")
    }
}

bench_function! {
    truncate => ripsaw::stdlib::Truncate;

    ellipsis {
        args: func_args![
            value: "Supercalifragilisticexpialidocious",
            limit: 5,
            suffix: "...",
        ],
        want: Ok("Super..."),
    }

    no_suffix {
        args: func_args![
            value: "Supercalifragilisticexpialidocious",
            limit: 5,
        ],
        want: Ok("Super"),
    }
}

bench_function! {
    unflatten => ripsaw::stdlib::Unflatten;

    nested_map {
        args: func_args![value: value!({"parent.child1": 1, "parent.child2": 2, key: "val"})],
        want: Ok(value!({parent: {child1: 1, child2: 2}, key: "val"})),
    }

    map_and_array {
        args: func_args![value: value!({
            "parent.child1": [1, [2, 3]],
            "parent.child2.grandchild1": 1,
            "parent.child2.grandchild2": [1, [2, 3], 4],
            "key": "val",
        })],
        want: Ok(value!({
            "parent": {
                "child1": [1, [2, 3]],
                "child2": {"grandchild1": 1, "grandchild2": [1, [2, 3], 4]},
            },
            "key": "val",
        })),
    }
}

bench_function! {
    unique => ripsaw::stdlib::Unique;

    default {
        args: func_args![
            value: value!(["bar", "foo", "baz", "foo"]),
        ],
        want: Ok(value!(["bar", "foo", "baz"])),
    }

    mixed_values {
        args: func_args![
            value: value!(["foo", [1,2,3], "123abc", 1, true, [1,2,3], "foo", true, 1]),
        ],
        want: Ok(value!(["foo", [1,2,3], "123abc", 1, true])),
    }
}

bench_function! {
    upcase => ripsaw::stdlib::Upcase;

    literal {
        args: func_args![value: "foo"],
        want: Ok("FOO")
    }
}

bench_function! {
    values => ripsaw::stdlib::Values;

    literal {
        args: func_args![value: value!({"key1": "val1", "key2": "val2"})],
        want: Ok(value!(["val1", "val2"])),
    }
}

bench_function! {
    camelcase => ripsaw::stdlib::Camelcase;

    default {
        args: func_args![value: "input-string"],
        want: Ok("inputString"),
    }
}

bench_function! {
    pascalcase => ripsaw::stdlib::Pascalcase;

    default {
        args: func_args![value: "input-string"],
        want: Ok("InputString"),
    }
}

bench_function! {
    kebabcase => ripsaw::stdlib::Kebabcase;

    default {
        args: func_args![value: "inputString"],
        want: Ok("input-string"),
    }
}

bench_function! {
    snakecase => ripsaw::stdlib::Snakecase;

    default {
        args: func_args![value: "input-string"],
        want: Ok("input_string"),
    }
}

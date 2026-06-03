#![deny(warnings, clippy::pedantic)]
pub use wasm_unsupported_function::WasmUnsupportedFunction;

use crate::compiler::Function;

mod string_utils;
mod util;
mod wasm_unsupported_function;

cfg_if::cfg_if! {
    if #[cfg(feature = "stdlib-base")] {
        // Base stdlib modules (always included with stdlib-base)
        mod abs;
        mod all;
        mod any;
        mod append;
        mod array;
        mod basename;
        mod boolean;
        mod ceil;
        mod casing;
        mod chunks;
        mod compact;
        mod contains;
        mod contains_all;
        mod del;
        mod dirname;
        mod downcase;
        mod ends_with;
        mod exists;
        mod filter;
        mod find;
        mod flatten;
        mod float;
        mod floor;
        mod for_each;
        mod format_int;
        mod format_number;
        mod format_timestamp;
        mod get;
        mod includes;
        mod integer;
        mod is_array;
        mod is_boolean;
        mod is_empty;
        mod is_float;
        mod is_integer;
        mod is_ipv4;
        mod is_ipv6;
        mod is_json;
        mod is_null;
        mod is_nullish;
        mod is_object;
        mod is_regex;
        mod is_string;
        mod is_timestamp;
        mod join;
        mod keys;
        mod length;
        mod map;
        mod map_keys;
        mod map_values;
        mod r#match;
        mod match_any;
        mod match_array;
        mod merge;
        mod mod_func;
        mod object;
        mod object_from_array;
        mod parse_float;
        mod parse_int;
        mod pop;
        mod push;
        mod remove;
        mod replace;
        mod replace_with;
        mod round;
        mod set;
        mod sieve;
        mod slice;
        mod split;
        mod split_path;
        mod starts_with;
        mod string;
        mod strip_whitespace;
        mod strlen;
        mod tag_types_externally;
        mod tally;
        mod tally_value;
        mod timestamp;
        mod to_bool;
        mod to_float;
        mod to_int;
        mod to_regex;
        mod to_string;
        mod truncate;
        mod unique;
        mod unnest;
        mod upcase;
        mod values;
        mod zip;

        // -----------------------------------------------------------------------------

        // Macro to keep pub use and all() function in sync
        macro_rules! stdlib_functions {
            (
                $(
                    $(#[$attr:meta])*
                    $path:path
                ),* $(,)?
            ) => {
                // Generate pub use statements
                $(
                    $(#[$attr])*
                    pub use $path;
                )*

                // Generate the all() function
                #[must_use]
                #[allow(clippy::too_many_lines)]
                pub fn all() -> Vec<Box<dyn Function>> {
                    vec![
                        $(
                            $(#[$attr])*
                            Box::new($path),
                        )*
                    ]
                }
            };
        }

        stdlib_functions! {
            // ===== Base stdlib functions (always included with stdlib-base) =====
            abs::Abs,
            all::All,
            any::Any,
            append::Append,
            basename::BaseName,
            boolean::Boolean,
            ceil::Ceil,
            chunks::Chunks,
            compact::Compact,
            contains::Contains,
            contains_all::ContainsAll,
            del::Del,
            dirname::DirName,
            downcase::Downcase,
            casing::camelcase::Camelcase,
            casing::pascalcase::Pascalcase,
            casing::snakecase::Snakecase,
            casing::screamingsnakecase::ScreamingSnakecase,
            casing::kebabcase::Kebabcase,
            ends_with::EndsWith,
            exists::Exists,
            filter::Filter,
            find::Find,
            flatten::Flatten,
            float::Float,
            floor::Floor,
            for_each::ForEach,
            format_int::FormatInt,
            format_number::FormatNumber,
            format_timestamp::FormatTimestamp,
            get::Get,
            includes::Includes,
            integer::Integer,
            is_array::IsArray,
            is_boolean::IsBoolean,
            is_empty::IsEmpty,
            is_float::IsFloat,
            is_integer::IsInteger,
            is_ipv4::IsIpv4,
            is_ipv6::IsIpv6,
            is_json::IsJson,
            is_null::IsNull,
            is_nullish::IsNullish,
            is_object::IsObject,
            is_regex::IsRegex,
            is_string::IsString,
            is_timestamp::IsTimestamp,
            join::Join,
            keys::Keys,
            length::Length,
            map::Map,
            map_keys::MapKeys,
            map_values::MapValues,
            match_any::MatchAny,
            match_array::MatchArray,
            merge::Merge,
            mod_func::Mod,
            object::Object,
            object_from_array::ObjectFromArray,
            parse_float::ParseFloat,
            parse_int::ParseInt,
            pop::Pop,
            push::Push,
            r#match::Match,
            remove::Remove,
            replace::Replace,
            replace_with::ReplaceWith,
            round::Round,
            set::Set,
            sieve::Sieve,
            slice::Slice,
            split::Split,
            split_path::SplitPath,
            starts_with::StartsWith,
            string::String,
            strip_whitespace::StripWhitespace,
            strlen::Strlen,
            tag_types_externally::TagTypesExternally,
            tally::Tally,
            tally_value::TallyValue,
            timestamp::Timestamp,
            to_bool::ToBool,
            to_float::ToFloat,
            to_int::ToInt,
            to_regex::ToRegex,
            to_string::ToString,
            truncate::Truncate,
            unique::Unique,
            unnest::Unnest,
            upcase::Upcase,
            values::Values,
            zip::Zip,
            self::array::Array,
        }
    }
}

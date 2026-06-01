use crate::compiler::prelude::*;

fn any<T>(value: Value, ctx: &mut Context, runner: &closure::Runner<T>) -> Resolved
where
    T: Fn(&mut Context) -> Resolved,
{
    const TRUE: Value = Value::Boolean(true);
    for item in value.into_iter(false) {
        if match item {
            IterItem::KeyValue(key, value) => runner.run_key_value(ctx, key, value)? == TRUE,
            IterItem::IndexValue(index, value) => {
                runner.run_index_value(ctx, index, value)? == TRUE
            }
            IterItem::Value(value) => *value == TRUE,
        } {
            return Ok(TRUE);
        }
    }

    Ok(Value::Boolean(false))
}

#[derive(Clone, Copy, Debug)]
pub struct Any;

impl Function for Any {
    fn identifier(&self) -> &'static str {
        "any"
    }

    fn usage(&self) -> &'static str {
        indoc! {"
            Iterate over a collection, returning true if a closure value is true.

            The function uses the \"function closure syntax\" to allow reading
            the key/value or index/value combination for each item in the
            collection, until a closure returns true.

            The same scoping rules apply to closure blocks as they do for
            regular blocks. This means that any variable defined in parent scopes
            is accessible, and mutations to those variables are preserved,
            but any new variables instantiated in the closure block are
            unavailable outside of the block.

            See the examples below to learn about the closure syntax.
        "}
    }

    fn category(&self) -> &'static str {
        Category::Enumerate.as_ref()
    }

    fn return_kind(&self) -> u16 {
        kind::BOOLEAN
    }
    fn parameters(&self) -> &'static [Parameter] {
        const PARAMETERS: &[Parameter] = &[Parameter::required(
            "value",
            kind::OBJECT | kind::ARRAY,
            "The array or object to iterate.",
        )];
        PARAMETERS
    }

    fn examples(&self) -> &'static [Example] {
        &[
            example! {
                title: "Find out if there is a 'b'",
                source: indoc! {r#"
                    .tags = ["foo", "bar", "foo", "baz"]
                    any(array(.tags)) -> |_index, value| {
                        contains(string(value), "b")
                    }
                "#},
                result: Ok("true"),
            },
            example! {
                title: "Find if any key matches a value",
                source: indoc! {r#"
                    any({ "a": 1, "b": 2 }) -> |_key, value| {
                        value == 3
                    }
                "#},
                result: Ok("false"),
            },
        ]
    }

    fn compile(
        &self,
        _state: &state::TypeState,
        _ctx: &mut FunctionCompileContext,
        arguments: ArgumentList,
    ) -> Compiled {
        let value = arguments.required("value");
        let closure = arguments.required_closure()?;

        Ok(AnyFn { value, closure }.as_expr())
    }

    fn closure(&self) -> Option<closure::Definition> {
        use closure::{Definition, Input, Output, Variable, VariableKind};

        Some(Definition {
            inputs: vec![Input {
                parameter_keyword: "value",
                kind: Kind::object(Collection::any()).or_array(Collection::any()),
                variables: vec![
                    Variable {
                        kind: VariableKind::TargetInnerKey,
                    },
                    Variable {
                        kind: VariableKind::TargetInnerValue,
                    },
                ],
                output: Output::Kind(Kind::boolean()),
                example: example! {
                    title: "iterate array",
                    source: "any([1, 2]) -> |_index, value| { value > 2 }",
                    result: Ok("false"),
                },
            }],
            is_iterator: true,
        })
    }
}

#[derive(Debug, Clone)]
struct AnyFn {
    value: Box<dyn Expression>,
    closure: Closure,
}

impl FunctionExpression for AnyFn {
    fn resolve(&self, ctx: &mut Context) -> ExpressionResult<Value> {
        let value = self.value.resolve(ctx)?;
        let Closure {
            variables,
            block,
            block_type_def: _,
        } = &self.closure;
        let runner = closure::Runner::new(variables, |ctx| block.resolve(ctx));

        any(value, ctx, &runner)
    }

    fn type_def(&self, _ctx: &state::TypeState) -> TypeDef {
        TypeDef::boolean()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{compiler::compile, diagnostic::Formatter, prelude::state::RuntimeState};

    use super::*;

    #[test]
    fn any_array() -> anyhow::Result<()> {
        let output = run_script("any([1, 2, 3, 4]) -> |_i, num| { num > 2 }")??;
        assert_eq!(Value::Boolean(true), output);
        let output = run_script("any([1, 2, 3, 4]) -> |_i, num| { num < 0 }")??;
        assert_eq!(Value::Boolean(false), output);
        let output = run_script("any([1, 2]) -> |_i, num| { true }")??;
        assert_eq!(Value::Boolean(true), output);
        Ok(())
    }

    #[test]
    fn any_object() -> anyhow::Result<()> {
        let output =
            run_script(r#"any({"a": 1, "b": 2}) -> |key, num| { key == "a" && num < 2 }"#)??;
        assert_eq!(Value::Boolean(true), output);
        let output = run_script(r#"any({"a": 1, "b": 2}) -> |key, _num| { key == "c" }"#)??;
        assert_eq!(Value::Boolean(false), output);
        let output = run_script(r#"any({"a": 1, "b": 2}) -> |key, num| { false }"#)??;
        assert_eq!(Value::Boolean(false), output);
        Ok(())
    }

    #[test]
    fn enforce_boolean_closure_value() {
        let err = compile("any([1, 2]) -> |_i, num| { 5 }", &[Box::new(Any)])
            .expect_err("should not compile");
        assert_eq!(1, err.len());
        let diagnostic = &err[0];
        assert_eq!(
            "type mismatch in closure return type".to_string(),
            diagnostic.message
        );
        assert_eq!(122, diagnostic.code);
    }

    fn run_script(script: &str) -> anyhow::Result<Resolved> {
        let compiled = compile(script, &[Box::new(Any)])
            .map_err(|e| anyhow::anyhow!("Compilation error: {}", Formatter::new(script, e)))?;
        let mut state = RuntimeState::default();
        let mut target: Value = BTreeMap::default().into();
        let tz = TimeZone::default();
        let mut ctx = Context::new(&mut target, &mut state, &tz);
        Ok(compiled.program.resolve(&mut ctx))
    }
}

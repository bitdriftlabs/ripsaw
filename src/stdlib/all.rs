use crate::compiler::prelude::*;

fn all<T>(value: Value, ctx: &mut Context, runner: &closure::Runner<T>) -> Resolved
where
    T: Fn(&mut Context) -> Resolved,
{
    const FALSE: Value = Value::Boolean(false);
    for item in value.into_iter(false) {
        if match item {
            IterItem::KeyValue(key, value) => runner.run_key_value(ctx, key, value)? == FALSE,
            IterItem::IndexValue(index, value) => {
                runner.run_index_value(ctx, index, value)? == FALSE
            }
            IterItem::Value(value) => *value == FALSE,
        } {
            return Ok(FALSE);
        }
    }

    Ok(Value::Boolean(true))
}

#[derive(Clone, Copy, Debug)]
pub struct All;

impl Function for All {
    fn identifier(&self) -> &'static str {
        "all"
    }

    fn usage(&self) -> &'static str {
        indoc! {"
            Iterate over a collection, returning false if a closure value is false.

            The function uses the \"function closure syntax\" to allow reading
            the key/value or index/value combination for each item in the
            collection, until a closure returns false.

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
        &[example! {
            title: "Find out if all values contain 'b'",
            source: indoc! {r#"
                    .tags = ["boo", "bar", "baz"]
                    any(array(.tags)) -> |_index, value| {
                        contains(string(value), "b")
                    }
                "#},
            result: Ok("true"),
        }]
    }

    fn compile(
        &self,
        _state: &state::TypeState,
        _ctx: &mut FunctionCompileContext,
        arguments: ArgumentList,
    ) -> Compiled {
        let value = arguments.required("value");
        let closure = arguments.required_closure()?;

        Ok(AllFn { value, closure }.as_expr())
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
                    source: "all([1, 2]) -> |_index, value| { value < 2 }",
                    result: Ok("false"),
                },
            }],
            is_iterator: true,
        })
    }
}

#[derive(Debug, Clone)]
struct AllFn {
    value: Box<dyn Expression>,
    closure: Closure,
}

impl FunctionExpression for AllFn {
    fn resolve(&self, ctx: &mut Context) -> ExpressionResult<Value> {
        let value = self.value.resolve(ctx)?;
        let Closure {
            variables,
            block,
            block_type_def: _,
        } = &self.closure;
        let runner = closure::Runner::new(variables, |ctx| block.resolve(ctx));

        all(value, ctx, &runner)
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
    fn all_array() -> anyhow::Result<()> {
        let output = run_script("all([1, 2, 3, 4]) -> |_i, num| { num > 2 }")??;
        assert_eq!(Value::Boolean(false), output);
        let output = run_script("all([1, 2, 3, 4]) -> |_i, num| { num > 0 }")??;
        assert_eq!(Value::Boolean(true), output);
        let output = run_script("all([1, 2]) -> |i, _num| { i < 3 }")??;
        assert_eq!(Value::Boolean(true), output);
        let output = run_script("all([1, 2]) -> |_i, _num| { true }")??;
        assert_eq!(Value::Boolean(true), output);
        Ok(())
    }

    #[test]
    fn all_object() -> anyhow::Result<()> {
        let output =
            run_script(r#"all({"a": 1, "b": 2}) -> |key, num| { key != "c" && num > 0 }"#)??;
        assert_eq!(Value::Boolean(true), output);
        let output = run_script(r#"all({"a": 1, "b": 2}) -> |key, _num| { key == "a" }"#)??;
        assert_eq!(Value::Boolean(false), output);
        let output = run_script(r#"all({"a": 1, "b": 2}) -> |_key, _num| { false }"#)??;
        assert_eq!(Value::Boolean(false), output);
        Ok(())
    }

    #[test]
    fn enforce_boolean_closure_value() {
        let err = compile("all([1, 2]) -> |_i, num| { 5 }", &[Box::new(All)])
            .expect_err("should not compile");
        assert_eq!(1, err.len());
        let diagnostic = &err[0];
        assert_eq!(122, diagnostic.code);
    }

    fn run_script(script: &str) -> anyhow::Result<Resolved> {
        let compiled = compile(script, &[Box::new(All)])
            .map_err(|e| anyhow::anyhow!("Compilation error: {}", Formatter::new(script, e)))?;
        let mut state = RuntimeState::default();
        let mut target: Value = BTreeMap::default().into();
        let tz = TimeZone::default();
        let mut ctx = Context::new(&mut target, &mut state, &tz);
        Ok(compiled.program.resolve(&mut ctx))
    }
}

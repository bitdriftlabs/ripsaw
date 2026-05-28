use crate::compiler::prelude::*;

fn map<T>(value: Value, ctx: &mut Context, runner: &closure::Runner<T>) -> Resolved
where
    T: Fn(&mut Context) -> Resolved,
{
    match value {
        Value::Array(array) => array
            .into_iter()
            .enumerate()
            .map(|(index, value)| runner.run_index_value(ctx, index, &value))
            .collect::<ExpressionResult<Vec<_>>>()
            .map(Into::into),

        _ => Err("function requires array type as input".into()),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Map;

impl Function for Map {
    fn identifier(&self) -> &'static str {
        "map"
    }

    fn usage(&self) -> &'static str {
        indoc! {"
            Map elements in an array.

            The function uses the function closure syntax to allow reading
            the index-value combination for each item in the collection,
            returning the result.

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
        kind::ARRAY
    }

    fn parameters(&self) -> &'static [Parameter] {
        const PARAMETERS: &[Parameter] = &[Parameter::required(
            "value",
            kind::ARRAY,
            "The array to map.",
        )];
        PARAMETERS
    }

    fn examples(&self) -> &'static [Example] {
        &[example! {
            title: "Map elements",
            source: indoc! {r#"
                    . = { "tags": ["foo", "bar", "foo", "baz"] }
                    map(array(.tags)) -> |_index, value| {
                        upcase(value)
                    }
                "#},
            result: Ok(r#"["FOO", "BAR", "FOO", "BAZ"]"#),
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

        Ok(MapFn { value, closure }.as_expr())
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
                output: Output::Kind(Kind::any()),
                example: example! {
                    title: "map array",
                    source: "map([1, 2]) -> |_index, value| { value * 2 }",
                    result: Ok("[2, 4]"),
                },
            }],
            is_iterator: true,
        })
    }
}

#[derive(Debug, Clone)]
struct MapFn {
    value: Box<dyn Expression>,
    closure: Closure,
}

impl FunctionExpression for MapFn {
    fn resolve(&self, ctx: &mut Context) -> ExpressionResult<Value> {
        let value = self.value.resolve(ctx)?;
        let Closure {
            variables,
            block,
            block_type_def: _,
        } = &self.closure;
        let runner = closure::Runner::new(variables, |ctx| block.resolve(ctx));

        map(value, ctx, &runner)
    }

    fn type_def(&self, ctx: &state::TypeState) -> TypeDef {
        let mut type_def = self.value.type_def(ctx);

        // Erase any type information from the array, as we can't know
        // what the resulting element types are
        type_def.kind_mut().add_array(Collection::any());

        type_def
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{compiler::compile, diagnostic::Formatter, prelude::state::RuntimeState};

    use super::*;

    #[test]
    fn map_ints() -> anyhow::Result<()> {
        let script = r"map([1, 2, 3, 4]) -> |_i, num| { num * 3 }";
        let compiled = compile(script, &[Box::new(Map)])
            .map_err(|e| anyhow::anyhow!("Compilation error: {}", Formatter::new(script, e)))?;
        let mut state = RuntimeState::default();
        let mut target: Value = BTreeMap::default().into();
        let tz = TimeZone::default();
        let mut ctx = Context::new(&mut target, &mut state, &tz);
        let output = compiled.program.resolve(&mut ctx)?;
        assert_eq!(
            Value::Array(vec![
                Value::Integer(3),
                Value::Integer(6),
                Value::Integer(9),
                Value::Integer(12)
            ]),
            output
        );
        Ok(())
    }
}

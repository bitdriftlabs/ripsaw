use crate::compiler::prelude::*;

fn assert(
    condition: Value,
    message: Option<Value>,
    format: Option<String>,
    span: Span,
) -> Resolved {
    if condition.try_boolean()? {
        return Ok(true.into());
    }

    let message = if let Some(message) = message {
        message.try_bytes_utf8_lossy()?.into_owned()
    } else {
        match format {
            Some(string) => format!("assertion failed: {string}"),
            None => "assertion failed".to_owned(),
        }
    };
    Err(ExpressionError::Abort {
        message: Some(message),
        span,
    })
}

#[derive(Clone, Copy, Debug)]
pub struct Assert;

impl Function for Assert {
    fn identifier(&self) -> &'static str {
        "assert"
    }

    fn usage(&self) -> &'static str {
        "Asserts the `condition`, which must be a Boolean expression. The program is aborted with `message` if the condition evaluates to `false`."
    }

    fn category(&self) -> &'static str {
        Category::Debug.as_ref()
    }

    fn internal_failure_reasons(&self) -> &'static [&'static str] {
        &["`condition` evaluates to `false`."]
    }

    fn return_kind(&self) -> u16 {
        kind::BOOLEAN
    }

    fn notices(&self) -> &'static [&'static str] {
        &[indoc! {"
            The `assert` function should be used in a standalone fashion and only when you want
            to abort the program. You should avoid it in logical expressions and other situations
            in which you want the program to continue if the condition evaluates to `false`.
        "}]
    }

    fn pure(&self) -> bool {
        false
    }

    fn parameters(&self) -> &'static [Parameter] {
        const PARAMETERS: &[Parameter] = &[
            Parameter::required("condition", kind::BOOLEAN, "The condition to check."),
            Parameter::optional(
                "message",
                kind::BYTES,
                "An optional custom error message. If the equality assertion fails, `message` is
appended to the default message prefix. See the [examples](#assert-examples) below
for a fully formed log message sample.",
            ),
        ];
        PARAMETERS
    }

    fn examples(&self) -> &'static [Example] {
        &[
            example! {
                title: "Assertion (true) - with message",
                source: r#"assert!("foo" == "foo", message: "\"foo\" must be \"foo\"!")"#,
                result: Ok("true"),
            },
            example! {
                title: "Assertion (false) - with message",
                source: r#"assert!("foo" == "bar", message: "\"foo\" must be \"foo\"!")"#,
                result: Err(r#""foo" must be "foo"!"#),
            },
            example! {
                title: "Assertion (false) - simple",
                source: "assert!(false)",
                result: Err(r"assertion failed"),
            },
        ]
    }

    fn compile(
        &self,
        _state: &state::TypeState,
        ctx: &mut FunctionCompileContext,
        arguments: ArgumentList,
    ) -> Compiled {
        let condition = arguments.required("condition");
        let message = arguments.optional("message");

        Ok(AssertFn {
            condition,
            span: ctx.span(),
            message,
        }
        .as_expr())
    }
}

#[derive(Debug, Clone)]
struct AssertFn {
    condition: Box<dyn Expression>,
    span: Span,
    message: Option<Box<dyn Expression>>,
}

impl FunctionExpression for AssertFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let condition = self.condition.resolve(ctx)?;
        let format = self.condition.format();
        let message = self.message.as_ref().map(|m| m.resolve(ctx)).transpose()?;

        assert(condition, message, format, self.span)
    }

    fn type_def(&self, _: &state::TypeState) -> TypeDef {
        TypeDef::boolean().fallible()
    }
}

impl fmt::Display for AssertFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{compiler::compile, diagnostic::Formatter, prelude::state::RuntimeState};

    use super::*;

    #[test]
    fn assert_pass_test() -> anyhow::Result<()> {
        let output = run_script(r#"x = 3; assert!(x == 3, "x must be 3")"#)??;
        assert_eq!(Value::Boolean(true), output);
        Ok(())
    }

    #[test]
    fn assert_error_message_test() -> anyhow::Result<()> {
        use ExpressionError::*;

        let script = r#"x = 3; assert!(x == 2, "x must be 2")"#;
        match run_script(script)?.expect_err("must fail") {
            Abort { span, message } => {
                assert_eq!(Some("x must be 2".to_string()), message);
                assert_eq!(7..script.len(), span.range());
            }
            other => anyhow::bail!("received wrong type of error: {other:#?}"),
        }
        Ok(())
    }

    #[test]
    fn assert_error_no_message_test() -> anyhow::Result<()> {
        use ExpressionError::*;

        let script = "x = 3; assert!(x == 2)";
        match run_script(script)?.expect_err("must fail") {
            Abort { span, message } => {
                assert_eq!(Some("assertion failed".to_string()), message);
                assert_eq!(7..script.len(), span.range());
            }
            other => anyhow::bail!("received wrong type of error: {other:#?}"),
        }
        Ok(())
    }

    fn run_script(script: &str) -> anyhow::Result<Resolved> {
        let compiled = compile(script, &[Box::new(Assert)])
            .map_err(|e| anyhow::anyhow!("Compilation error: {}", Formatter::new(script, e)))?;
        let mut state = RuntimeState::default();
        let mut target: Value = BTreeMap::default().into();
        let tz = TimeZone::default();
        let mut ctx = Context::new(&mut target, &mut state, &tz);
        Ok(compiled.program.resolve(&mut ctx))
    }
}

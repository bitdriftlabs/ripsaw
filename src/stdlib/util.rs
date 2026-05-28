use crate::compiler::{Context, Expression, Resolved, TypeState};
use crate::value::Value;

/// Rounds the given number to the given precision.
/// Takes a function parameter so the exact rounding function (ceil, floor or round)
/// can be specified.
#[inline]
#[allow(clippy::cast_precision_loss)] //TODO evaluate removal options
pub(crate) fn round_to_precision<F>(num: f64, precision: i64, fun: F) -> f64
where
    F: Fn(f64) -> f64,
{
    let multiplier = 10_f64.powf(precision as f64);
    fun(num * multiplier) / multiplier
}

pub(crate) fn is_nullish(value: &Value) -> bool {
    match value {
        Value::Bytes(v) => {
            if v.is_empty() || v.as_ref() == b"-" {
                return true;
            }

            let s = value.as_str().expect("value should be bytes");
            s.chars().all(char::is_whitespace)
        }
        Value::Null => true,
        _ => false,
    }
}

#[derive(Clone, Debug)]
pub(super) enum ConstOrExpr {
    Const(Value),
    Expr(Box<dyn Expression>),
}

impl ConstOrExpr {
    pub(super) fn new(expr: Box<dyn Expression>, state: &TypeState) -> Self {
        match expr.resolve_constant(state) {
            Some(cnst) => Self::Const(cnst),
            None => Self::Expr(expr),
        }
    }

    pub(super) fn optional(expr: Option<Box<dyn Expression>>, state: &TypeState) -> Option<Self> {
        expr.map(|expr| Self::new(expr, state))
    }

    pub(super) fn resolve(&self, ctx: &mut Context) -> Resolved {
        match self {
            Self::Const(value) => Ok(value.clone()),
            Self::Expr(expr) => expr.resolve(ctx),
        }
    }
}

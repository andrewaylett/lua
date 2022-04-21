use crate::ast::expression::Expression;
use anyhow::{Context, Result};
use pest::iterators::Pairs;

use crate::eval::{try_static_eval, Value, GLOBAL_CONTEXT};
use crate::parse::parse;
use crate::parse::Rule;

#[track_caller]
pub(crate) fn assert_static_expression(expected: Value, expression: &Expression) {
    let r = try_static_eval(expression).unwrap();
    assert_eq!(
        expected, r,
        "Expression was {:?} => {:?}",
        expression.span, expression.inner
    );
}

#[track_caller]
pub(crate) async fn assert_expression(expected: Value, expression: &Expression<'_>) {
    let r = expression.evaluate(&*GLOBAL_CONTEXT).await;
    assert_eq!(
        Ok(expected),
        r,
        "Expression was {:?} => {:?}",
        expression.span,
        expression.inner
    );
}

#[track_caller]
pub(crate) fn parse_expression(input: &str) -> Result<Pairs<Rule>> {
    let p = parse(Rule::expression, input)?;
    let first = p.peek().context("Expected a parse")?;
    assert_eq!(first.as_span().start(), 0);
    assert_eq!(first.as_span().end(), input.len());
    Ok(p)
}

macro_rules! test_static_eval {
    ($func_name:ident, $input:expr, $result:expr) => {
        #[test]
        fn $func_name() -> Result<()> {
            let p = parse_expression($input)?;
            let e = Expression::try_from(p)?;
            assert_static_expression($result, &e);
            Ok(())
        }
    };
}

macro_rules! test_eval_inner {
    ($func_name:ident, $input:expr, $result:expr) => {
        #[tokio::test]
        async fn $func_name() -> Result<()> {
            let p = parse_expression($input)?;
            let e = Expression::try_from(p)?;
            assert_expression($result, &e).await;
            Ok(())
        }
    };
}

macro_rules! test_eval {
    ($name:ident, $input:expr, $result:expr) => {
        mod $name {
            use super::*;
            test_static_eval!(static_eval, $input, $result);
            test_eval_inner!(eval, $input, $result);
        }
    };
}

macro_rules! test_eval_int {
    ($name:ident, $input:expr) => {
        test_eval!($name, stringify!($input), Value::Int($input));
    };
}

test_eval_int!(int_add, 1 + 2);
test_eval_int!(multiply_higher_precedence_than_add, 2 + 3 * 4);
test_eval_int!(parens, (1 + 2) * 3);
test_eval_int!(negative, -1);
test_eval_int!(bit_and, 6 & 3);
test_eval_int!(bit_or, 1 | 2);
test_eval_int!(shl, 13 << 20);
test_eval_int!(shr, 100000 >> 10);
test_eval_int!(xor, 6 ^ 10);

test_eval!(bit_nand, "6 &^ 10", Value::Int(4));

test_eval!(bit_xor, "6 ^ 10", Value::Int(12));

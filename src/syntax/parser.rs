use super::ast::Expr;
use chumsky::prelude::*;

pub fn expr_parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> {
    recursive(|expr| {
        let num = text::int(10)
            .map(|s: &str| Expr::Num(s.parse().unwrap()))
            .padded();

        let list = expr
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just('['), just(']'))
            .map(Expr::List)
            .padded();

        num.or(list)
    })
    .then_ignore(end())
}

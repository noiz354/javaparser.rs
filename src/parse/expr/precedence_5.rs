use parse::combinator::{any_symbol, get_and_not_followed_by, symbol};
use parse::expr::precedence_6;
use parse::id_gen::IdGen;
use parse::tree::{BinaryOperation, Expr};
use parse::{ParseResult, Tokens};

pub fn parse_tail<'def, 'r>(
    left: Expr<'def>,
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok((input, operator)) = get_and_not_followed_by(symbol('|'), any_symbol("|="))(input) {
        let (input, right) = precedence_6::parse(input, id_gen)?;

        let expr = Expr::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        });

        parse_tail(expr, input, id_gen)
    } else {
        Ok((input, left))
    }
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, left) = precedence_6::parse(input, id_gen)?;
    parse_tail(left, input, id_gen)
}

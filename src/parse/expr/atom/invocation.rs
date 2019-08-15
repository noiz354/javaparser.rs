use either::Either;
use parse::combinator::{identifier, separated_list, symbol};
use parse::expr::atom::name;
use parse::tpe::type_args;
use parse::tree::{Expr, Keyword, MethodCall, Name, SuperConstructorCall, TypeArg};
use parse::{expr, ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse_args(input: Tokens) -> ParseResult<Vec<Expr>> {
    let (input, _) = symbol('(')(input)?;
    let (input, args) = separated_list(symbol(','), expr::parse)(input)?;
    let (input, _) = symbol(')')(input)?;

    Ok((input, args))
}

pub fn parse_tail<'a>(
    input: Tokens<'a>,
    prefix_opt: Option<Expr<'a>>,
    keyword_or_name: Either<Keyword<'a>, Name<'a>>,
    type_args_opt: Option<Vec<TypeArg<'a>>>,
) -> ParseResult<'a, Expr<'a>> {
    let (input, args) = parse_args(input)?;

    match keyword_or_name {
        Either::Left(keyword) => {
            if keyword.name.fragment == "super" {
                if let Some(Expr::This(this)) = prefix_opt {
                    return Ok((
                        input,
                        Expr::SuperConstructorCall(SuperConstructorCall {
                            prefix_opt: Some(this),
                            name: keyword.name,
                            type_args_opt,
                            args,
                        }),
                    ));
                }
            }

            Err(input)
        }
        Either::Right(name) => Ok((
            input,
            Expr::MethodCall(MethodCall {
                prefix_opt: prefix_opt.map(Box::new),
                name: name.name,
                type_args_opt,
                args,
            }),
        )),
    }
}

pub fn parse<'a>(input: Tokens<'a>, prefix_opt: Option<Expr<'a>>) -> ParseResult<'a, Expr<'a>> {
    let (input, type_args_opt) = if prefix_opt.is_some() {
        type_args::parse(input)?
    } else {
        (input, None)
    };

    let (input, keyword_or_name) = name::parse(input)?;

    parse_tail(input, prefix_opt, keyword_or_name, type_args_opt)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Expr, Int, Lambda, LiteralString, MethodCall, Param, Type};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(
                &code(
                    r#"
method()
            "#
                ),
                None
            ),
            Ok((
                &[] as Tokens,
                Expr::MethodCall(MethodCall {
                    prefix_opt: None,
                    name: span(1, 1, "method"),
                    type_args_opt: None,
                    args: vec![],
                })
            ))
        );
    }

    #[test]
    fn test_with_args() {
        assert_eq!(
            parse(
                &code(
                    r#"
method(1, "a")
            "#
                ),
                None
            ),
            Ok((
                &[] as Tokens,
                Expr::MethodCall(MethodCall {
                    prefix_opt: None,
                    name: span(1, 1, "method"),
                    type_args_opt: None,
                    args: vec![
                        Expr::Int(Int {
                            value: span(1, 8, "1")
                        }),
                        Expr::String(LiteralString {
                            value: span(1, 11, "\"a\"")
                        }),
                    ],
                })
            ))
        );
    }

    #[test]
    fn test_lambda() {
        assert_eq!(
            parse(
                &code(
                    r#"
method(1, (x) -> 2)
            "#
                ),
                None
            ),
            Ok((
                &[] as Tokens,
                Expr::MethodCall(MethodCall {
                    prefix_opt: None,
                    name: span(1, 1, "method"),
                    type_args_opt: None,
                    args: vec![
                        Expr::Int(Int {
                            value: span(1, 8, "1")
                        }),
                        Expr::Lambda(Lambda {
                            params: vec![Param {
                                modifiers: vec![],
                                tpe: Type::UnknownType,
                                is_varargs: false,
                                name: span(1, 12, "x")
                            }],
                            expr_opt: Some(Box::new(Expr::Int(Int {
                                value: span(1, 18, "2")
                            }))),
                            block_opt: None
                        }),
                    ],
                })
            ))
        );
    }
}
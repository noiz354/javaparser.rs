use either::Either;
use parse::combinator::{identifier, separated_list, symbol, symbol2};
use parse::def::param;
use parse::expr::atom::name;
use parse::statement::block::parse_block;
use parse::tree::{Block, Expr, Lambda, Param, Type};
use parse::{expr, ParseResult, Tokens};

fn parse_block_or_single_expr(input: Tokens) -> ParseResult<Either<Block, Expr>> {
    match parse_block(input) {
        Ok((input, block)) => Ok((input, Either::Left(block))),
        Err(_) => {
            let (input, expr) = expr::parse(input)?;
            Ok((input, Either::Right(expr)))
        }
    }
}

fn parse_param_with_type_or_without_type(input: Tokens) -> ParseResult<Param> {
    match param::parse(input) {
        Ok(result) => Ok(result),
        Err(_) => {
            let (input, name) = identifier(input)?;
            Ok((
                input,
                Param {
                    modifiers: vec![],
                    tpe: Type::UnknownType,
                    is_varargs: false,
                    name,
                },
            ))
        }
    }
}

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, params) = if let Ok((input, _)) = symbol('(')(input) {
        let (input, params) =
            separated_list(symbol(','), parse_param_with_type_or_without_type)(input)?;
        let (input, _) = symbol(')')(input)?;

        (input, params)
    } else if let Ok((input, Either::Right(name))) = name::parse(input) {
        (
            input,
            vec![Param {
                modifiers: vec![],
                tpe: Type::UnknownType,
                is_varargs: false,
                name: name.name,
            }],
        )
    } else {
        return Err(input);
    };

    let (input, _) = symbol2('-', '>')(input)?;

    let (input, block_or_expr) = parse_block_or_single_expr(input)?;

    let (block_opt, expr_opt) = match block_or_expr {
        Either::Left(block) => (Some(block), None),
        Either::Right(expr) => (None, Some(Box::new(expr))),
    };

    Ok((
        input,
        Expr::Lambda(Lambda {
            params,
            expr_opt,
            block_opt,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        Block, ClassType, Expr, Int, Lambda, Param, PrimitiveType, ReturnStmt, Statement, Type,
    };
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_single_with_args() {
        assert_eq!(
            parse(&code(
                r#"
(Test t, a, int i) -> 1
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Lambda(Lambda {
                    params: vec![
                        Param {
                            modifiers: vec![],
                            tpe: Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 2, "Test"),
                                type_args_opt: None
                            }),
                            is_varargs: false,
                            name: span(1, 7, "t"),
                        },
                        Param {
                            modifiers: vec![],
                            tpe: Type::UnknownType,
                            is_varargs: false,
                            name: span(1, 10, "a"),
                        },
                        Param {
                            modifiers: vec![],
                            tpe: Type::Primitive(PrimitiveType {
                                name: span(1, 13, "int")
                            }),
                            is_varargs: false,
                            name: span(1, 17, "i"),
                        }
                    ],
                    expr_opt: Some(Box::new(Expr::Int(Int {
                        value: span(1, 23, "1")
                    }))),
                    block_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_simple() {
        assert_eq!(
            parse(&code(
                r#"
(x) -> 2
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Lambda(Lambda {
                    params: vec![Param {
                        modifiers: vec![],
                        tpe: Type::UnknownType,
                        is_varargs: false,
                        name: span(1, 2, "x"),
                    }],
                    expr_opt: Some(Box::new(Expr::Int(Int {
                        value: span(1, 8, "2")
                    }))),
                    block_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_simple2() {
        assert_eq!(
            parse(&code(
                r#"
x -> 2
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Lambda(Lambda {
                    params: vec![Param {
                        modifiers: vec![],
                        tpe: Type::UnknownType,
                        is_varargs: false,
                        name: span(1, 1, "x"),
                    }],
                    expr_opt: Some(Box::new(Expr::Int(Int {
                        value: span(1, 6, "2")
                    }))),
                    block_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_block() {
        assert_eq!(
            parse(&code(
                r#"
() -> { return 1; }
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Lambda(Lambda {
                    params: vec![],
                    expr_opt: None,
                    block_opt: Some(Block {
                        stmts: vec![Statement::Return(ReturnStmt {
                            expr_opt: Some(Expr::Int(Int {
                                value: span(1, 16, "1")
                            }))
                        })]
                    })
                })
            ))
        );
    }
}
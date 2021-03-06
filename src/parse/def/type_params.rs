use parse::combinator::{identifier, keyword, separated_list, separated_nonempty_list, symbol};
use parse::id_gen::IdGen;
use parse::tpe::class;
use parse::tree::{ClassType, TypeParam, TypeParamExtend};
use parse::{ParseResult, Tokens};
use std::cell::RefCell;

pub fn parse_extends<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Vec<TypeParamExtend<'def>>> {
    if let Ok((input, _)) = keyword("extends")(input) {
        separated_nonempty_list(symbol('&'), |input| {
            let (input, c) = class::parse_no_array(input)?;
            Ok((input, TypeParamExtend::Class(c)))
        })(input)
    } else {
        Ok((input, vec![]))
    }
}

pub fn parse_type_param<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, TypeParam<'def>> {
    let (input, name) = identifier(input)?;
    let (input, extends) = parse_extends(input)?;

    Ok((
        input,
        TypeParam {
            name,
            extends,
            def_opt: RefCell::new(None),
            id: id_gen.get_next("TypeParam", name.fragment),
        },
    ))
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Vec<TypeParam<'def>>> {
    if let Ok((input, _)) = symbol('<')(input) {
        let (input, type_params) =
            separated_list(symbol(','), |i| parse_type_param(i, id_gen))(input)?;
        let (input, _) = symbol('>')(input)?;
        Ok((input, type_params))
    } else {
        Ok((input, vec![]))
    }
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{ClassType, TypeArg, TypeParam};
//    use parse::Tokens;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//<A, B extends A, C extends String & Another<A>>
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                vec![
//                    TypeParam {
//                        name: span(1, 2, "A"),
//                        extends: vec![]
//                    },
//                    TypeParam {
//                        name: span(1, 5, "B"),
//                        extends: vec![ClassType {
//                            prefix_opt: None,
//                            name: span(1, 15, "A"),
//                            type_args_opt: None,
//                            def_opt: None
//                        }]
//                    },
//                    TypeParam {
//                        name: span(1, 18, "C"),
//                        extends: vec![
//                            ClassType {
//                                prefix_opt: None,
//                                name: span(1, 28, "String"),
//                                type_args_opt: None,
//                                def_opt: None
//                            },
//                            ClassType {
//                                prefix_opt: None,
//                                name: span(1, 37, "Another"),
//                                type_args_opt: Some(vec![TypeArg::Class(ClassType {
//                                    prefix_opt: None,
//                                    name: span(1, 45, "A"),
//                                    type_args_opt: None,
//                                    def_opt: None
//                                })]),
//                                def_opt: None
//                            }
//                        ]
//                    },
//                ]
//            ))
//        );
//    }
//
//    #[test]
//    fn test_empty() {
//        assert_eq!(parse(&generate_tokens("")), Ok((&[] as Tokens, vec![])));
//    }
//}

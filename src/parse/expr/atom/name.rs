use either::Either;
use parse::combinator::{any_keyword, identifier};
use parse::tpe::primitive;
use parse::tree::{Keyword, Name};
use parse::{ParseResult, Tokens};
use std::cell::Cell;
use tokenize::span::Span;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Either<Keyword<'def>, Name<'def>>> {
    if let Ok((input, name)) = identifier(input) {
        Ok((
            input,
            Either::Right(Name {
                name,
                resolved_opt: Cell::new(None),
            }),
        ))
    } else if let Ok((input, name)) = any_keyword(input) {
        Ok((input, Either::Left(Keyword { name })))
    } else {
        Err(input)
    }
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use either::Either;
//    use parse::tree::Name;
//    use parse::Tokens;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_bare() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//name_something
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Either::Right(Name {
//                    name: span(1, 1, "name_something"),
//                })
//            ))
//        );
//    }
//}

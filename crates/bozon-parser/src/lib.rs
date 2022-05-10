#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use chumsky::prelude::*;

use bozon_ast::*;

fn prefix() -> impl Parser<char, PrefixKind, Error = Simple<char>> {
    just('\'')
        .map(|_| PrefixKind::Quote)
        .or(just('`').map(|_| PrefixKind::QuasiQuote))
        .or(just(',').map(|_| PrefixKind::Unquote))
        .or(just(",@").map(|_| PrefixKind::UnquoteSplicing))
        .labelled("prefix")
}

fn ident() -> impl Parser<char, AtomKind, Error = Simple<char>> {
    // XXX: Maybe include more whitespace symbols?
    none_of::<_, _, Simple<char>>("()[]{}\t\n\r ")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(AtomKind::Ident)
        .labelled("ident")
}

fn string() -> impl Parser<char, AtomKind, Error = Simple<char>> {
    none_of::<_, _, Simple<char>>("\"")
        .repeated()
        .delimited_by(just('"'), just('"'))
        .collect::<String>()
        .map(AtomKind::String)
        .labelled("string")
}

/// Returns program parser
///
/// # Example
///
/// ```
/// use bozon_parser::program;
/// use bozon_ast::*;
/// use bozon_span::Span;
/// use chumsky::Parser;
///
/// let src = "(+ 1 1)";
/// let ast = vec![Atom {
///     prefix: None,
///     kind: AtomKind::List(
///         vec![
///             Atom {
///                 prefix: None,
///                 kind: AtomKind::Ident("+".into()),
///                 span: Span::new(1, 2)
///             },
///             Atom {
///                 prefix: None,
///                 kind: AtomKind::Ident("1".into()),
///                 span: Span::new(3, 4)
///             },
///             Atom {
///                 prefix: None,
///                 kind: AtomKind::Ident("1".into()),
///                 span: Span::new(5, 6)
///             }
///         ],
///         BracketKind::Round
///     ),
///     span: Span::new(0, 7)
/// }];
/// assert_eq!(program().parse(src), Ok(ast));
///
/// ```
pub fn program() -> impl Parser<char, Vec<Atom>, Error = Simple<char>> {
    let sexp = recursive(|l| {
        prefix()
            .or_not()
            .then(
                l.clone()
                    .separated_by(text::whitespace())
                    .delimited_by(just('(').padded(), just(')').padded())
                    .map(|lst| AtomKind::List(lst, BracketKind::Round))
                    .or(l
                        .clone()
                        .separated_by(text::whitespace())
                        .delimited_by(just('{').padded(), just('}').padded())
                        .map(|lst| AtomKind::List(lst, BracketKind::Curly)))
                    .or(l
                        .separated_by(text::whitespace())
                        .delimited_by(just('[').padded(), just(']').padded())
                        .map(|lst| AtomKind::List(lst, BracketKind::Square)))
                    .or(string())
                    .or(ident()),
            )
            .map_with_span(|(prefix, kind), span| Atom {
                prefix,
                kind,
                span: span.into(),
            })
            .padded()
    });

    sexp.separated_by(text::whitespace()).collect::<Vec<Atom>>()
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    use bozon_ast::*;
    use bozon_span::Span;

    #[test]
    fn simple() {
        use super::program;

        assert_eq!(
            program().parse("()"),
            Ok(vec![Atom {
                prefix: None,
                kind: AtomKind::List(vec![], BracketKind::Round),
                span: Span::new(0, 2)
            }])
        );

        assert_eq!(
            program().parse("( )"),
            Ok(vec![Atom {
                prefix: None,
                kind: AtomKind::List(vec![], BracketKind::Round),
                span: Span::new(0, 3)
            }])
        );

        assert_eq!(
            super::program().parse("\t\r\n (\t\r\n )"),
            Ok(vec![Atom {
                prefix: None,
                kind: AtomKind::List(vec![], BracketKind::Round),
                span: Span::new(4, 10)
            }])
        );
    }

    #[quickcheck]
    fn prefix(input: String) -> bool {
        // NOTE: is_ascii_alphanumeric because with is_alphanumeric it sometimes fails on \u{80} character for some reason.
        (input.chars().all(|x| char::is_ascii_alphanumeric(&x)) && input.starts_with('\'')
            || input.starts_with('`')
            || input.starts_with(',')
            || input.starts_with(",@")
            || input.starts_with("#'")
            || input.starts_with("#`")
            || input.starts_with("#,")
            || input.starts_with("#,@")) as u8
            <= super::prefix().parse(input).is_ok() as u8
    }

    #[test]
    fn span() {
        assert_eq!(
            super::string()
                .map_with_span(|kind, span| Atom::new(None, kind, span))
                .parse(r#"""  "#),
            Ok(Atom {
                prefix: None,
                kind: AtomKind::String(r#""#.into()),
                span: Span::new(0, 2)
            }),
        );
    }

    #[quickcheck]
    fn string(input: String) -> bool {
        let parser = super::string().map_with_span(|kind, span| Atom::new(None, kind, span));

        if let Ok(Atom {
            prefix: _,
            kind: AtomKind::String(s),
            span,
        }) = parser.parse(input)
        {
            span.len() == s.len() + 2
        } else {
            true
        }
    }

    #[test]
    fn ident() {
        assert_eq!(
            super::ident().parse("hellow"),
            Ok(AtomKind::Ident("hellow".into())),
        );
        assert_eq!(super::ident().parse("1"), Ok(AtomKind::Ident("1".into())),);
        assert_eq!(
            super::ident().parse("42 "),
            Ok(AtomKind::Ident("42".into())),
        );
    }

    #[test]
    fn list_whitespaces() {
        use super::program;

        assert_eq!(
            program().parse(r#"( "Hello")"#),
            Ok(vec![Atom {
                prefix: None,
                kind: AtomKind::List(
                    vec![Atom {
                        prefix: None,
                        kind: AtomKind::String("Hello".into()),
                        span: Span::new(2, 9)
                    }],
                    BracketKind::Round
                ),
                span: Span::new(0, 10)
            }])
        );

        assert_eq!(
            program().parse("(1)"),
            Ok(vec![Atom {
                prefix: None,
                kind: AtomKind::List(
                    vec![Atom {
                        prefix: None,
                        kind: AtomKind::Ident("1".into()),
                        span: Span::new(1, 2)
                    }],
                    BracketKind::Round
                ),
                span: Span::new(0, 3)
            }])
        );

        assert_eq!(
            program().parse("( 1)"),
            Ok(vec![Atom {
                prefix: None,
                kind: AtomKind::List(
                    vec![Atom {
                        prefix: None,
                        kind: AtomKind::Ident("1".into()),
                        span: Span::new(2, 3)
                    }],
                    BracketKind::Round
                ),
                span: Span::new(0, 4)
            }])
        );

        assert_eq!(
            program().parse("(1 )"),
            Ok(vec![Atom {
                prefix: None,
                kind: AtomKind::List(
                    vec![Atom {
                        prefix: None,
                        kind: AtomKind::Ident("1".into()),
                        span: Span::new(1, 2)
                    }],
                    BracketKind::Round
                ),
                span: Span::new(0, 4)
            }])
        );

        assert_eq!(
            program().parse("( 1 )"),
            Ok(vec![Atom {
                prefix: None,
                kind: AtomKind::List(
                    vec![Atom {
                        prefix: None,
                        kind: AtomKind::Ident("1".into()),
                        span: Span::new(2, 3)
                    }],
                    BracketKind::Round
                ),
                span: Span::new(0, 5)
            }])
        );
    }
}

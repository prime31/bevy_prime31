use {
    super::{
        core::{
            nom::{combinator::iterator, error::ErrorKind},
            Input, Parse, ParseResult,
        },
        nom::{self, error::ParseError, IResult},
    },
};

pub use nom_fields::fields;

pub fn parse<'i, T, E>(input: Input<'i>) -> ParseResult<T, E>
where
    E: ParseError<Input<'i>>,
    T: Parse<'i, E>,
{
    T::parse(input)
}

pub fn many_fixed<const N: usize, I, O, E, F>(parser: F) -> impl Fn(I) -> IResult<I, [O; N], E>
where
    E: ParseError<I> + Clone,
    I: Clone,
    F: Fn(I) -> IResult<I, O, E>,
    O: std::fmt::Debug + Copy
{
    move |input| {
        use std::convert::TryInto;

        let mut iter = iterator(input.clone(), &parser);

        let vec = iter.collect::<Vec<_>>();
        iter.finish().and_then(|(rest, _)| match vec.as_slice().try_into() {
            Ok(array) => Ok((rest, array)),
            Err(_) => Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Verify))),
        })
    }
}

pub fn quoted_string<'i, E>(input: &'i str) -> IResult<&'i str, &'i str, E>
where
    E: ParseError<&'i str>,
{
    let mut escaped = false;
    let mut iter = input.chars();

    let error = |ctx| {
        Err(nom::Err::Error(E::add_context(
            input,
            ctx,
            E::from_error_kind(input, ErrorKind::Verify),
        )))
    };

    if let Some('"') = iter.next() {
        let end = iter
            .take_while(|c| match c {
                '"' if !escaped => false,
                '\\' => {
                    escaped = true;
                    true
                }
                _ => {
                    escaped = false;
                    true
                }
            })
            .count()
            + 1;

        if input[end..].starts_with('"') {
            Ok((&input[end + 1..], &input[1..end]))
        } else {
            error("no closing quote")
        }
    } else {
        error("no opening quote")
    }
}

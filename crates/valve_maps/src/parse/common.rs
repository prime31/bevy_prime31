use {
    arrayvec::{Array, ArrayVec},
    super::{
        core::{
            Input,
            Parse,
            ParseResult,
            nom::{
                error::ErrorKind,
                combinator::iterator
            }
        },
        nom::{
            self,
            IResult,
            error::ParseError,
        }
    }
};

pub use nom_fields::fields;

pub fn parse<'i, T, E>(input: Input<'i>) -> ParseResult<T, E>
where
    E: ParseError<Input<'i>>,
    T: Parse<'i, E>
{
    T::parse(input)
}

pub fn many_fixed<A, I, O, E, F>(parser: F) -> impl Fn(I) -> IResult<I, A, E>
where
    A: Array<Item = O>,
    E: ParseError<I> + Clone,
    I: Clone,
    F: Fn(I) -> IResult<I, O, E>
{
    move |input| {
        let mut iter = iterator(input.clone(), &parser);
        let array = iter
            .collect::<ArrayVec<A>>()
            .into_inner();

        iter.finish()
            .and_then(|(rest, _)| match array {
                Ok(array) => Ok((rest, array)),
                Err(_) => Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Verify)))
            })
    }
}

pub fn quoted_string<'i, E>(input: &'i str) -> IResult<&'i str, &'i str, E>
where E: ParseError<&'i str> {
    let mut escaped = false;
    let mut iter = input.chars();

    let error = |ctx| Err(nom::Err::Error(
        E::add_context(input, ctx, E::from_error_kind(input, ErrorKind::Verify))
    ));

    if let Some('"') = iter.next() {
        let end = iter.take_while(|c| match c {
            '"' if !escaped => false,
            '\\' => {
                escaped = true;
                true
            },
            _ => {
                escaped = false;
                true
            }
        }).count() + 1;

        if input[end..].starts_with('"') {
            Ok((&input[end + 1..], &input[1..end]))
        } else {
            error("no closing quote")
        }
    } else {
        error("no opening quote")
    }
}

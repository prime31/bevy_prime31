use {
    ::nom::{
        IResult,
        error::{ErrorKind, ParseError}
    },
};

pub(crate) mod nom {
    pub use nom::{
        *,
        error,
        multi,
        branch,
        methods,
        sequence,
        combinator,
        bits::complete as bits,
        bytes::complete as bytes,
        number::complete as number,
        character::complete as character,
    };
}

/// Type alias for this crate's input type, which is `&str`.
pub type Input<'i> = &'i str;

/// The error type provided by this crate. It is returned by the main [parse](function@crate::parse) function.
/// If you wish to use your own `nom` error type, you may use the [Parse](crate::parse::core::Parse)
/// implementation on [Map](crate::parse::formats::Map) directly, as it's generic over the error type.
/// See the `custom_error` example for a demonstration.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Error<'i> {
    pub input: Input<'i>,
    pub kind: ErrorKind,
    pub context: &'static str
}

impl <'i> ParseError<Input<'i>> for Error<'i> {
    fn from_error_kind(input: Input<'i>, kind: ErrorKind) -> Self {
        Error {
            input,
            kind,
            context: ""
        }
    }

    fn append(input: Input<'i>, kind: ErrorKind, other: Self) -> Self {
        Error {
            input,
            kind,
            ..other
        }
    }

    fn add_context(input: Input<'i>, context: &'static str, other: Self) -> Self {
        Error {
            input,
            context,
            ..other
        }
    }
}

/// Type alias for the Result type used by this crate.
pub type ParseResult<'i, T, E = Error<'i>> = IResult<Input<'i>, T, E>;

/// The main parsing trait of this crate, implemented for every component of a map.
pub trait Parse<'i, E = Error<'i>>
where
    E: ParseError<Input<'i>>,
    Self: Sized
{
    fn parse(input: Input<'i>) -> ParseResult<Self, E>;
}
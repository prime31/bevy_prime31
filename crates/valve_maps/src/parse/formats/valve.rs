use {
    crate::parse::{
        common::{fields, parse},
        formats::{
            shared::{Vector3, separator, sep_terminated, maybe_sep_terminated}
        },
        core::{
            Parse,
            Input,
            ParseResult,
            nom::{
                number::float,
                character::char,
                error::ParseError,
                combinator::{map, opt},
                sequence::{pair, delimited}
            },
        },
    }
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32
}

impl <'i, E> Parse<'i, E> for Vector2
where E: ParseError<Input<'i>> + Clone {
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        fields!(Vector2:
            x = sep_terminated(float),
            y = float
        )(input)
    }
}

/// Valve Software's map format used in Half-Life 1.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Valve;

/// Representation of the Valve format's texture alignment.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct TextureAlignment {
    pub axes: Axes,
    pub rotation: f32,
    pub scale: Scale
}

impl <'i, E> Parse<'i, E> for TextureAlignment
where E: ParseError<Input<'i>> + Clone {
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        fields!(TextureAlignment:
            axes = maybe_sep_terminated(parse),
            rotation = sep_terminated(float),
            scale = parse
        )(input)
    }
}

/// The u and v axes of the Valve format's texture alignment.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Axes {
    pub u: Axis,
    pub v: Axis
}

impl <'i, E> Parse<'i, E> for Axes
where E: ParseError<Input<'i>> + Clone {
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        fields!(Axes:
            u = maybe_sep_terminated(parse),
            v = parse
        )(input)
    }
}

/// A [texture alignment](TextureAlignment) axis in Valve's map format.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Axis {
    pub normal: Vector3,
    pub offset: f32
}

impl <'i, E> Parse<'i, E> for Axis
where E: ParseError<Input<'i>> + Clone {
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        delimited(
            pair(char('['), opt(separator)),
            fields!(Axis:
                normal = sep_terminated(parse),
                offset = float
            ),
            pair(opt(separator), char(']'))
        )(input)
    }
}

/// The scale of a Valve format [Texture](super::shared::Texture).
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Scale {
    pub u: f32,
    pub v: f32
}

impl <'i, E> Parse<'i, E> for Scale
where E: ParseError<Input<'i>> + Clone {
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        map(
            Vector2::parse,
            |vec| Scale {
                u: vec.x,
                v: vec.y
            }
        )(input)
    }
}
use bevy::prelude::Vec3;

use super::valve::TextureAlignment;

use {
    crate::parse::{
        common::{fields, many_fixed, parse, quoted_string},
        core::{
            nom::{
                branch::alt,
                bytes::{tag, take_till},
                character::{char, line_ending, multispace1, not_line_ending},
                combinator::{iterator, map, opt, recognize},
                error::ParseError,
                multi::{fold_many0, many0},
                sequence::{delimited, pair, preceded, terminated},
            },
            Input, Parse, ParseResult,
        },
    },
    std::{
        collections::HashMap,
        ops::{Deref, DerefMut},
    },
};

pub(crate) fn separator<'i, E>(input: Input<'i>) -> ParseResult<Input<'i>, E>
where
    E: ParseError<Input<'i>> + Clone,
{
    recognize(|input| {
        let mut iter = iterator(input, alt((multispace1, terminated(comment, line_ending))));
        iter.for_each(drop);
        iter.finish()
    })(input)
}

pub(crate) fn sep_terminated<'i, F, O, E>(parsed: F) -> impl Fn(Input<'i>) -> ParseResult<O, E>
where
    F: Fn(Input<'i>) -> ParseResult<O, E>,
    E: ParseError<Input<'i>> + Clone,
{
    terminated(parsed, separator)
}

pub(crate) fn maybe_sep_terminated<'i, F, O, E>(parsed: F) -> impl Fn(Input<'i>) -> ParseResult<O, E>
where
    F: Fn(Input<'i>) -> ParseResult<O, E>,
    E: ParseError<Input<'i>> + Clone,
{
    terminated(parsed, opt(separator))
}

/// A wrapper around a `HashMap<String, String>` representing
/// an entity's key/value pairs. In a map file, they usually look
/// something like this:
/// ```plain
/// "classname" "light"
/// "wait" "2"
/// "light" "600"
/// "angle" "315"
/// "mangle" "0 90 0"
/// "origin" "-2704 1908 50"
/// "_color" "1.00 0.93 0.70"
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Fields(pub HashMap<String, String>);

impl Fields {
    pub fn into_inner(self) -> HashMap<String, String> {
        self.0
    }

    pub fn get_property(&self, name: &str) -> Option<&str> {
        if let Some(s) = self.0.get(&String::from(name)) {
            return Some(&s[..]);
        }
        None
    }

    pub fn is_sensor(&self) -> bool {
        if let Some(prop) = self.get("classname") {
            return prop == "sensor";
        }
        false
    }
}

impl Deref for Fields {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Fields {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'i, E> Parse<'i, E> for Fields
where
    E: ParseError<Input<'i>> + Clone,
{
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        map(
            fold_many0(
                maybe_sep_terminated(pair(maybe_sep_terminated(quoted_string), quoted_string)),
                HashMap::new(),
                |mut map, (k, v)| {
                    // filter out internal TrenchBroom data
                    if !k.starts_with("_tb_") {
                        map.insert(k.into(), v.into());
                    }
                    map
                },
            ),
            Fields,
        )(input)
    }
}

/// Representation of a map entity with [key/value pairs](Fields) and a list
/// of [Brush](Brush)es, which may be empty if the entity in question is a
/// point entity, like a light.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MapEntity {
    pub fields: Fields,
    pub brushes: Vec<Brush>,
}

impl<'i, E> Parse<'i, E> for MapEntity
where
    E: ParseError<Input<'i>> + Clone,
{
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        delimited(
            pair(char('{'), opt(separator)),
            fields!(
                MapEntity: fields = maybe_sep_terminated(parse),
                brushes = many0(maybe_sep_terminated(parse))
            ),
            char('}'),
        )(input)
    }
}

/// Representation of a plane with three points describing a half-space and a texture. In a map file, it usually looks
/// something like this with the valve format:
/// ```plain
/// ( 816 -796 356 ) ( 816 -804 356 ) ( 808 -804 356 ) stone1_3 [ 0 -1 0 -20 ] [ 1 0 0 16 ] -0 1 1
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Plane {
    pub points: [Vec3; 3],
    pub texture: Texture,
}

impl Plane {
    pub fn normal(&self) -> Vec3 {
        let v0v1 = self.points[1] - self.points[0];
        let v0v2 = self.points[2] - self.points[0];
        v0v2.cross(v0v1).normalize()
    }

    pub fn dist(&self) -> f32 {
        let n = self.normal();
        n.dot(self.points[0])
    }
}

impl<'i, E> Parse<'i, E> for Plane
where
    E: ParseError<Input<'i>> + Clone,
{
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        fields!(
            Plane: points = many_fixed(maybe_sep_terminated(delimited(
                pair(char('('), opt(separator)),
                parse,
                pair(opt(separator), char(')'))
            ),)),
            texture = parse
        )(input)
    }
}

/// Representation of a texture, consisting of the
/// texture's name and alignment. The format of the
/// latter differs between map formats.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Texture {
    pub name: String,
    pub alignment: TextureAlignment,
}

impl<'i, E> Parse<'i, E> for Texture
where
    E: ParseError<Input<'i>> + Clone,
{
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        fields!(
            Texture: name = map(sep_terminated(take_till(char::is_whitespace)), String::from),
            alignment = parse
        )(input)
    }
}

/// Representation of a map brush, consisting of a
/// list of [Plane](Plane)s.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Brush {
    pub planes: Vec<Plane>,
}

impl<'i, E> Parse<'i, E> for Brush
where
    E: ParseError<Input<'i>> + Clone,
{
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        map(
            delimited(
                pair(char('{'), opt(separator)),
                many0(maybe_sep_terminated(parse)),
                pair(opt(separator), char('}')),
            ),
            |planes| Brush { planes },
        )(input)
    }
}

pub(crate) fn comment<'i, E>(input: Input<'i>) -> ParseResult<Input<'i>, E>
where
    E: ParseError<Input<'i>>,
{
    preceded(tag("//"), not_line_ending)(input)
}

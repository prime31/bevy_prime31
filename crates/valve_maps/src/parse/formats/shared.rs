use bevy::prelude::Vec3;

use super::valve::TextureAlignment;

use {
    std::{
        collections::HashMap,
        ops::{Deref, DerefMut}
    },
    crate::parse::{
        common::{fields, parse, quoted_string, many_fixed},
        core::{
            Parse,
            Input,
            ParseResult,
            nom::{
                branch::alt,
                number::float,
                error::ParseError,
                bytes::{tag, take_till},
                multi::{many0, fold_many0},
                combinator::{map, opt, iterator, recognize},
                sequence::{pair, delimited, terminated, preceded},
                character::{char, multispace1, line_ending, not_line_ending}
            }
        }
    }
};

pub(crate) fn separator<'i, E>(input: Input<'i>) -> ParseResult<Input<'i>, E>
where E: ParseError<Input<'i>> + Clone {
    recognize(
        |input| {
            let mut iter = iterator(
                input,
                alt((
                    multispace1,
                    terminated(comment, line_ending),
                ))
            );
            iter.for_each(drop);
            iter.finish()
        }
    )(input)
}

pub(crate) fn sep_terminated<'i, F, O, E>(parsed: F) -> impl Fn(Input<'i>) -> ParseResult<O, E>
where
    F: Fn(Input<'i>) -> ParseResult<O, E>,
    E: ParseError<Input<'i>> + Clone
{
    terminated(parsed, separator)
}

pub(crate) fn maybe_sep_terminated<'i, F, O, E>(parsed: F) -> impl Fn(Input<'i>) -> ParseResult<O, E>
    where
        F: Fn(Input<'i>) -> ParseResult<O, E>,
        E: ParseError<Input<'i>> + Clone
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

impl <'i, E> Parse<'i, E> for Fields
where E: ParseError<Input<'i>> + Clone {
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        map(
            fold_many0(
                maybe_sep_terminated(
                    pair(
                        maybe_sep_terminated(quoted_string),
                        quoted_string
                    )
                ),
                HashMap::new(),
                |mut map, (k, v)| {
                    map.insert(k.into(), v.into());
                    map
                }
            ),
            Fields
        )(input)
    }
}

/// Representation of a map entity with [key/value pairs](Fields) and a list
/// of [Brush](Brush)es, which may be empty if the entity in question is a
/// point entity, like a light.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MapEntity {
    pub fields: Fields,
    pub brushes: Vec<Brush>
}

impl MapEntity {
    pub fn get_property(&self, name: &str) -> Option<&str> {
        if let Some(s) = self.fields.get(&String::from(name)) {
            return Some(&s[..]);
        }
        None
    }
}

impl <'i, E> Parse<'i, E> for MapEntity
where
    E: ParseError<Input<'i>> + Clone
{
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        delimited(
            pair(char('{'), opt(separator)),
            fields!(MapEntity:
                fields = maybe_sep_terminated(parse),
                brushes = many0(maybe_sep_terminated(parse))
            ),
            char('}')
        )(input)
    }
}

/// Representation of a plane with three points describing a
/// half-space and a texture. In a map file, it usually looks
/// something like this with the valve format:
/// ```plain
/// ( 816 -796 356 ) ( 816 -804 356 ) ( 808 -804 356 ) stone1_3 [ 0 -1 0 -20 ] [ 1 0 0 16 ] -0 1 1
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Plane {
    pub points: [Vector3; 3],
    pub texture: Texture
}

impl Plane {
    pub fn normal(&self) -> Vec3 {
        let v0v1 = self.v1() - self.v0();
        let v0v2 = self.v2() - self.v0();
        v0v2.cross(v0v1).normalize()
    }

    pub fn dist(&self) -> f32 {
        let n = self.normal();
        n.dot(self.points[0].as_vec3())
    }

    pub fn v0(&self) -> Vec3 {
        self.points[0].as_vec3()
    }

    pub fn v1(&self) -> Vec3 {
        self.points[1].as_vec3()
    }

    pub fn v2(&self) -> Vec3 {
        self.points[2].as_vec3()
    }
}

impl <'i, E> Parse<'i, E> for Plane
where
    E: ParseError<Input<'i>> + Clone
{
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        fields!(Plane:
            points = many_fixed(
                maybe_sep_terminated(
                    delimited(
                        pair(char('('), opt(separator)),
                        parse,
                        pair(opt(separator), char(')'))
                    ),
                )
            ),
            texture = parse
        )(input)
    }
}

/// A simple three-dimensional vector using `f32`s.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3 {
    pub fn as_vec3(self) -> Vec3 {
        return Vec3::new(self.x, self.y, self.z);
    }
}

impl Into<Vector3> for Vec3 {
    fn into(self) -> Vector3 {
        return Vector3 {x: self.x, y: self.y, z: self.z};
    }
}

impl <'i, E> Parse<'i, E> for Vector3
where E: ParseError<Input<'i>> + Clone {
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        fields!(Vector3:
            x = sep_terminated(float),
            y = sep_terminated(float),
            z = float
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

impl <'i, E> Parse<'i, E> for Texture
where
    E: ParseError<Input<'i>> + Clone
{
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        fields!(Texture:
            name = map(
                sep_terminated(
                    take_till(char::is_whitespace)
                ),
                String::from
            ),
            alignment = parse
        )(input)
    }
}

/// Representation of a map brush, consisting of a
/// list of [Plane](Plane)s.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Brush {
    pub planes: Vec<Plane>
}

impl <'i, E> Parse<'i, E> for Brush
where
    E: ParseError<Input<'i>> + Clone
{
    fn parse(input: Input<'i>) -> ParseResult<Self, E> {
        map(
            delimited(
                pair(char('{'), opt(separator)),
                many0(maybe_sep_terminated(parse)),
                pair(opt(separator), char('}'))
            ),
            |planes| Brush { planes }
        )(input)
    }
}

pub(crate) fn comment<'i, E>(input: Input<'i>) -> ParseResult<Input<'i>, E>
where E: ParseError<Input<'i>> {
    preceded(tag("//"), not_line_ending)(input)
}


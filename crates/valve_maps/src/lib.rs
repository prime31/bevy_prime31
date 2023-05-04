pub mod bevy;
pub mod convert;
pub mod generate;
pub mod parse;

use parse::core::{
    nom::{self, combinator::all_consuming},
    Error, Input, Parse,
};

pub use parse::{formats::Map, *};

/// Convenience function to parse a map from a string. Assumes that the input
/// consists entirely of the map and returns the [Error](parse::core::Error)
/// type provided by this crate. If you wish to integrate with other `nom` parsers,
/// using the [Parse](parse::core::Parse) implementation on [Map](parse::formats::Map)
/// is recommended.
pub fn parse<'i>(input: Input<'i>) -> Result<Map, nom::Err<Error<'i>>> {
    all_consuming(Map::parse)(input).map(|(_rest, map)| map)
}

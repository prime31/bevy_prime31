pub mod shared;
pub mod valve;

use crate::{
    generate::{entity_build, Geometry, TextureInfo},
    parse::{
        common::parse,
        core::{
            nom::{
                combinator::{map, opt},
                multi::many1,
                sequence::preceded,
            },
            Input, Parse, ParseResult,
        },
        formats::shared::{maybe_sep_terminated, separator},
    },
};

pub use valve::Valve;

use self::shared::MapEntity;

/// Representation of a Quake/Half-Life 1 map as a `Vec` of entities
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Map {
    pub entities: Vec<MapEntity>,
}

impl Map {
    pub fn get_texture_names(&self) -> Vec<&String> {
        let mut textures = Vec::new();

        for ent in self.entities.iter() {
            for brushes in ent.brushes.iter() {
                for planes in brushes.planes.iter() {
                    if planes.texture.name != "__TB_empty" {
                        textures.push(&planes.texture.name);
                    }
                }
            }
        }

        textures.sort();
        textures.dedup();
        textures
    }

    pub fn build_entity_geometry(&self, textures: &TextureInfo) -> Vec<Geometry> {
        // Build geometry
        self.entities
            .iter()
            .map(|entity| entity_build(&textures, entity))
            .collect()
    }
}

impl<'i> Parse<'i> for Map {
    fn parse(input: Input<'i>) -> ParseResult<Self> {
        preceded(
            opt(separator),
            map(many1(maybe_sep_terminated(parse)), |entities| Map { entities }),
        )(input)
    }
}

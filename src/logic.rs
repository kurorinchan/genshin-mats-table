use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;

mod asset;

const RESOURCES_FILE: &str = "resources.json";
const LEVEL_UP_MAT: &str = "Talent Level-Up Material";

#[derive(
    EnumIter, Debug, AsRefStr, EnumString, Clone, Serialize, Deserialize, PartialEq, Eq, Hash,
)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(EnumIter, Debug, AsRefStr, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum TalentLevelUpMaterialType {
    // Mondstadt.
    Freedom,
    Resistance,
    Ballad,

    // Liyue.
    Prosperity,
    Diligence,
    Gold,

    // Inazuma.
    Transience,
    Elegance,
    Light,

    // Sumeru.
    Admonition,
    Ingenuity,
    Praxis,

    // Fontaine.
    Equity,
    Justice,
    Order,

    // Natlan.
    Contention,
    Kindling,
    Conflict,
}

impl TalentLevelUpMaterialType {
    // Gethe the type name from the full name of the material.
    fn from_full_name(fullname: &str) -> Option<Self> {
        Self::iter().find(|mat_type| fullname.contains(mat_type.as_ref()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalentLevelUpMaterial {
    pub name: String,
    pub mat_type: TalentLevelUpMaterialType,
    pub days: Vec<DayOfWeek>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub talent_materials: Vec<TalentLevelUpMaterial>,
}

mod serde_data {
    use serde::Deserialize;
    #[derive(Deserialize, Debug)]
    pub struct Material {
        #[serde(rename = "type")]
        pub mat_type: String,
        pub name: String,

        // This field is not currently used.
        #[serde(skip)]
        #[allow(dead_code)]
        rarity: i32,
    }

    #[derive(Deserialize, Debug)]
    pub struct CharacterTalentMat {
        #[serde(rename = "name")]
        pub character_name: String,
        pub materials: Vec<Material>,
        // This does not handle traveler mats e.g. "Traveler (Dendro)". Consider handling them.
    }

    #[derive(Deserialize, Debug)]
    #[serde(tag = "object_type", rename_all = "kebab-case")]

    pub enum CharacterTalentLevelUpEntry {
        #[serde(rename = "talent-level-up-materials")]
        TalentLevelUpMaterials(CharacterTalentMat),
        #[serde(untagged)]
        Other {},
    }

    #[derive(Deserialize, Debug)]
    pub struct CharacterTalentLevelUpRoot {
        pub data: Vec<CharacterTalentLevelUpEntry>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourcesRoot {
        pub data: Vec<ResourceEntry>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourceEntry {
        #[serde(rename = "object_type")]
        pub object_type: String,
        pub name: String,
        #[serde(rename = "type")]
        pub type_field: String,
        pub rarity: Option<i64>,
        pub thumbnail: String,
        pub link: String,
        pub category: Option<String>,
        pub points: Option<i64>,
        pub group: Option<String>,
        pub region: Option<String>,
        #[serde(default)]
        pub source: Vec<String>,
        pub domain_level: Option<i64>,
        pub days: Option<Vec<String>>,
        #[serde(rename = "group_type")]
        pub group_type: Option<String>,
    }
}

impl Character {
    pub fn new(character_name: &str, materials: &[TalentLevelUpMaterial]) -> Self {
        Self {
            name: character_name.to_owned(),
            talent_materials: materials.to_vec(),
        }
    }

    pub fn uses_same_material(&self, other: &Self) -> bool {
        if self.talent_materials.is_empty() || other.talent_materials.is_empty() {
            return false;
        }

        self.talent_materials[0].mat_type == other.talent_materials[0].mat_type
    }
}

// TODO: Might be better to have this in a JSON and read it.

pub fn day_to_mat_type() -> HashMap<DayOfWeek, Vec<TalentLevelUpMaterialType>> {
    type MatType = TalentLevelUpMaterialType;
    let mapping = [
        (
            DayOfWeek::Monday,
            vec![
                MatType::Freedom,
                MatType::Prosperity,
                MatType::Transience,
                MatType::Admonition,
                MatType::Equity,
                MatType::Contention,
            ],
        ),
        (
            DayOfWeek::Tuesday,
            vec![
                MatType::Resistance,
                MatType::Diligence,
                MatType::Elegance,
                MatType::Ingenuity,
                MatType::Justice,
                MatType::Kindling,
            ],
        ),
        (
            DayOfWeek::Wednesday,
            vec![
                MatType::Ballad,
                MatType::Gold,
                MatType::Light,
                MatType::Praxis,
                MatType::Order,
                MatType::Conflict,
            ],
        ),
    ];

    let mut map = HashMap::new();
    for (day, mat_types) in mapping {
        map.insert(day, mat_types.clone());
    }

    map.insert(
        DayOfWeek::Thursday,
        map.get(&DayOfWeek::Monday).unwrap().clone(),
    );
    map.insert(
        DayOfWeek::Friday,
        map.get(&DayOfWeek::Tuesday).unwrap().clone(),
    );
    map.insert(
        DayOfWeek::Saturday,
        map.get(&DayOfWeek::Wednesday).unwrap().clone(),
    );

    map.insert(DayOfWeek::Sunday, MatType::iter().collect());
    map
}

pub fn group_by_material(
    characters: Vec<Character>,
) -> HashMap<TalentLevelUpMaterialType, Vec<Character>> {
    let mut map = HashMap::new();
    for character in characters {
        if character.talent_materials.is_empty() {
            continue;
        }
        let mat_type = character.talent_materials[0].mat_type.clone();
        map.entry(mat_type).or_insert_with(Vec::new).push(character);
    }
    map
}

pub fn read_character_mats() -> Result<Vec<Character>> {
    let resources = read_resources()?;
    let f =
        asset::Asset::get("character-talent-level-up.json").context("failed to find json file")?;
    let character_talents: serde_data::CharacterTalentLevelUpRoot =
        serde_json::from_slice(&f.data)?;

    let talent_materials: Vec<&serde_data::CharacterTalentMat> = character_talents
        .data
        .iter()
        .filter_map(|data| match data {
            serde_data::CharacterTalentLevelUpEntry::TalentLevelUpMaterials(talent_mat) => {
                Some(talent_mat)
            }
            serde_data::CharacterTalentLevelUpEntry::Other {} => None,
        })
        .collect();

    // TODO: This is too long. Make it a function.
    let characters = talent_materials
        .iter()
        .map(|talent_mat| {
            // Now create TalentLevelUpMaterial from |materials| by joining the data from resources.
            // TODO: Make this a function or somehow give it some meaningful name.
            let talent_mats: Vec<TalentLevelUpMaterial> = talent_mat
                .materials
                .iter()
                .filter_map(|mat| {
                    if mat.mat_type != LEVEL_UP_MAT {
                        return None;
                    }

                    // Now find the material name in resources to get all the day of week.
                    let days_of_week = resources.iter().find_map(|resource| {
                        if mat.name == resource.name {
                            resource.days.as_ref()
                        } else {
                            None
                        }
                    })?;

                    let days_of_week: Vec<DayOfWeek> = days_of_week
                        .iter()
                        .map(|day_of_week| DayOfWeek::from_str(day_of_week).unwrap())
                        .collect();

                    let mat_type = TalentLevelUpMaterialType::from_full_name(&mat.name)?;

                    Some(TalentLevelUpMaterial {
                        name: mat.name.clone(),
                        mat_type,
                        days: days_of_week,
                    })
                })
                .collect();
            Character::new(&talent_mat.character_name, &talent_mats)
        })
        .collect();
    Ok(characters)
}

fn read_resources() -> Result<Vec<serde_data::ResourceEntry>> {
    let f = asset::Asset::get(RESOURCES_FILE).context("failed to find json file")?;
    let root: serde_data::ResourcesRoot = serde_json::from_slice(&f.data)?;
    let resources = root.data;

    // Preserve "Talent Level-Up Material" only.
    let resources: Vec<serde_data::ResourceEntry> = resources
        .into_iter()
        .filter(|resource| resource.object_type == "item" && resource.type_field == LEVEL_UP_MAT)
        .collect();

    Ok(resources)
}

#[cfg(test)]
mod tests {
    use super::*;
    use more_asserts::*;

    #[test]
    fn test_read_resources() -> Result<()> {
        let resources = read_resources()?;
        for resource in resources {
            assert_eq!(resource.type_field, "Talent Level-Up Material");
        }

        Ok(())
    }

    // Verify that the data loads and has at least one character.
    // Cannot check too much, as it requires checking implementation details.
    #[test]
    fn test_read_character_mats() -> Result<()> {
        let characters = read_character_mats().unwrap();
        assert_ge!(characters.len(), 1);
        let furina = characters
            .iter()
            .find(|character| character.name == "Furina")
            .unwrap();
        assert_ge!(furina.talent_materials.len(), 1);

        let material = &furina.talent_materials[0];
        assert_eq!(material.mat_type, TalentLevelUpMaterialType::Justice);
        Ok(())
    }

    #[test]
    fn test_group_by_material() -> Result<()> {
        // TODO: Prefer self contained tests. Don't read from data set here. Instead create a
        // toy dataset in this test.
        let characters = read_character_mats().unwrap();
        let group = group_by_material(&characters);
        assert_ge!(group.len(), 1);

        // Pick one material type that should be used by more than one character
        // and verify that all the characters use the same material type.
        let mat_type = TalentLevelUpMaterialType::Justice;
        let characters = group.get(&mat_type).unwrap();
        assert_gt!(characters.len(), 1);

        let character = characters[0];
        for c in characters {
            assert!(c.uses_same_material(character));
        }

        Ok(())
    }
}

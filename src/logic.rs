use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use chrono::Datelike;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;
use serde_data::WordEntry;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
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
    pub thumbnail: String,
}

mod serde_data {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

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

    // For reading resources.json.
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

    // For reading characters.json.
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CharactersRoot {
        #[serde(skip)]
        pub version: String,
        pub data: Vec<CharacterEntry>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CharacterEntry {
        #[serde(rename = "object_type")]
        pub object_type: String,
        pub name: String,
        #[serde(rename = "type")]
        pub type_field: String,
        pub rarity: i64,
        pub element: String,
        pub weapon: String,
        // This is Value because it could be a String or an array of Strings.
        #[serde(skip)]
        pub region: Value,
        pub talents: Vec<Talent>,
        pub constellations: Vec<Constellation>,
        pub thumbnail: String,
        pub link: String,
        #[serde(rename = "display_name")]
        pub display_name: Option<String>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Talent {
        #[serde(rename = "object_type")]
        pub object_type: String,
        pub name: String,
        #[serde(rename = "type")]
        pub type_field: String,
        pub thumbnail: String,
        pub link: String,
        pub constellation: Option<i64>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Constellation {
        #[serde(rename = "object_type")]
        pub object_type: String,
        pub name: String,
        pub level: i64,
        pub thumbnail: String,
        pub link: String,
    }

    // For words.json.
    // For now, it is only used to go from English to Japanese.
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct WordEntry {
        pub en: String,
        // Some entries do not have it, this must be Option.
        pub ja: Option<String>,
    }
}

impl Character {
    pub fn new(
        name: String,
        talent_materials: Vec<TalentLevelUpMaterial>,
        thumbnail: String,
    ) -> Self {
        Self {
            name,
            talent_materials,
            thumbnail,
        }
    }
}

// TODO: Might be better to have this in a JSON and read it.

pub fn day_to_mat_type() -> HashMap<DayOfWeek, Vec<TalentLevelUpMaterialType>> {
    type MatType = TalentLevelUpMaterialType;
    let mapping = [
        (
            DayOfWeek::Monday,
            vec![
                MatType::Contention,
                MatType::Equity,
                MatType::Admonition,
                MatType::Transience,
                MatType::Prosperity,
                MatType::Freedom,
            ],
        ),
        (
            DayOfWeek::Tuesday,
            vec![
                MatType::Kindling,
                MatType::Justice,
                MatType::Ingenuity,
                MatType::Elegance,
                MatType::Diligence,
                MatType::Resistance,
            ],
        ),
        (
            DayOfWeek::Wednesday,
            vec![
                MatType::Conflict,
                MatType::Order,
                MatType::Praxis,
                MatType::Light,
                MatType::Gold,
                MatType::Ballad,
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
        let mat_type = character.talent_materials[0].mat_type;
        map.entry(mat_type).or_insert_with(Vec::new).push(character);
    }
    map
}

// Extracts string inside the quotation symbols: 「」
fn extract_str_in_ja_quotes(ja: &str) -> Option<&str> {
    let start = ja.find("「")?;
    let end = ja.find("」")?;
    if start > end {
        return None;
    }

    let s = &ja[start..end];
    // s contains 「. So remove it. Note that its not just start+1 because the character is non
    // ascii.
    s.chars().next().map(|c| &s[c.len_utf8()..])
}

// Returns the display name for mat_type.
pub fn mat_type_to_name(mat_type: TalentLevelUpMaterialType) -> Result<String> {
    let words = read_words()?;
    let teaching = format!("Teachings of {}", mat_type.as_ref());

    let entry = words
        .get(&teaching)
        .with_context(|| format!("failed to find {}", &teaching))?;

    let ja = entry
        .ja
        .as_ref()
        .with_context(|| format!("No japanese translationf for {}", &teaching))?;

    let contains_ja_quotes = ja.contains("「") && ja.contains("」");
    if !contains_ja_quotes {
        bail!("failed to find 「」in {}", &teaching);
    }

    let ja = extract_str_in_ja_quotes(ja).context("failed to extract string")?;
    Ok(ja.to_owned())
}

// This converts the filename from png to webp. Does not actually convert a file to webp file.
fn png2webp_extension(filename: &str) -> String {
    let out = Path::new(filename).with_extension("webp");
    out.to_string_lossy().to_string()
}

// Now create TalentLevelUpMaterial from |materials| by joining the data from resources.
fn convert_to_talent_mats(
    talent_mat: &serde_data::CharacterTalentMat,
    resources: &HashMap<String, serde_data::ResourceEntry>,
) -> Vec<TalentLevelUpMaterial> {
    let talent_mats: Vec<TalentLevelUpMaterial> = talent_mat
        .materials
        .iter()
        .filter_map(|mat| {
            if mat.mat_type != LEVEL_UP_MAT {
                return None;
            }

            // Now find the material name in resources to get all the day of week.
            let days_of_week = resources
                .get(&mat.name)
                .map(|resource| resource.days.as_ref())?;

            let days_of_week: Vec<DayOfWeek> = days_of_week?
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
    talent_mats
}

pub fn read_character_mats() -> Result<Vec<Character>> {
    let en_to_jp = read_en_to_jp()?;
    let character_entries = read_characters()?;
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

    let characters = talent_materials
        .iter()
        .map(|talent_mat| {
            let thumbnail = character_entries
                .get(&talent_mat.character_name)
                .map(|entry| png2webp_extension(&entry.thumbnail));

            // Best to keep the English name displayed if it fails to find the translation.
            let character_name = en_to_jp
                .get(&talent_mat.character_name)
                .unwrap_or(&talent_mat.character_name);
            Character::new(
                character_name.to_owned(),
                convert_to_talent_mats(talent_mat, &resources),
                thumbnail.unwrap(),
            )
        })
        .collect();
    Ok(characters)
}

fn read_resources() -> Result<HashMap<String, serde_data::ResourceEntry>> {
    let f = asset::Asset::get(RESOURCES_FILE).context("failed to find json file")?;
    let root: serde_data::ResourcesRoot = serde_json::from_slice(&f.data)?;
    let resources = root.data;

    // Preserve "Talent Level-Up Material" only.
    let resources: HashMap<String, serde_data::ResourceEntry> = resources
        .iter()
        .filter(|resource| resource.object_type == "item" && resource.type_field == LEVEL_UP_MAT)
        .map(|resource| (resource.name.clone(), resource.clone()))
        .collect();

    Ok(resources)
}

const CHARACTERS_FILE: &str = "characters.json";

fn read_characters() -> Result<HashMap<String, serde_data::CharacterEntry>> {
    let f = asset::Asset::get(CHARACTERS_FILE).context("failed to find json file")?;
    let root: serde_data::CharactersRoot = serde_json::from_slice(&f.data)?;
    let entries = root.data;

    let entries: HashMap<String, serde_data::CharacterEntry> = entries
        .iter()
        .map(|entry| (entry.name.clone(), entry.clone()))
        .collect();

    Ok(entries)
}

fn read_words() -> Result<HashMap<String, WordEntry>> {
    let f = asset::Asset::get("words.json").context("failed to find json file")?;
    let root: Vec<serde_data::WordEntry> = serde_json::from_slice(&f.data)?;
    let mut map = HashMap::new();
    for entry in root {
        map.insert(entry.en.clone(), entry);
    }

    Ok(map)
}

fn read_en_to_jp() -> Result<HashMap<String, String>> {
    let en = asset::Asset::get("en_itemid.json").context("failed to find json file")?;
    let jp = asset::Asset::get("jp_itemid.json").context("failed to find json file")?;

    let en: HashMap<String, u32> = serde_json::from_slice(&en.data)?;
    let jp: HashMap<String, u32> = serde_json::from_slice(&jp.data)?;

    let mut en_to_jp = HashMap::new();
    for (en_key, en_itemid) in &en {
        for (jp_key, jp_itemid) in &jp {
            if en_itemid == jp_itemid {
                en_to_jp.insert(en_key.to_owned(), jp_key.to_owned());
            }
        }
    }

    Ok(en_to_jp)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantDay {
    pub day_of_week: DayOfWeek,
    pub display_name: String,
    pub is_today: bool,
}

pub fn relevant_days() -> Vec<RelevantDay> {
    let weekday = Local::now().weekday();
    let weekday = match weekday {
        chrono::Weekday::Mon => DayOfWeek::Monday,
        chrono::Weekday::Tue => DayOfWeek::Tuesday,
        chrono::Weekday::Wed => DayOfWeek::Wednesday,
        chrono::Weekday::Thu => DayOfWeek::Thursday,
        chrono::Weekday::Fri => DayOfWeek::Friday,
        chrono::Weekday::Sat => DayOfWeek::Saturday,
        chrono::Weekday::Sun => DayOfWeek::Sunday,
    };
    vec![
        RelevantDay {
            day_of_week: DayOfWeek::Monday,
            display_name: "月/木".to_string(),
            is_today: weekday == DayOfWeek::Monday
                || weekday == DayOfWeek::Thursday
                || weekday == DayOfWeek::Sunday,
        },
        RelevantDay {
            day_of_week: DayOfWeek::Tuesday,
            display_name: "火/金".to_string(),
            is_today: weekday == DayOfWeek::Tuesday
                || weekday == DayOfWeek::Friday
                || weekday == DayOfWeek::Sunday,
        },
        RelevantDay {
            day_of_week: DayOfWeek::Wednesday,
            display_name: "水/土".to_string(),
            is_today: weekday == DayOfWeek::Wednesday
                || weekday == DayOfWeek::Saturday
                || weekday == DayOfWeek::Sunday,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Ok;
    use more_asserts::*;

    #[test]
    fn test_read_resources() -> Result<()> {
        let resources = read_resources()?;
        for resource in resources.values() {
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
            .find(|character| character.name == "フリーナ")
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
        let group = group_by_material(characters);
        assert_ge!(group.len(), 1);

        // Pick one material type that should be used by more than one character
        // and verify that all the characters use the same material type.
        let mat_type = TalentLevelUpMaterialType::Justice;
        let characters = group.get(&mat_type).unwrap();
        assert_gt!(characters.len(), 1);

        Ok(())
    }

    #[test]
    fn test_read_words() -> Result<()> {
        let words = read_words()?;
        assert_ge!(words.len(), 1);
        Ok(())
    }

    #[test]
    fn test_read_en_to_jp() {
        let en_to_jp = read_en_to_jp().unwrap();
        assert_ge!(en_to_jp.len(), 1);
        assert_eq!(en_to_jp.get("Diluc").unwrap(), "ディルック");
    }

    #[test]
    fn test_mat_type_to_name() -> Result<()> {
        let justice = mat_type_to_name(TalentLevelUpMaterialType::Justice)?;
        assert_eq!("正義", justice);

        Ok(())
    }
}

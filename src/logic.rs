// Things to edit (in this file) when adding new characters
// - Add new materials to TalentLevelUpMaterialType enum.

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
use std::str::FromStr;
use std::vec;
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

#[derive(
    EnumIter,
    Debug,
    AsRefStr,
    Clone,
    Hash,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Copy,
    PartialOrd,
    Ord,
)]
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
    // Get the type name from the full name of the material.
    // e.g. "Teaching of Justice" -> Justice
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
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

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

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct TalentMaterial {
        pub name: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct CharacterEntry {
        pub name: String,
        pub names: HashMap<String, String>,
        pub thumbnail: String,
        pub talent_materials: Vec<TalentMaterial>,
    }

    // For reading characters.json.
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

pub fn day_to_mat_type() -> HashMap<DayOfWeek, Vec<TalentLevelUpMaterialType>> {
    type MatType = TalentLevelUpMaterialType;
    let mut map = HashMap::new();
    // TODO: Don't unwrap here in case of failure.
    let resources = read_resources().unwrap();
    for (name, resource) in &resources {
        if !name.contains("Teachings of ") {
            continue;
        }
        let days = resource.days.as_ref().unwrap();
        for day in days {
            let day = DayOfWeek::from_str(day).unwrap();
            let mat_type = MatType::from_full_name(name).unwrap();
            map.entry(day).or_insert_with(Vec::new).push(mat_type);
        }
    }

    // Relying on enum to be in oldest to newest.
    for values in map.values_mut() {
        values.sort();
        values.reverse();
    }
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

fn read_better_characters() -> Result<Vec<serde_data::CharacterEntry>> {
    const CHARACTERS_FILE: &str = "clean-characters.json";
    let f = asset::Asset::get(CHARACTERS_FILE)
        .with_context(|| format!("failed to find json {}", CHARACTERS_FILE))?;
    let root: Vec<serde_data::CharacterEntry> = serde_json::from_slice(&f.data)?;
    Ok(root)
}

fn material_name_to_day_of_week(
    name: &str,
    resources: &HashMap<String, serde_data::ResourceEntry>,
) -> Option<Vec<DayOfWeek>> {
    // Now find the material name in resources to get all the day of week.
    let days_of_week = resources.get(name).map(|resource| resource.days.as_ref())?;

    let days_of_week: Vec<DayOfWeek> = days_of_week?
        .iter()
        .map(|day_of_week| DayOfWeek::from_str(day_of_week).unwrap())
        .collect();

    Some(days_of_week)
}

pub fn read_character_mats() -> Result<Vec<Character>> {
    let better_characters = read_better_characters()?;
    let resources = read_resources()?;

    let characters = better_characters.iter().filter_map(|better_character| {
        const CHARACTER_NAME_LANGUAGE: &str = "JP";
        if better_character.name == "Traveler" {
            return None;
        }
        let name = better_character.names[CHARACTER_NAME_LANGUAGE].clone();
        let thumbnail = format!(
            "Character_{}_Thumb.webp",
            &better_character.name.replace(" ", "_")
        );

        let talent_materials = better_character
            .talent_materials
            .iter()
            .map(|talent_material| {
                let mat_type =
                    TalentLevelUpMaterialType::from_full_name(&talent_material.name).unwrap();
                let days = material_name_to_day_of_week(&talent_material.name, &resources).unwrap();
                TalentLevelUpMaterial {
                    name: talent_material.name.clone(),
                    mat_type,
                    days,
                }
            })
            .collect();
        Some(Character::new(name, talent_materials, thumbnail))
    });

    Ok(characters.collect())
}

// Read resources from json file.
// This is mainly used to get the day of week for materials, so that this source code does not have
// to be edited much to support new materials.
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

fn read_words() -> Result<HashMap<String, WordEntry>> {
    let f = asset::Asset::get("words.json").context("failed to find json file")?;
    let root: Vec<serde_data::WordEntry> = serde_json::from_slice(&f.data)?;
    let mut map = HashMap::new();
    for entry in root {
        map.insert(entry.en.clone(), entry);
    }

    Ok(map)
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
    fn test_mat_type_to_name() -> Result<()> {
        let justice = mat_type_to_name(TalentLevelUpMaterialType::Justice)?;
        assert_eq!("正義", justice);

        Ok(())
    }

    #[test]
    fn test_day_to_mat_type() -> Result<()> {
        let day_to_mat = day_to_mat_type();
        assert_ge!(day_to_mat.len(), 1);

        // Make sure there are entries for all day of the week.
        assert_ge!(day_to_mat.get(&DayOfWeek::Monday).unwrap().len(), 1);
        assert_ge!(day_to_mat.get(&DayOfWeek::Tuesday).unwrap().len(), 1);
        assert_ge!(day_to_mat.get(&DayOfWeek::Wednesday).unwrap().len(), 1);
        assert_ge!(day_to_mat.get(&DayOfWeek::Thursday).unwrap().len(), 1);
        assert_ge!(day_to_mat.get(&DayOfWeek::Friday).unwrap().len(), 1);
        assert_ge!(day_to_mat.get(&DayOfWeek::Saturday).unwrap().len(), 1);
        assert_ge!(day_to_mat.get(&DayOfWeek::Sunday).unwrap().len(), 1);

        let materials = day_to_mat.get(&DayOfWeek::Monday).unwrap();

        let found = materials
            .iter()
            .find(|mat_type| *mat_type == &TalentLevelUpMaterialType::Contention);

        assert!(found.is_some());
        Ok(())
    }
}

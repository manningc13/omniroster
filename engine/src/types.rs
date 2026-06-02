use serde::{Deserialize, Serialize};

/// A dice roll threshold like "3+" stored as the integer 3.
pub type RollThreshold = u8;

/// Represents a stat that is either a fixed number or a roll threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StatValue {
    Number(i32),
    Roll(String), // "2+", "3+", etc — kept as string for display, parsed when needed
    Text(String), // "6\"", "D6+2", etc
}

impl StatValue {
    /// If this is a roll threshold, return the numeric part.
    pub fn as_roll(&self) -> Option<u8> {
        match self {
            StatValue::Roll(s) => s.trim_end_matches('+').parse().ok(),
            _ => None,
        }
    }

    /// Improve a roll threshold by 1 step (3+ → 2+), capped at 2+.
    pub fn improve_roll(&self) -> StatValue {
        match self.as_roll() {
            Some(n) if n > 2 => StatValue::Roll(format!("{}+", n - 1)),
            _ => self.clone(),
        }
    }

    /// Worsen a roll threshold by 1 step (3+ → 4+), capped at 6+.
    pub fn worsen_roll(&self) -> StatValue {
        match self.as_roll() {
            Some(n) if n < 6 => StatValue::Roll(format!("{}+", n + 1)),
            _ => self.clone(),
        }
    }
}

/// Base unit profile as loaded from JSON data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitProfile {
    pub id: String,
    pub name: String,
    pub faction: String,
    pub keywords: Vec<String>,
    pub base_stats: UnitStats,
    pub weapons: Vec<Weapon>,
    pub abilities: Vec<Ability>,
    pub points: u32,
}

/// The core stat block for a unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitStats {
    pub movement: StatValue,
    pub toughness: StatValue,
    pub save: StatValue,
    pub wounds: StatValue,
    pub leadership: StatValue,
    pub oc: StatValue,
    pub bs: StatValue,
    pub ws: StatValue,
}

/// A weapon profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    pub weapon_type: WeaponType,
    pub range: Option<String>, // None for melee
    pub attacks: StatValue,
    pub skill: StatValue,      // BS for ranged, WS for melee
    pub strength: StatValue,
    pub ap: i32,
    pub damage: StatValue,
    pub keywords: Vec<String>, // "Rapid Fire 1", "Blast", etc
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeaponType {
    Ranged,
    Melee,
}

/// A named ability on a unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub name: String,
    pub description: String,
}

/// Active modifiers applied to the army state this turn.
/// Each variant is a distinct game mechanic that can affect resolved stats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Modifier {
    /// Adeptus Mechanicus Doctrina Imperatives
    DoctrinaImperative { doctrina: Doctrina },
    /// A stratagem applied to a specific unit
    Stratagem { stratagem_id: String, unit_id: String },
    /// Damage degradation threshold crossed (vehicles)
    DamageDegradation { unit_id: String, bracket: u8 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Doctrina {
    Protector,   // BS +1 for Skitarii
    Conqueror,   // WS +1 for Skitarii
    EliminationVolley, // Rapid Fire weapons double shots
    DataPsalmEnrichment, // Wound on 6+ regardless of Toughness
}

/// The fully resolved stat card for a unit after all modifiers are applied.
/// This is what the frontend renders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedUnit {
    pub id: String,
    pub name: String,
    pub faction: String,
    pub keywords: Vec<String>,
    pub stats: UnitStats,
    pub stat_changes: StatChanges,
    pub weapons: Vec<ResolvedWeapon>,
    pub abilities: Vec<Ability>,
    pub active_ability_notes: Vec<String>,
}

/// Tracks which stats changed from base, for UI highlighting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatChanges {
    pub bs_changed: bool,
    pub ws_changed: bool,
    pub movement_changed: bool,
    pub toughness_changed: bool,
}

/// A weapon after modifier resolution, with change flags for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedWeapon {
    pub name: String,
    pub weapon_type: WeaponType,
    pub range: Option<String>,
    pub attacks: StatValue,
    pub skill: StatValue,
    pub strength: StatValue,
    pub ap: i32,
    pub damage: StatValue,
    pub keywords: Vec<String>,
    pub skill_changed: bool,
    pub attacks_changed: bool,
}

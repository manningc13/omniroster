use crate::types::*;

/// Resolves a unit's stats by applying all active modifiers in order.
/// This is the core of the rules engine — all stat computation happens here.
pub fn resolve_unit(profile: &UnitProfile, modifiers: &[Modifier]) -> ResolvedUnit {
    let mut stats = profile.base_stats.clone();
    let mut stat_changes = StatChanges::default();
    let mut weapon_overrides: Vec<WeaponOverride> = profile
        .weapons
        .iter()
        .map(|w| WeaponOverride::from_weapon(w))
        .collect();
    let mut ability_notes: Vec<String> = Vec::new();

    for modifier in modifiers {
        apply_modifier(
            modifier,
            profile,
            &mut stats,
            &mut stat_changes,
            &mut weapon_overrides,
            &mut ability_notes,
        );
    }

    let resolved_weapons = profile
        .weapons
        .iter()
        .zip(weapon_overrides.iter())
        .map(|(w, ov)| ResolvedWeapon {
            name: w.name.clone(),
            weapon_type: w.weapon_type.clone(),
            range: w.range.clone(),
            attacks: ov.attacks.clone(),
            skill: ov.skill.clone(),
            strength: w.strength.clone(),
            ap: w.ap,
            damage: w.damage.clone(),
            keywords: ov.keywords.clone(),
            skill_changed: ov.skill_changed,
            attacks_changed: ov.attacks_changed,
        })
        .collect();

    ResolvedUnit {
        id: profile.id.clone(),
        name: profile.name.clone(),
        faction: profile.faction.clone(),
        keywords: profile.keywords.clone(),
        stats,
        stat_changes,
        weapons: resolved_weapons,
        abilities: profile.abilities.clone(),
        active_ability_notes: ability_notes,
    }
}

/// Mutable weapon state during resolution.
struct WeaponOverride {
    attacks: StatValue,
    skill: StatValue,
    keywords: Vec<String>,
    skill_changed: bool,
    attacks_changed: bool,
}

impl WeaponOverride {
    fn from_weapon(w: &Weapon) -> Self {
        WeaponOverride {
            attacks: w.attacks.clone(),
            skill: w.skill.clone(),
            keywords: w.keywords.clone(),
            skill_changed: false,
            attacks_changed: false,
        }
    }
}

/// Applies a single modifier to the mutable state.
/// Adding new modifiers (stratagems, canticles, etc) means adding a new match arm here.
fn apply_modifier(
    modifier: &Modifier,
    profile: &UnitProfile,
    stats: &mut UnitStats,
    changes: &mut StatChanges,
    weapons: &mut Vec<WeaponOverride>,
    ability_notes: &mut Vec<String>,
) {
    match modifier {
        Modifier::DoctrinaImperative { doctrina } => {
            apply_doctrina(doctrina, profile, stats, changes, weapons, ability_notes);
        }
        Modifier::HaloscreedEnhancement { enhancement, unit_ids } => {
            apply_haloscreed_enhancement(
                enhancement, profile, unit_ids, stats, changes, weapons, ability_notes,
            );
        }
        Modifier::Stratagem { stratagem_id, unit_id } => {
            if unit_id == &profile.id {
                apply_stratagem(stratagem_id, profile, stats, changes, weapons, ability_notes);
            }
        }
        Modifier::DamageDegradation { unit_id, bracket } => {
            if unit_id == &profile.id {
                apply_damage_bracket(*bracket, stats, changes, weapons, ability_notes);
            }
        }
    }
}

fn apply_doctrina(
    doctrina: &Doctrina,
    profile: &UnitProfile,
    stats: &mut UnitStats,
    changes: &mut StatChanges,
    weapons: &mut Vec<WeaponOverride>,
    ability_notes: &mut Vec<String>,
) {
    let is_skitarii = profile.keywords.iter().any(|k| k == "Skitarii");

    match doctrina {
        Doctrina::Protector => {
            if is_skitarii {
                stats.bs = stats.bs.improve_roll();
                changes.bs_changed = true;
                // Improve BS on all ranged weapons
                for (w, pw) in weapons.iter_mut().zip(profile.weapons.iter()) {
                    if pw.weapon_type == WeaponType::Ranged {
                        let improved = w.skill.improve_roll();
                        if improved != w.skill {
                            w.skill = improved;
                            w.skill_changed = true;
                        }
                    }
                }
            }
        }
        Doctrina::Conqueror => {
            if is_skitarii {
                stats.ws = stats.ws.improve_roll();
                changes.ws_changed = true;
                for (w, pw) in weapons.iter_mut().zip(profile.weapons.iter()) {
                    if pw.weapon_type == WeaponType::Melee {
                        let improved = w.skill.improve_roll();
                        if improved != w.skill {
                            w.skill = improved;
                            w.skill_changed = true;
                        }
                    }
                }
            }
        }
        Doctrina::EliminationVolley => {
            for (w, pw) in weapons.iter_mut().zip(profile.weapons.iter()) {
                if pw.weapon_type == WeaponType::Ranged {
                    let has_rapid_fire = pw.keywords.iter().any(|k| k.starts_with("Rapid Fire"));
                    if has_rapid_fire {
                        if let StatValue::Number(n) = w.attacks {
                            w.attacks = StatValue::Number(n * 2);
                            w.attacks_changed = true;
                        }
                        w.keywords = w.keywords.iter().map(|k| {
                            if let Some(rest) = k.strip_prefix("Rapid Fire ") {
                                if let Ok(n) = rest.parse::<u32>() {
                                    return format!("Rapid Fire {}", n * 2);
                                }
                            }
                            k.clone()
                        }).collect();
                    }
                }
            }
        }
        Doctrina::DataPsalmEnrichment => {
            ability_notes.push(
                "Data-Psalm: Enrichment — All attacks wound on a 6 regardless of Toughness."
                    .to_string(),
            );
        }
    }
}

/// Haloscreed Battle Clade — Noospheric Transference enhancement.
/// Only applies to units currently holding the Halo Override keyword (by ID).
fn apply_haloscreed_enhancement(
    enhancement: &HaloscreedEnhancement,
    profile: &UnitProfile,
    unit_ids: &[String],
    stats: &mut UnitStats,
    changes: &mut StatChanges,
    _weapons: &mut Vec<WeaponOverride>,
    ability_notes: &mut Vec<String>,
) {
    if !unit_ids.iter().any(|id| id == &profile.id) {
        return;
    }

    match enhancement {
        HaloscreedEnhancement::ElectromotiveEnergisation => {
            let new_movement = stats.movement.add_movement(2);
            if new_movement != stats.movement {
                stats.movement = new_movement;
                changes.movement_changed = true;
            }
            ability_notes.push(
                "Electromotive Energisation: +2\" to this unit's Move this turn.".to_string(),
            );
        }
        HaloscreedEnhancement::MicroactuatorBracing => {
            if let StatValue::Number(t) = stats.toughness {
                stats.toughness = StatValue::Number(t + 1);
                changes.toughness_changed = true;
            }
            ability_notes.push(
                "Microactuator Bracing: +1 to this unit's Toughness this turn.".to_string(),
            );
        }
        HaloscreedEnhancement::PredationProtocols => {
            ability_notes.push(
                "Predation Protocols: This unit can Advance and still declare a Charge this turn."
                    .to_string(),
            );
        }
        HaloscreedEnhancement::MutedServomotors => {
            ability_notes.push(
                "Muted Servomotors: Stealth — enemy units cannot target this unit from more than 12\" away."
                    .to_string(),
            );
        }
    }
}

fn apply_stratagem(
    stratagem_id: &str,
    _profile: &UnitProfile,
    _stats: &mut UnitStats,
    _changes: &mut StatChanges,
    _weapons: &mut Vec<WeaponOverride>,
    ability_notes: &mut Vec<String>,
) {
    match stratagem_id {
        "protector_imperatives_overcharge" => {
            ability_notes.push("Overcharge: Re-roll hit rolls of 1 this phase.".to_string());
        }
        _ => {}
    }
}

/// 10th edition vehicle damage: when bracket >= 1, all hit rolls are at -1.
/// Represented by worsening every weapon's skill by one step.
fn apply_damage_bracket(
    bracket: u8,
    _stats: &mut UnitStats,
    changes: &mut StatChanges,
    weapons: &mut Vec<WeaponOverride>,
    ability_notes: &mut Vec<String>,
) {
    if bracket >= 1 {
        for w in weapons.iter_mut() {
            let worsened = w.skill.worsen_roll();
            if worsened != w.skill {
                w.skill = worsened;
                w.skill_changed = true;
            }
        }
        changes.damaged = true;
        ability_notes.push("DAMAGED: -1 to all Hit rolls this turn.".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranger_profile() -> UnitProfile {
        UnitProfile {
            id: "skitarii-rangers".to_string(),
            name: "Skitarii Rangers".to_string(),
            faction: "Adeptus Mechanicus".to_string(),
            keywords: vec!["Infantry".to_string(), "Skitarii".to_string(), "Core".to_string()],
            points: 130,
            base_stats: UnitStats {
                movement: StatValue::Text("6\"".to_string()),
                toughness: StatValue::Number(3),
                save: StatValue::Roll("4+".to_string()),
                wounds: StatValue::Number(1),
                leadership: StatValue::Roll("6+".to_string()),
                oc: StatValue::Number(2),
                bs: StatValue::Roll("3+".to_string()),
                ws: StatValue::Roll("4+".to_string()),
            },
            weapons: vec![
                Weapon {
                    name: "Galvanic rifle".to_string(),
                    weapon_type: WeaponType::Ranged,
                    range: Some("30\"".to_string()),
                    attacks: StatValue::Number(2),
                    skill: StatValue::Roll("3+".to_string()),
                    strength: StatValue::Number(4),
                    ap: 0,
                    damage: StatValue::Number(1),
                    keywords: vec!["Rapid Fire 1".to_string()],
                },
                Weapon {
                    name: "Close combat weapon".to_string(),
                    weapon_type: WeaponType::Melee,
                    range: None,
                    attacks: StatValue::Number(1),
                    skill: StatValue::Roll("4+".to_string()),
                    strength: StatValue::Number(3),
                    ap: 0,
                    damage: StatValue::Number(1),
                    keywords: vec![],
                },
            ],
            abilities: vec![],
        }
    }

    #[test]
    fn test_no_modifiers_returns_base_stats() {
        let profile = ranger_profile();
        let resolved = resolve_unit(&profile, &[]);
        assert_eq!(resolved.stats.bs, StatValue::Roll("3+".to_string()));
        assert_eq!(resolved.stats.ws, StatValue::Roll("4+".to_string()));
        assert!(!resolved.stat_changes.bs_changed);
    }

    #[test]
    fn test_protector_doctrina_improves_bs() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::DoctrinaImperative { doctrina: Doctrina::Protector }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert_eq!(resolved.stats.bs, StatValue::Roll("2+".to_string()));
        assert!(resolved.stat_changes.bs_changed);
        assert!(!resolved.stat_changes.ws_changed);
    }

    #[test]
    fn test_conqueror_doctrina_improves_ws() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::DoctrinaImperative { doctrina: Doctrina::Conqueror }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert_eq!(resolved.stats.ws, StatValue::Roll("3+".to_string()));
        assert!(resolved.stat_changes.ws_changed);
        assert!(!resolved.stat_changes.bs_changed);
    }

    #[test]
    fn test_bs_cannot_improve_beyond_2_plus() {
        let val = StatValue::Roll("2+".to_string());
        assert_eq!(val.improve_roll(), StatValue::Roll("2+".to_string()));
    }

    #[test]
    fn test_elimination_volley_doubles_rapid_fire_attacks() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::DoctrinaImperative { doctrina: Doctrina::EliminationVolley }];
        let resolved = resolve_unit(&profile, &modifiers);
        let galvanic = resolved.weapons.iter().find(|w| w.name == "Galvanic rifle").unwrap();
        assert_eq!(galvanic.attacks, StatValue::Number(4));
        assert!(galvanic.attacks_changed);
    }

    #[test]
    fn test_enrichment_adds_ability_note() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::DoctrinaImperative { doctrina: Doctrina::DataPsalmEnrichment }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert!(!resolved.active_ability_notes.is_empty());
        assert!(resolved.active_ability_notes[0].contains("Enrichment"));
    }

    #[test]
    fn test_haloscreed_energisation_increases_movement_via_json() {
        // Exercises the real deserialisation path where "6\"" becomes Roll, not Text.
        let json = r#"{
            "id":"skitarii-rangers","name":"Rangers","faction":"AdMech",
            "keywords":["Skitarii"],"points":130,
            "base_stats":{"movement":"6\"","toughness":3,"save":"4+","wounds":1,
                          "leadership":"6+","oc":2,"bs":"3+","ws":"4+"},
            "weapons":[],"abilities":[]
        }"#;
        let profile: UnitProfile = serde_json::from_str(json).unwrap();
        let modifiers = vec![Modifier::HaloscreedEnhancement {
            enhancement: HaloscreedEnhancement::ElectromotiveEnergisation,
            unit_ids: vec!["skitarii-rangers".to_string()],
        }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert_eq!(resolved.stats.movement, StatValue::Text("8\"".to_string()));
        assert!(resolved.stat_changes.movement_changed);
    }

    #[test]
    fn test_haloscreed_energisation_increases_movement() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::HaloscreedEnhancement {
            enhancement: HaloscreedEnhancement::ElectromotiveEnergisation,
            unit_ids: vec!["skitarii-rangers".to_string()],
        }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert_eq!(resolved.stats.movement, StatValue::Text("8\"".to_string()));
        assert!(resolved.stat_changes.movement_changed);
    }

    #[test]
    fn test_haloscreed_bracing_increases_toughness() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::HaloscreedEnhancement {
            enhancement: HaloscreedEnhancement::MicroactuatorBracing,
            unit_ids: vec!["skitarii-rangers".to_string()],
        }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert_eq!(resolved.stats.toughness, StatValue::Number(4));
        assert!(resolved.stat_changes.toughness_changed);
    }

    #[test]
    fn test_haloscreed_does_not_affect_non_ho_units() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::HaloscreedEnhancement {
            enhancement: HaloscreedEnhancement::MicroactuatorBracing,
            unit_ids: vec!["some-other-unit".to_string()],
        }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert_eq!(resolved.stats.toughness, StatValue::Number(3));
        assert!(!resolved.stat_changes.toughness_changed);
    }

    #[test]
    fn test_haloscreed_protocols_adds_ability_note() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::HaloscreedEnhancement {
            enhancement: HaloscreedEnhancement::PredationProtocols,
            unit_ids: vec!["skitarii-rangers".to_string()],
        }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert!(resolved.active_ability_notes.iter().any(|n| n.contains("Predation")));
    }

    #[test]
    fn test_damage_bracket_worsens_weapon_skills() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::DamageDegradation {
            unit_id: "skitarii-rangers".to_string(),
            bracket: 1,
        }];
        let resolved = resolve_unit(&profile, &modifiers);
        let galvanic = resolved.weapons.iter().find(|w| w.name == "Galvanic rifle").unwrap();
        assert_eq!(galvanic.skill, StatValue::Roll("4+".to_string()));
        assert!(galvanic.skill_changed);
        assert!(resolved.stat_changes.damaged);
    }

    #[test]
    fn test_damage_bracket_zero_has_no_effect() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::DamageDegradation {
            unit_id: "skitarii-rangers".to_string(),
            bracket: 0,
        }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert!(!resolved.stat_changes.damaged);
        assert!(!resolved.weapons[0].skill_changed);
    }
}

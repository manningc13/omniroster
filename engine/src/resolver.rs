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
        Modifier::Stratagem { stratagem_id, unit_id } => {
            if unit_id == &profile.id {
                apply_stratagem(stratagem_id, profile, stats, changes, weapons, ability_notes);
            }
        }
        Modifier::DamageDegradation { unit_id, bracket } => {
            if unit_id == &profile.id {
                apply_damage_bracket(*bracket, stats, changes);
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
                for w in weapons.iter_mut() {
                    // Only improve ranged weapon BS
                    if matches!(
                        profile.weapons.iter().find(|pw| pw.name == profile.weapons[0].name),
                        Some(pw) if pw.weapon_type == WeaponType::Ranged
                    ) {
                        let improved = w.skill.improve_roll();
                        if improved != w.skill {
                            w.skill = improved;
                            w.skill_changed = true;
                        }
                    }
                }
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
                    // Double the attacks for Rapid Fire weapons
                    let has_rapid_fire = pw.keywords.iter().any(|k| k.starts_with("Rapid Fire"));
                    if has_rapid_fire {
                        if let StatValue::Number(n) = w.attacks {
                            w.attacks = StatValue::Number(n * 2);
                            w.attacks_changed = true;
                        }
                        // Update the keyword to reflect doubled shots
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

fn apply_stratagem(
    stratagem_id: &str,
    _profile: &UnitProfile,
    stats: &mut UnitStats,
    _changes: &mut StatChanges,
    _weapons: &mut Vec<WeaponOverride>,
    ability_notes: &mut Vec<String>,
) {
    // Placeholder: stratagem resolution will go here as stratagems are added.
    // Each stratagem_id maps to specific stat/weapon mutations.
    match stratagem_id {
        "protector_imperatives_overcharge" => {
            ability_notes.push("Overcharge: Re-roll hit rolls of 1 this phase.".to_string());
        }
        _ => {
            // Unknown stratagem — no-op, safe to ignore
        }
    }
}

fn apply_damage_bracket(bracket: u8, stats: &mut UnitStats, _changes: &mut StatChanges) {
    // Vehicle damage degradation — worsens stats as wounds are lost.
    // bracket 0 = full health, 1 = middle bracket, 2 = lowest bracket.
    // Concrete values should come from unit data; this is a generic fallback.
    match bracket {
        1 => {
            stats.movement = stats.movement.worsen_roll();
            stats.bs = stats.bs.worsen_roll();
        }
        2 => {
            stats.movement = stats.movement.worsen_roll();
            stats.bs = stats.bs.worsen_roll();
            stats.ws = stats.ws.worsen_roll();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

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
        let modifiers = vec![Modifier::DoctrinaImperative {
            doctrina: Doctrina::Protector,
        }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert_eq!(resolved.stats.bs, StatValue::Roll("2+".to_string()));
        assert!(resolved.stat_changes.bs_changed);
        assert!(!resolved.stat_changes.ws_changed);
    }

    #[test]
    fn test_conqueror_doctrina_improves_ws() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::DoctrinaImperative {
            doctrina: Doctrina::Conqueror,
        }];
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
        let modifiers = vec![Modifier::DoctrinaImperative {
            doctrina: Doctrina::EliminationVolley,
        }];
        let resolved = resolve_unit(&profile, &modifiers);
        let galvanic = resolved.weapons.iter().find(|w| w.name == "Galvanic rifle").unwrap();
        assert_eq!(galvanic.attacks, StatValue::Number(4));
        assert!(galvanic.attacks_changed);
    }

    #[test]
    fn test_enrichment_adds_ability_note() {
        let profile = ranger_profile();
        let modifiers = vec![Modifier::DoctrinaImperative {
            doctrina: Doctrina::DataPsalmEnrichment,
        }];
        let resolved = resolve_unit(&profile, &modifiers);
        assert!(!resolved.active_ability_notes.is_empty());
        assert!(resolved.active_ability_notes[0].contains("Enrichment"));
    }
}

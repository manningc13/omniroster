mod types;
mod resolver;

use wasm_bindgen::prelude::*;
use types::*;

/// Resolves a unit's stat card given its base profile and a list of active modifiers.
/// Called from TypeScript: `resolveUnit(profileJson, modifiersJson) => resolvedJson`
#[wasm_bindgen]
pub fn resolve_unit(profile_json: &str, modifiers_json: &str) -> Result<String, JsValue> {
    let profile: UnitProfile = serde_json::from_str(profile_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid unit profile: {}", e)))?;

    let modifiers: Vec<Modifier> = serde_json::from_str(modifiers_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid modifiers: {}", e)))?;

    let resolved = resolver::resolve_unit(&profile, &modifiers);

    serde_json::to_string(&resolved)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Validates an army list and returns a list of validation errors (empty = valid).
/// Called from TypeScript: `validateArmy(armyJson) => errorsJson`
#[wasm_bindgen]
pub fn validate_army(army_json: &str) -> Result<String, JsValue> {
    let army: ArmyList = serde_json::from_str(army_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid army list: {}", e)))?;

    let errors = run_army_validation(&army);

    serde_json::to_string(&errors)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// An army list as passed in from the frontend.
#[derive(serde::Deserialize)]
struct ArmyList {
    faction: String,
    points_limit: u32,
    units: Vec<ArmyUnit>,
}

#[derive(serde::Deserialize)]
struct ArmyUnit {
    profile_id: String,
    points: u32,
    count: u32,
}

#[derive(serde::Serialize)]
struct ValidationError {
    code: String,
    message: String,
}

fn run_army_validation(army: &ArmyList) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let total_points: u32 = army.units.iter().map(|u| u.points * u.count).sum();
    if total_points > army.points_limit {
        errors.push(ValidationError {
            code: "POINTS_EXCEEDED".to_string(),
            message: format!(
                "Army is {} points over the limit ({} / {})",
                total_points - army.points_limit,
                total_points,
                army.points_limit
            ),
        });
    }

    // Further validation (unit count limits, detachment rules, etc.) goes here.

    errors
}

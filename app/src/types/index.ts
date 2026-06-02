// Mirrors the Rust types in engine/src/types.rs.
// When you add a new field in Rust, add it here too.

export type StatValue = number | string; // number for T/W/OC, string for "3+", "6\"", "D6+2"

export interface UnitStats {
  movement: StatValue;
  toughness: StatValue;
  save: StatValue;
  wounds: StatValue;
  leadership: StatValue;
  oc: StatValue;
  bs: StatValue;
  ws: StatValue;
}

export type WeaponType = 'ranged' | 'melee';

export interface Weapon {
  name: string;
  weapon_type: WeaponType;
  range: string | null;
  attacks: StatValue;
  skill: StatValue;
  strength: StatValue;
  ap: number;
  damage: StatValue;
  keywords: string[];
}

export interface Ability {
  name: string;
  description: string;
}

export interface UnitProfile {
  id: string;
  name: string;
  faction: string;
  keywords: string[];
  base_stats: UnitStats;
  weapons: Weapon[];
  abilities: Ability[];
  points: number;
}

// Modifier types — extend this union as new mechanics are added
export type Doctrina =
  | 'protector'
  | 'conqueror'
  | 'elimination_volley'
  | 'data_psalm_enrichment';

export type Modifier =
  | { type: 'doctrina_imperative'; doctrina: Doctrina }
  | { type: 'stratagem'; stratagem_id: string; unit_id: string }
  | { type: 'damage_degradation'; unit_id: string; bracket: number };

// Resolved output from the rules engine
export interface StatChanges {
  bs_changed: boolean;
  ws_changed: boolean;
  movement_changed: boolean;
  toughness_changed: boolean;
}

export interface ResolvedWeapon {
  name: string;
  weapon_type: WeaponType;
  range: string | null;
  attacks: StatValue;
  skill: StatValue;
  strength: StatValue;
  ap: number;
  damage: StatValue;
  keywords: string[];
  skill_changed: boolean;
  attacks_changed: boolean;
}

export interface ResolvedUnit {
  id: string;
  name: string;
  faction: string;
  keywords: string[];
  stats: UnitStats;
  stat_changes: StatChanges;
  weapons: ResolvedWeapon[];
  abilities: Ability[];
  active_ability_notes: string[];
}

// Army list types
export interface ArmyUnit {
  profile_id: string;
  points: number;
  count: number;
}

export interface ArmyList {
  name: string;
  faction: string;
  points_limit: number;
  units: ArmyUnit[];
}

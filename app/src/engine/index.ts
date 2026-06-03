import init, { resolve_unit } from './omniroster_engine.js';
import type { UnitProfile, Modifier, ResolvedUnit } from '../types';

// Initialise the WASM module once; all callers await this same promise.
const ready = init();

export async function resolveUnit(
  profile: UnitProfile,
  modifiers: Modifier[],
): Promise<ResolvedUnit> {
  await ready;
  try {
    const json = resolve_unit(JSON.stringify(profile), JSON.stringify(modifiers));
    return JSON.parse(json) as ResolvedUnit;
  } catch (err) {
    console.error('[engine] resolve_unit failed for', profile.id, err);
    console.error('[engine] modifiers sent:', JSON.stringify(modifiers, null, 2));
    throw err;
  }
}

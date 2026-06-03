import init, { resolve_unit } from './omniroster_engine.js';
import type { UnitProfile, Modifier, ResolvedUnit } from '../types';

// Initialise the WASM module once; all callers await this same promise.
const ready = init();

export async function resolveUnit(
  profile: UnitProfile,
  modifiers: Modifier[],
): Promise<ResolvedUnit> {
  await ready;
  const json = resolve_unit(JSON.stringify(profile), JSON.stringify(modifiers));
  return JSON.parse(json) as ResolvedUnit;
}

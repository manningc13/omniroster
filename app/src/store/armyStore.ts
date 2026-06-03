import { create } from 'zustand';
import type {
  UnitProfile, Modifier, ResolvedUnit,
  Doctrina, Detachment, HaloscreedEnhancement, RosterUnit,
} from '../types';
import { resolveUnit } from '../engine/index.ts';

interface ArmyState {
  units: RosterUnit[];
  pointsLimit: number;
  activeModifiers: Modifier[];      // doctrina + stratagems only; derived modifiers built in _resolveAll
  selectedUnitId: string | null;
  detachment: Detachment | null;
  haloscreeedEnhancement: HaloscreedEnhancement | null;
  resolvedUnits: Record<string, ResolvedUnit>;

  selectUnit: (id: string) => void;
  setDoctrina: (doctrina: Doctrina | null) => void;
  setDetachment: (detachment: Detachment | null) => void;
  setHaloscreedEnhancement: (enhancement: HaloscreedEnhancement | null) => void;
  addUnit: (profile: UnitProfile) => void;
  removeUnit: (id: string) => void;
  toggleHaloOverride: (id: string) => void;
  setUnitWounds: (id: string, wounds: number) => void;

  _resolveAll: () => Promise<void>;
}

export const useArmyStore = create<ArmyState>((set, get) => ({
  units: [],
  pointsLimit: 2000,
  activeModifiers: [],
  selectedUnitId: null,
  detachment: null,
  haloscreeedEnhancement: null,
  resolvedUnits: {},

  selectUnit: (id) => set({ selectedUnitId: id }),

  setDoctrina: async (doctrina) => {
    const current = get().activeModifiers.filter(m => m.type !== 'doctrina_imperative');
    const next: Modifier[] = doctrina
      ? [...current, { type: 'doctrina_imperative', doctrina }]
      : current;
    set({ activeModifiers: next });
    await get()._resolveAll();
  },

  setDetachment: (detachment) => {
    // Clearing detachment also clears any detachment-specific state
    if (!detachment) {
      set({ detachment: null, haloscreeedEnhancement: null });
    } else {
      set({ detachment });
    }
  },

  setHaloscreedEnhancement: async (enhancement) => {
    set({ haloscreeedEnhancement: enhancement });
    await get()._resolveAll();
  },

  addUnit: async (profile) => {
    if (get().units.some(u => u.profile.id === profile.id)) return;
    const totalWounds = typeof profile.base_stats.wounds === 'number'
      ? profile.base_stats.wounds : 1;
    set(state => ({
      units: [...state.units, { profile, haloOverride: false, currentWounds: totalWounds }],
    }));
    await get()._resolveAll();
  },

  removeUnit: async (id) => {
    set(state => ({
      units: state.units.filter(u => u.profile.id !== id),
      selectedUnitId: state.selectedUnitId === id ? null : state.selectedUnitId,
    }));
    await get()._resolveAll();
  },

  toggleHaloOverride: async (id) => {
    set(state => ({
      units: state.units.map(u =>
        u.profile.id === id ? { ...u, haloOverride: !u.haloOverride } : u
      ),
    }));
    // HO list changed — re-resolve so Haloscreed enhancement targets the right units
    if (get().haloscreeedEnhancement) {
      await get()._resolveAll();
    }
  },

  setUnitWounds: async (id, wounds) => {
    set(state => ({
      units: state.units.map(u =>
        u.profile.id === id ? { ...u, currentWounds: Math.max(0, wounds) } : u
      ),
    }));
    await get()._resolveAll();
  },

  _resolveAll: async () => {
    const { units, activeModifiers, haloscreeedEnhancement } = get();

    // Build the full modifier list, including derived modifiers
    const allModifiers: Modifier[] = [...activeModifiers];

    // Haloscreed enhancement — targets units currently flagged as Halo Override
    const hoUnitIds = units.filter(u => u.haloOverride).map(u => u.profile.id);
    if (haloscreeedEnhancement && hoUnitIds.length > 0) {
      allModifiers.push({
        type: 'haloscreed_enhancement',
        enhancement: haloscreeedEnhancement,
        unit_ids: hoUnitIds,
      });
    }

    // Damage degradation — active when a vehicle is at or below its damage threshold
    for (const u of units) {
      const threshold = damageThreshold(u);
      if (threshold > 0 && u.currentWounds <= threshold) {
        allModifiers.push({ type: 'damage_degradation', unit_id: u.profile.id, bracket: 1 });
      }
    }

    console.debug('[store] resolving with modifiers:', allModifiers);

    try {
      const entries = await Promise.all(
        units.map(async u => {
          const resolved = await resolveUnit(u.profile, allModifiers);
          return [u.profile.id, resolved] as const;
        })
      );
      set({ resolvedUnits: Object.fromEntries(entries) });
    } catch (err) {
      console.error('[store] _resolveAll failed:', err);
    }
  },
}));

/** Returns the wound count at/below which a unit enters its damaged bracket, or 0 if N/A. */
function damageThreshold(u: RosterUnit): number {
  const isVehicle = u.profile.keywords.includes('Vehicle');
  if (!isVehicle) return 0;
  if (u.profile.damage_threshold !== undefined) return u.profile.damage_threshold;
  const total = typeof u.profile.base_stats.wounds === 'number'
    ? u.profile.base_stats.wounds : 0;
  return Math.floor(total / 2);
}

// ── Derived selectors ─────────────────────────────────────────────────────────

export const selectTotalPoints = (state: ArmyState) =>
  state.units.reduce((sum, u) => sum + u.profile.points, 0);

export const selectActiveDoctrina = (state: ArmyState): Doctrina | null => {
  const mod = state.activeModifiers.find(m => m.type === 'doctrina_imperative');
  return mod && mod.type === 'doctrina_imperative' ? mod.doctrina : null;
};

export const selectSelectedUnit = (state: ArmyState): ResolvedUnit | null => {
  if (!state.selectedUnitId) return null;
  return state.resolvedUnits[state.selectedUnitId] ?? null;
};

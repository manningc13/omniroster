import { create } from 'zustand';
import type { UnitProfile, Modifier, ResolvedUnit, Doctrina } from '../types';
import { resolveUnit } from '../engine';

interface ArmyState {
  // Army list
  units: UnitProfile[];
  pointsLimit: number;

  // Game state
  activeModifiers: Modifier[];
  selectedUnitId: string | null;

  // Resolved output (computed async by the rules engine)
  resolvedUnits: Record<string, ResolvedUnit>;

  // Actions
  selectUnit: (id: string) => void;
  setDoctrina: (doctrina: Doctrina | null) => void;
  addUnit: (profile: UnitProfile) => void;
  removeUnit: (id: string) => void;

  // Internal
  _resolveAll: () => Promise<void>;
}

export const useArmyStore = create<ArmyState>((set, get) => ({
  units: [],
  pointsLimit: 2000,
  activeModifiers: [],
  selectedUnitId: null,
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

  addUnit: async (profile) => {
    set(state => ({ units: [...state.units, profile] }));
    await get()._resolveAll();
  },

  removeUnit: async (id) => {
    set(state => ({
      units: state.units.filter(u => u.id !== id),
      selectedUnitId: state.selectedUnitId === id ? null : state.selectedUnitId,
    }));
    await get()._resolveAll();
  },

  _resolveAll: async () => {
    const { units, activeModifiers } = get();
    const entries = await Promise.all(
      units.map(async u => {
        const resolved = await resolveUnit(u, activeModifiers);
        return [u.id, resolved] as const;
      })
    );
    set({ resolvedUnits: Object.fromEntries(entries) });
  },
}));

// Derived selectors
export const selectTotalPoints = (state: ArmyState) =>
  state.units.reduce((sum, u) => sum + u.points, 0);

export const selectActiveDoctrina = (state: ArmyState): Doctrina | null => {
  const mod = state.activeModifiers.find(m => m.type === 'doctrina_imperative');
  return mod && mod.type === 'doctrina_imperative' ? mod.doctrina : null;
};

export const selectSelectedUnit = (state: ArmyState): ResolvedUnit | null => {
  if (!state.selectedUnitId) return null;
  return state.resolvedUnits[state.selectedUnitId] ?? null;
};

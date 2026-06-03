import { useState } from 'react';
import { useArmyStore, selectTotalPoints } from '../store/armyStore';
import type { UnitProfile } from '../types';
import admechData from '../data/units/admech.json';

const AVAILABLE = admechData as UnitProfile[];

export default function RosterPanel() {
  const units              = useArmyStore(s => s.units);
  const detachment         = useArmyStore(s => s.detachment);
  const selectedUnitId     = useArmyStore(s => s.selectedUnitId);
  const totalPoints        = useArmyStore(selectTotalPoints);
  const addUnit            = useArmyStore(s => s.addUnit);
  const removeUnit         = useArmyStore(s => s.removeUnit);
  const selectUnit         = useArmyStore(s => s.selectUnit);
  const toggleHaloOverride = useArmyStore(s => s.toggleHaloOverride);
  const setUnitWounds      = useArmyStore(s => s.setUnitWounds);

  const [pickedId, setPickedId] = useState(AVAILABLE[0]?.id ?? '');
  const alreadyAdded = units.some(u => u.profile.id === pickedId);

  return (
    <section className="panel roster-panel">
      <h3>Roster — {totalPoints} pts</h3>

      <div className="add-unit-row">
        <select value={pickedId} onChange={e => setPickedId(e.target.value)}>
          {AVAILABLE.map(u => (
            <option key={u.id} value={u.id}>{u.name} ({u.points} pts)</option>
          ))}
        </select>
        <button
          disabled={alreadyAdded}
          onClick={() => {
            const profile = AVAILABLE.find(u => u.id === pickedId);
            if (profile) addUnit(profile);
          }}
        >
          + Add
        </button>
      </div>

      {units.length === 0 ? (
        <p className="roster-empty">No units added yet.</p>
      ) : (
        <ul className="roster-list">
          {units.map(u => {
            const isVehicle = u.profile.keywords.includes('Vehicle');
            const totalWounds = typeof u.profile.base_stats.wounds === 'number'
              ? u.profile.base_stats.wounds : null;

            return (
              <li
                key={u.profile.id}
                className={`roster-unit${selectedUnitId === u.profile.id ? ' selected' : ''}${u.currentWounds === 0 ? ' destroyed' : ''}`}
                onClick={() => selectUnit(u.profile.id)}
              >
                <span className="unit-name">{u.profile.name}</span>

                {isVehicle && totalWounds !== null && (
                  <span className="wound-counter" onClick={e => e.stopPropagation()}>
                    <button
                      className="wound-btn"
                      onClick={() => setUnitWounds(u.profile.id, u.currentWounds - 1)}
                      disabled={u.currentWounds <= 0}
                    >−</button>
                    <span className={`wound-display${u.currentWounds === 0 ? ' destroyed' : ''}`}>
                      {u.currentWounds}/{totalWounds}W
                    </span>
                    <button
                      className="wound-btn"
                      onClick={() => setUnitWounds(u.profile.id, u.currentWounds + 1)}
                      disabled={u.currentWounds >= totalWounds}
                    >+</button>
                  </span>
                )}

                {detachment === 'haloscreed_battleclade' && (
                  <label className="halo-label" onClick={e => e.stopPropagation()}>
                    <input
                      type="checkbox"
                      checked={u.haloOverride}
                      onChange={() => toggleHaloOverride(u.profile.id)}
                    />
                    HO
                  </label>
                )}

                <span className="unit-points">{u.profile.points}</span>
                <button
                  className="remove-btn"
                  onClick={e => { e.stopPropagation(); removeUnit(u.profile.id); }}
                >✕</button>
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}

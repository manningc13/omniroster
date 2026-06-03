import { useArmyStore, selectSelectedUnit } from '../store/armyStore';
import type { ResolvedUnit, ResolvedWeapon } from '../types';

export default function StatCard() {
  const unit = useArmyStore(selectSelectedUnit);

  if (!unit) {
    return (
      <div className="stat-card-empty">
        Select a unit from the roster to view its stat card.
      </div>
    );
  }

  return (
    <div className="stat-card">
      <header className="stat-card-header">
        <div className="stat-card-title-row">
          <h2>{unit.name}</h2>
          {unit.stat_changes.damaged && (
            <span className="damaged-badge">DAMAGED</span>
          )}
        </div>
        <div className="stat-card-meta">
          <span className="faction">{unit.faction}</span>
          <span className="keywords-line">{unit.keywords.join(' · ')}</span>
        </div>
      </header>

      <StatBlock unit={unit} />
      <WeaponTable weapons={unit.weapons} />
      <AbilityList unit={unit} />
    </div>
  );
}

function StatBlock({ unit }: { unit: ResolvedUnit }) {
  const { stats, stat_changes } = unit;

  const cells = [
    { label: 'M',  value: stats.movement,   changed: stat_changes.movement_changed },
    { label: 'T',  value: stats.toughness,  changed: stat_changes.toughness_changed },
    { label: 'Sv', value: stats.save,       changed: false },
    { label: 'W',  value: stats.wounds,     changed: false },
    { label: 'Ld', value: stats.leadership, changed: false },
    { label: 'OC', value: stats.oc,         changed: false },
  ];

  return (
    <div className="stat-block">
      {cells.map(({ label, value, changed }) => (
        <div key={label} className={`stat-cell${changed ? ' changed' : ''}`}>
          <span className="stat-label">{label}</span>
          <span className="stat-value">{value}</span>
        </div>
      ))}
    </div>
  );
}

function WeaponTable({ weapons }: { weapons: ResolvedWeapon[] }) {
  if (weapons.length === 0) return null;

  const ranged = weapons.filter(w => w.weapon_type === 'ranged');
  const melee  = weapons.filter(w => w.weapon_type === 'melee');

  return (
    <div className="weapon-section">
      {ranged.length > 0 && <WeaponGroup label="Ranged" weapons={ranged} />}
      {melee.length > 0  && <WeaponGroup label="Melee"  weapons={melee} />}
    </div>
  );
}

function WeaponGroup({ label, weapons }: { label: string; weapons: ResolvedWeapon[] }) {
  return (
    <div className="weapon-group">
      <div className="weapon-group-label">{label}</div>
      <div className="weapon-table-wrapper">
        <table className="weapon-table">
          <thead>
            <tr>
              <th>Weapon</th>
              {label === 'Ranged' && <th>Range</th>}
              <th>A</th>
              <th>{label === 'Ranged' ? 'BS' : 'WS'}</th>
              <th>S</th>
              <th>AP</th>
              <th>D</th>
              <th>Keywords</th>
            </tr>
          </thead>
          <tbody>
            {weapons.map(w => (
              <tr key={w.name}>
                <td>{w.name}</td>
                {label === 'Ranged' && <td>{w.range ?? '—'}</td>}
                <td className={w.attacks_changed ? 'changed' : ''}>{w.attacks}</td>
                <td className={w.skill_changed ? 'changed' : ''}>{w.skill}</td>
                <td>{w.strength}</td>
                <td className={typeof w.ap === 'number' && w.ap < 0 ? 'ap-neg' : ''}>{w.ap}</td>
                <td>{w.damage}</td>
                <td className="kw">{w.keywords.join(', ')}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function AbilityList({ unit }: { unit: ResolvedUnit }) {
  const hasNotes     = unit.active_ability_notes.length > 0;
  const hasAbilities = unit.abilities.length > 0;
  if (!hasNotes && !hasAbilities) return null;

  return (
    <div className="ability-list">
      {hasNotes && (
        <div className="ability-group">
          {unit.active_ability_notes.map(note => (
            <div key={note} className="active-note">{note}</div>
          ))}
        </div>
      )}
      {hasAbilities && (
        <div className="ability-group">
          {unit.abilities.map(a => (
            <div key={a.name} className="ability-item">
              <strong>{a.name}: </strong>
              <span>{a.description}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

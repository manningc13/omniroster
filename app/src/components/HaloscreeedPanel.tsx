import { useArmyStore } from '../store/armyStore';
import type { HaloscreedEnhancement } from '../types';

const ENHANCEMENTS: { value: HaloscreedEnhancement; label: string; hint: string }[] = [
  { value: 'electromotive_energisation', label: 'Electromotive Energisation', hint: '+2" Move' },
  { value: 'microactuator_bracing',      label: 'Microactuator Bracing',      hint: '+1 Toughness' },
  { value: 'predation_protocols',        label: 'Predation Protocols',        hint: 'Advance + Charge' },
  { value: 'muted_servomotors',          label: 'Muted Servomotors',          hint: 'Stealth <12"' },
];

export default function HaloscreeedPanel() {
  const enhancement    = useArmyStore(s => s.haloscreedEnhancement);
  const hoCount        = useArmyStore(s => s.units.filter(u => u.haloOverride).length);
  const setEnhancement = useArmyStore(s => s.setHaloscreedEnhancement);

  return (
    <section className="panel">
      <h3>Noospheric Transference</h3>
      {hoCount === 0 ? (
        <p className="panel-hint">Mark units as HO in the roster to enable enhancements.</p>
      ) : (
        <p className="panel-hint">{hoCount} Halo Override unit{hoCount !== 1 ? 's' : ''} active.</p>
      )}
      <div className="doctrina-buttons">
        {ENHANCEMENTS.map(e => (
          <button
            key={e.value}
            className={`doctrina-btn${enhancement === e.value ? ' active' : ''}`}
            disabled={hoCount === 0}
            title={e.hint}
            onClick={() => setEnhancement(enhancement === e.value ? null : e.value)}
          >
            {e.label}
            <span className="doctrina-hint">{e.hint}</span>
          </button>
        ))}
      </div>
    </section>
  );
}

import { useArmyStore, selectActiveDoctrina } from '../store/armyStore';
import type { Doctrina } from '../types';

const DOCTRINAS: { value: Doctrina; label: string; hint: string }[] = [
  { value: 'protector',              label: 'Protector',              hint: 'Skitarii BS +1' },
  { value: 'conqueror',              label: 'Conqueror',              hint: 'Skitarii WS +1' },
  { value: 'elimination_volley',     label: 'Elimination Volley',     hint: 'Rapid Fire attacks ×2' },
  { value: 'data_psalm_enrichment',  label: 'Data-Psalm: Enrichment', hint: 'Wound on 6+ always' },
];

export default function DoctrinaSelector() {
  const active = useArmyStore(selectActiveDoctrina);
  const setDoctrina = useArmyStore(s => s.setDoctrina);

  return (
    <section className="panel">
      <h3>Doctrina Imperative</h3>
      <div className="doctrina-buttons">
        {DOCTRINAS.map(d => (
          <button
            key={d.value}
            className={`doctrina-btn${active === d.value ? ' active' : ''}`}
            title={d.hint}
            onClick={() => setDoctrina(active === d.value ? null : d.value)}
          >
            {d.label}
            <span className="doctrina-hint">{d.hint}</span>
          </button>
        ))}
      </div>
    </section>
  );
}

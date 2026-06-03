import { useArmyStore } from '../store/armyStore';
import type { Detachment } from '../types';

const DETACHMENTS: { value: Detachment; label: string }[] = [
  { value: 'haloscreed_battleclade', label: 'Haloscreed Battleclade' },
];

export default function DetachmentSelector() {
  const detachment = useArmyStore(s => s.detachment);
  const setDetachment = useArmyStore(s => s.setDetachment);

  return (
    <section className="panel">
      <h3>Detachment</h3>
      <select
        value={detachment ?? ''}
        onChange={e => {
          const v = e.target.value;
          setDetachment(v ? v as Detachment : null);
        }}
      >
        <option value="">— None —</option>
        {DETACHMENTS.map(d => (
          <option key={d.value} value={d.value}>{d.label}</option>
        ))}
      </select>
    </section>
  );
}

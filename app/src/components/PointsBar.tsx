import { useArmyStore, selectTotalPoints } from '../store/armyStore';

export default function PointsBar() {
  const total = useArmyStore(selectTotalPoints);
  const limit = useArmyStore(s => s.pointsLimit);
  const pct = Math.min((total / limit) * 100, 100);
  const over = total > limit;

  return (
    <div className="points-bar">
      <span className={`points-label${over ? ' over' : ''}`}>
        {total} / {limit} pts
      </span>
      <div className="points-track">
        <div
          className={`points-fill${over ? ' over' : ''}`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}

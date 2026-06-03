import { useArmyStore } from './store/armyStore';
import DetachmentSelector from './components/DetachmentSelector';
import DoctrinaSelector from './components/DoctrinaSelector';
import HaloscreeedPanel from './components/HaloscreeedPanel';
import PointsBar from './components/PointsBar';
import RosterPanel from './components/RosterPanel';
import StatCard from './components/StatCard';

export default function App() {
  const detachment = useArmyStore(s => s.detachment);

  return (
    <div className="app">
      <header className="app-header">
        <h1>Omniroster</h1>
        <PointsBar />
      </header>
      <main className="app-main">
        <aside className="left-panel">
          <DetachmentSelector />
          <DoctrinaSelector />
          {detachment === 'haloscreed_battleclade' && <HaloscreeedPanel />}
          <RosterPanel />
        </aside>
        <section className="right-panel">
          <StatCard />
        </section>
      </main>
    </div>
  );
}

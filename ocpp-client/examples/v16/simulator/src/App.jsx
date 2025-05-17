import React, { useState } from 'react';
import { useTauriEvents } from './hooks/useTauriEvents';
import ConnectorStatus from './components/ConnectorStatus';
import ChargerStatus from './components/ChargerStatus';
import LogPanel from './components/LogPanel';
import './App.css';

function App() {
  const { 
    connectorStatuses, 
    chargerRunning, 
    logs, 
    clearLogs, 
    numberOfConnectors // Get numberOfConnectors
  } = useTauriEvents();
  const [prettyPrintJson, setPrettyPrintJson] = useState(false);
  
  const togglePrettyPrintJson = () => {
    setPrettyPrintJson(prev => !prev);
  };

  // Generate connectorIds dynamically
  const connectorIds = Array.from({ length: numberOfConnectors }, (_, i) => i + 1);

  return (
    <div className="app">
      <header className="app-header">
        <h1>EV Charger Simulator</h1>
        <ChargerStatus running={chargerRunning} />
      </header>
      
      <main className="app-content">
        {/* Conditional rendering for connectors section */}
        {chargerRunning && numberOfConnectors > 0 && (
        <section className="connectors-section">
          <h2>Connectors</h2>
          <div className="connectors-grid">
            {connectorIds.map(id => (
                <ConnectorStatus 
                key={id}
                connectorId={id}
                  status={connectorStatuses[id] || 'Available'} // Default to 'Available' if somehow not set
                chargerRunning={chargerRunning}
              />
            ))}
          </div>
        </section>
        )}
        
        {/* Show a message if connectors are not ready */}
        {(!chargerRunning || numberOfConnectors === 0) && (
          <section className="connectors-placeholder">
            {!chargerRunning ? <p>Charger is not running. Waiting for charger to start...</p> 
                             : <p>Waiting for connector initialization...</p>}
          </section>
        )}
        
        <section className="logs-section">
          <LogPanel 
            logs={logs} 
            onClear={clearLogs}
            prettyPrintJson={prettyPrintJson}
            onTogglePrettyPrint={togglePrettyPrintJson}
          />
        </section>
      </main>
      
      <footer className="app-footer">
        
      </footer>
    </div>
  );
}

export default App;

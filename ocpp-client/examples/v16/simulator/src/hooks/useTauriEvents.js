import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { listenToConnectorStatus, listenToChargerStatus, listenToLogs } from '../services/tauriService';

const MSG_IN_PREFIX = "[MSG_IN] ";
const MSG_OUT_PREFIX = "[MSG_OUT] ";

export const useTauriEvents = () => {
  const [connectorStatuses, setConnectorStatuses] = useState({});
  const [chargerRunning, setChargerRunning] = useState(false);
  const [logs, setLogs] = useState([]);
  const [numberOfConnectors, setNumberOfConnectors] = useState(0);

  useEffect(() => {
    // Listener for Init event
    const unlistenInit = listen('init', (event) => {
      const { connectors } = event.payload;
      if (typeof connectors === 'number' && connectors > 0) {
        setNumberOfConnectors(connectors);
        // Initialize statuses for new connectors to "Available" if not already set
        setConnectorStatuses(prevStatuses => {
          const newStatuses = { ...prevStatuses };
          for (let i = 1; i <= connectors; i++) {
            if (!newStatuses[i]) { // Only set if not already present
              newStatuses[i] = 'Available';
            }
          }
          return newStatuses;
        });
      }
    });

    const connectorStatusPromise = listenToConnectorStatus((payload) => {
      const { connector_id, status } = payload;
      setConnectorStatuses(prev => ({
        ...prev,
        [connector_id]: status
      }));
    });

    const chargerStatusPromise = listenToChargerStatus((payload) => {
      const { running } = payload;
      setChargerRunning(running);
      if (!running) {
        // Optionally reset connector states or show them as unavailable when charger stops
        // For now, we just use the chargerRunning flag to control UI interaction
      }
    });

    const logPromise = listenToLogs((payload) => {
      let processedKind = payload.kind;
      let processedMessage = payload.message;
      let originalPrefix = "";

      if (payload.message.startsWith(MSG_IN_PREFIX)) {
        processedKind = 'MSG_IN';
        processedMessage = payload.message.substring(MSG_IN_PREFIX.length);
        originalPrefix = MSG_IN_PREFIX;
      } else if (payload.message.startsWith(MSG_OUT_PREFIX)) {
        processedKind = 'MSG_OUT';
        processedMessage = payload.message.substring(MSG_OUT_PREFIX.length);
        originalPrefix = MSG_OUT_PREFIX;
      }

      const timestamp = new Date().toISOString();
      setLogs(prev => [...prev, { 
        originalKind: payload.kind,
        kind: processedKind, 
        message: processedMessage, 
        originalPrefix: originalPrefix,
        timestamp 
      }]);
    });

    return () => {
      unlistenInit.then(fn => fn()); // Cleanup init listener
      connectorStatusPromise.then(unlisten => unlisten());
      chargerStatusPromise.then(unlisten => unlisten());
      logPromise.then(unlisten => unlisten());
    };
  }, []); // Empty dependency array, listeners set up once
  return {
    connectorStatuses,
    chargerRunning,
    logs,
    numberOfConnectors,
    clearLogs: () => setLogs([])
  };
};
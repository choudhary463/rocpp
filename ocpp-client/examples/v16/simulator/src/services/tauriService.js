import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

// Command functions
export const sendIdTag = async (connectorId, idTag) => {
  try {
    await invoke('send_id_tag', { 
      connectorId: connectorId, 
      idTag: idTag 
    });
    return { success: true };
  } catch (error) {
    console.error('Error sending ID tag:', error);
    return { success: false, error };
  }
};

export const setConnectorState = async (connectorId, state) => {
  if (!['plug', 'unplug', 'faulty'].includes(state)) {
    return { success: false, error: 'Invalid state. Must be "plug", "unplug", or "faulty"' };
  }
  
  try {
    await invoke('set_connector_state', { 
      connectorId: connectorId, 
      stateStr: state
    });
    return { success: true };
  } catch (error) {
    console.error('Error setting connector state:', error);
    return { success: false, error };
  }
};

// Event listeners
export const listenToConnectorStatus = (callback) => {
  return listen('connector_status', (event) => {
    callback(event.payload);
  });
};

export const listenToChargerStatus = (callback) => {
  return listen('charger_status', (event) => {
    callback(event.payload);
  });
};

export const listenToLogs = (callback) => {
  return listen('log', (event) => {
    callback(event.payload);
  });
};
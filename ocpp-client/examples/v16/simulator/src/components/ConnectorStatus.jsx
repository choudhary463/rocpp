import React, { useState } from 'react';
import { sendIdTag, setConnectorState } from '../services/tauriService';
import '../styles/ConnectorStatus.css';

const ConnectorStatus = ({ connectorId, status, chargerRunning }) => {
  const [idTag, setIdTag] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [connectorState, setConnectorStateLocal] = useState('unplug'); 
  
  const getStatusColor = () => {
    switch (status) {
      case 'Available':
        return 'green';
      case 'Preparing':
      case 'Charging':
        return 'blue';
      case 'SuspendedEVSE':
      case 'SuspendedEV':
        return 'orange';
      case 'Finishing':
        return 'purple';
      case 'Reserved':
        return 'yellow';
      case 'Unavailable':
      case 'Faulted':
        return 'red';
      default:
        return 'gray';
    }
  };

  const handleSendIdTag = async (e) => {
    e.preventDefault();
    
    if (!idTag.trim()) {
      console.warn('ID tag is empty for connector', connectorId);
      return;
    }
    
    setIsSubmitting(true);
    try {
      const result = await sendIdTag(connectorId, idTag);
      
      if (result.success) {
        setIdTag(''); // Clear input on success
      } else {
        console.error('Failed to send ID tag for connector', connectorId, result.error);
      }
    } catch (error) {
      console.error('Error sending ID tag for connector', connectorId, error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleStateChange = async (state) => {
    if (state === connectorState) return;
    setIsSubmitting(true);
    try {
      const result = await setConnectorState(connectorId, state);
      
      if (result.success) {
        setConnectorStateLocal(state);
      } else {
        console.error(`Failed to set connector ${connectorId} state to ${state}`, result.error);
      }
    } catch (error) {
      console.error(`Error setting connector ${connectorId} state to ${state}`, error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="connector-status">
      {!chargerRunning && (
        <div className="connector-loading-overlay">
          <div className="loading-spinner"></div>
          <p>Charger not running...</p>
        </div>
      )}
      
      <div className="connector-header">
        <h3>Connector {connectorId}</h3>
        <div 
          className="status-indicator" 
          style={{ backgroundColor: getStatusColor() }}
        />
      </div>
      
      <div className="status-details">
        <p>Status: <strong>{status || 'Available'}</strong></p>
      </div>
      
      <div className="connector-controls-section">
        <div className="connector-state-controls">
          <button 
            onClick={() => handleStateChange('plug')}
            disabled={!chargerRunning || isSubmitting || connectorState === 'plug'}
            className={`state-button plug-button ${connectorState === 'plug' ? 'active' : ''}`}
          >
            Plug
          </button>
          
          <button 
            onClick={() => handleStateChange('unplug')}
            disabled={!chargerRunning || isSubmitting || connectorState === 'unplug'}
            className={`state-button unplug-button ${connectorState === 'unplug' ? 'active' : ''}`}
          >
            Unplug
          </button>
          
          <button 
            onClick={() => handleStateChange('faulty')}
            disabled={!chargerRunning || isSubmitting || connectorState === 'faulty'}
            className={`state-button faulty-button ${connectorState === 'faulty' ? 'active' : ''}`}
          >
            Faulty
          </button>
        </div>
        
        <form onSubmit={handleSendIdTag} className="id-tag-form">
          <input
            type="text"
            value={idTag}
            onChange={(e) => setIdTag(e.target.value)}
            placeholder="Enter ID tag"
            disabled={!chargerRunning || isSubmitting}
            className="id-tag-input"
          />
          <button 
            type="submit" 
            disabled={!chargerRunning || isSubmitting}
            className="send-id-button"
          >
            Send RFID
          </button>
        </form>
          </div>
      </div>
  );
};

export default ConnectorStatus;
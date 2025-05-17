import React from 'react';
import '../styles/ChargerStatus.css';

const ChargerStatus = ({ running }) => {
  return (
    <div className={`charger-status ${running ? 'running' : 'stopped'}`}>
      <h2>Charger Status</h2>
      <div className="status-display">
        <div className="status-circle"></div>
        <p>{running ? 'Running' : 'Stopped'}</p>
      </div>
    </div>
  );
};

export default ChargerStatus;
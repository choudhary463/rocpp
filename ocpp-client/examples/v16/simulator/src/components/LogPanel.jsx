import React, { useRef, useEffect, useState } from 'react';
import '../styles/LogPanel.css';

const SCROLL_THRESHOLD = 50; // Pixels from bottom to still auto-scroll

const LogPanel = ({ logs, onClear, prettyPrintJson, onTogglePrettyPrint }) => {
  const logContainerRef = useRef(null);
  const logEndRef = useRef(null);
  const [isAtBottom, setIsAtBottom] = useState(true);
  const [isUserScrolledUp, setIsUserScrolledUp] = useState(false);

  // Check if user is at bottom when scrolling
  const handleScroll = () => {
    const container = logContainerRef.current;
    if (!container) return;

    const distanceFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight;
    
    // Update both states based on scroll position
    if (distanceFromBottom <= SCROLL_THRESHOLD) {
      setIsAtBottom(true);
      setIsUserScrolledUp(false);
    } else {
      setIsAtBottom(false);
      setIsUserScrolledUp(true);
    }
  };

  // Auto-scroll only if user was at bottom before new logs were added
  useEffect(() => {
    const container = logContainerRef.current;
    if (!container) return;

    if (isAtBottom) {
      // User was at bottom, so auto-scroll to new bottom
      container.scrollTo({ top: container.scrollHeight, behavior: 'smooth' });
    }
    // If user wasn't at bottom, don't auto-scroll
  }, [logs, isAtBottom]); // Depend on logs and isAtBottom

  const scrollToBottom = () => {
    const container = logContainerRef.current;
    if (container) {
      container.scrollTo({ top: container.scrollHeight, behavior: 'smooth' });
      setIsAtBottom(true);
      setIsUserScrolledUp(false);
    }
  };

  const getLogClass = (kind) => {
    switch (kind.toUpperCase()) {
      case 'ERROR': return 'log-error';
      case 'WARNING': return 'log-warning';
      case 'INFO': return 'log-info';
      case 'DEBUG': return 'log-debug';
      case 'MSG_IN': return 'log-msg-in';
      case 'MSG_OUT': return 'log-msg-out';
      default: return '';
    }
  };

  const formatTime = (timestamp) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString();
  };

  const formatDisplayMessage = (log) => {
    let displayMessage = log.message;
    if (prettyPrintJson && (log.kind === 'MSG_IN' || log.kind === 'MSG_OUT')) {
      try {
        const parsedJson = JSON.parse(log.message);
        displayMessage = JSON.stringify(parsedJson, null, 2);
      } catch (e) {
        displayMessage = log.message; 
      }
    }
    return displayMessage;
  };

  return (
    <div className="log-panel">
      <div className="log-header">
        <h2>System Logs</h2>
        <div className="log-actions">
          {isUserScrolledUp && (
            <button onClick={scrollToBottom} className="scroll-to-bottom-btn">
              Scroll to Latest
            </button>
          )}
          <button 
            onClick={onTogglePrettyPrint} 
            className={`toggle-pretty-print-btn ${prettyPrintJson ? 'active' : ''}`}
          >
            {prettyPrintJson ? 'Raw JSON' : 'Pretty JSON'}
          </button>
          <button onClick={onClear} className="clear-logs-btn">Clear Logs</button>
        </div>
      </div>
      <div 
        className="log-container" 
        ref={logContainerRef}
        onScroll={handleScroll}
      >
        {logs.length === 0 ? (
          <p className="no-logs">No logs to display</p>
        ) : (
          logs.map((log, index) => (
            <div key={index} className={`log-entry ${getLogClass(log.kind)}`}>
              <span className="log-time">[{formatTime(log.timestamp)}]</span>
              <span className="log-kind">[{log.kind}]</span>
              <span className="log-message">
                {formatDisplayMessage(log)}
              </span>
            </div>
          ))
        )}
        <div ref={logEndRef} />
      </div>
    </div>
  );
};

export default LogPanel;
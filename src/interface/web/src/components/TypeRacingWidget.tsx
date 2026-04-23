//! TypeRacing Widget - WebUI real-time type prediction component
//! 
//! Features:
//! - Real-time type prediction display
//! - Debounce mechanism to prevent LSP request explosion
//! - Prediction results highlighted and sorted by confidence
//! - Error boundary handling

import { useState, useEffect, useCallback, useRef } from 'react';

/** Prediction node data interface */
interface PredictionNode {
  type_name: string;
  confidence: number;
  source: 'LspHover' | 'LspDefinition' | 'LspReferences' | 'Heuristic' | 'Historical';
}

/** Prediction response interface */
interface PredictResponse {
  predictions: PredictionNode[];
  uri: string;
  line: number;
  character: number;
}

/** Component Props */
interface TypeRacingWidgetProps {
  uri: string;
  line: number;
  character: number;
  code: string;
  debounceMs?: number;
}

/** Error boundary state */
interface ErrorState {
  hasError: boolean;
  message: string;
}

/**
 * TypeRacingWidget - Real-time type prediction component
 */
export function TypeRacingWidget({
  uri,
  line,
  character,
  code,
  debounceMs = 300
}: TypeRacingWidgetProps) {
  const [predictions, setPredictions] = useState<PredictionNode[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<ErrorState>({ hasError: false, message: '' });
  const [selectedIndex, setSelectedIndex] = useState(0);
  
  const debounceTimerRef = useRef<number | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  /**
   * Call prediction API to get type prediction results
   */
  const fetchPredictions = useCallback(async () => {
    if (!uri || line < 0 || character < 0) return;

    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    abortControllerRef.current = new AbortController();

    setIsLoading(true);
    setError({ hasError: false, message: '' });

    try {
      const response = await fetch('/api/typeracing/predict', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ uri, line, character, code }),
        signal: abortControllerRef.current.signal
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data: PredictResponse = await response.json();
      
      // Sort by confidence (high to low)
      const sorted = data.predictions.sort((a, b) => b.confidence - a.confidence);
      setPredictions(sorted);
      setSelectedIndex(0);
    } catch (err) {
      if (err instanceof Error && err.name !== 'AbortError') {
        setError({ hasError: true, message: err.message });
      }
    } finally {
      setIsLoading(false);
    }
  }, [uri, line, character, code]);

  /**
   * Debounced position change handling
   */
  useEffect(() => {
    if (debounceTimerRef.current) {
      window.clearTimeout(debounceTimerRef.current);
    }

    debounceTimerRef.current = window.setTimeout(() => {
      fetchPredictions();
    }, debounceMs);

    return () => {
      if (debounceTimerRef.current) {
        window.clearTimeout(debounceTimerRef.current);
      }
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, [fetchPredictions, debounceMs]);

  const handleSelect = (index: number) => {
    setSelectedIndex(index);
    const selected = predictions[index];
    if (selected) {
      window.dispatchEvent(new CustomEvent('typeracing:select', {
        detail: { prediction: selected, uri, line, character }
      }));
    }
  };

  const getConfidenceColor = (confidence: number): string => {
    if (confidence >= 0.9) return '#4caf50';
    if (confidence >= 0.7) return '#ff9800';
    return '#f44336';
  };

  const getSourceIcon = (source: PredictionNode['source']): string => {
    const icons: Record<string, string> = {
      LspHover: 'H',
      LspDefinition: 'D',
      LspReferences: 'R',
      Heuristic: '?',
      Historical: '@'
    };
    return icons[source] || '?';
  };

  if (error.hasError) {
    return (
      <div className="typeracing-widget error">
        <div className="widget-header">TypeRacing Prediction</div>
        <div className="error-message">
          Prediction Error: {error.message}
        </div>
        <button onClick={fetchPredictions} className="retry-btn">
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="typeracing-widget">
      <div className="widget-header">
        <span>TypeRacing Prediction</span>
        {isLoading && <span className="loading-indicator">Loading...</span>}
      </div>

      <div className="predictions-list">
        {predictions.length === 0 && !isLoading && (
          <div className="empty-state">No predictions available</div>
        )}

        {predictions.map((pred, index) => (
          <div
            key={`${pred.type_name}-${index}`}
            className={`prediction-item ${index === selectedIndex ? 'selected' : ''}`}
            onClick={() => handleSelect(index)}
            role="button"
            tabIndex={0}
          >
            <span className="source-icon">{getSourceIcon(pred.source)}</span>
            <span className="type-name">{pred.type_name}</span>
            <span
              className="confidence-badge"
              style={{ backgroundColor: getConfidenceColor(pred.confidence) }}
            >
              {(pred.confidence * 100).toFixed(0)}%
            </span>
          </div>
        ))}
      </div>

      <div className="widget-footer">
        <small>
          {predictions.length} prediction{predictions.length !== 1 ? 's' : ''}
          {code && <span> - Line {line + 1}, Col {character + 1}</span>}
        </small>
      </div>
    </div>
  );
}

export default TypeRacingWidget;

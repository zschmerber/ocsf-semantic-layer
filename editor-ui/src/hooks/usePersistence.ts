/**
 * React hook for managing editor persistence.
 * 
 * Handles:
 * - Initializing persistence on mount
 * - Cleaning up on unmount
 * - Providing save status
 * 
 * Requirements: 6.5, 6.6
 */

import { useEffect, useState, useCallback } from 'react';
import {
  initializePersistence,
  cleanupPersistence,
  saveNow,
} from '../store/persistence';

export interface UsePersistenceResult {
  /** Whether a model was restored from localStorage on load */
  wasRestored: boolean;
  /** Whether the last save operation succeeded */
  lastSaveSucceeded: boolean | null;
  /** Force an immediate save */
  save: () => boolean;
}

/**
 * Hook to manage editor persistence lifecycle.
 * 
 * Usage:
 * ```tsx
 * function App() {
 *   const { wasRestored, save } = usePersistence();
 *   
 *   return (
 *     <div>
 *       {wasRestored && <span>Restored from previous session</span>}
 *       <button onClick={save}>Save Now</button>
 *     </div>
 *   );
 * }
 * ```
 */
export function usePersistence(): UsePersistenceResult {
  const [wasRestored, setWasRestored] = useState(false);
  const [lastSaveSucceeded, setLastSaveSucceeded] = useState<boolean | null>(null);

  useEffect(() => {
    const { restored } = initializePersistence();
    setWasRestored(restored);

    return () => {
      cleanupPersistence();
    };
  }, []);

  const save = useCallback(() => {
    const success = saveNow();
    setLastSaveSucceeded(success);
    return success;
  }, []);

  return {
    wasRestored,
    lastSaveSucceeded,
    save,
  };
}

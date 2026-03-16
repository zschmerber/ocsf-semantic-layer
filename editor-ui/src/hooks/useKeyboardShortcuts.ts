/**
 * React hook for handling keyboard shortcuts in the editor.
 * 
 * Registers global keyboard event listeners for:
 * - Ctrl+S (or Cmd+S on Mac): Trigger export/save
 * - Ctrl+Z (or Cmd+Z on Mac): Undo last action
 * - Ctrl+Y (or Cmd+Y on Mac): Redo last undone action
 * 
 * Requirements: 8.4
 */

import { useEffect, useCallback } from 'react';
import { useEditorStore } from '../store';

export interface UseKeyboardShortcutsOptions {
  /** Callback to trigger save/export action */
  onSave: () => void;
}

/**
 * Hook to register keyboard shortcuts for the editor.
 * 
 * Usage:
 * ```tsx
 * function App() {
 *   const handleSave = () => {
 *     exportModelAsYaml(model);
 *   };
 *   
 *   useKeyboardShortcuts({ onSave: handleSave });
 *   
 *   return <div>...</div>;
 * }
 * ```
 */
export function useKeyboardShortcuts({ onSave }: UseKeyboardShortcutsOptions): void {
  const undo = useEditorStore((state) => state.undo);
  const redo = useEditorStore((state) => state.redo);
  const canUndo = useEditorStore((state) => state.canUndo);
  const canRedo = useEditorStore((state) => state.canRedo);

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      // Check for modifier key (Ctrl on Windows/Linux, Cmd on Mac)
      const isModifierPressed = event.ctrlKey || event.metaKey;

      if (!isModifierPressed) {
        return;
      }

      // Normalize the key to lowercase for consistent comparison
      const key = event.key.toLowerCase();

      switch (key) {
        case 's':
          // Ctrl+S / Cmd+S: Save/Export
          event.preventDefault();
          onSave();
          break;

        case 'z':
          // Ctrl+Z / Cmd+Z: Undo
          // Note: Ctrl+Shift+Z is also a common redo shortcut on some platforms
          if (event.shiftKey) {
            // Ctrl+Shift+Z: Redo (alternative shortcut)
            event.preventDefault();
            if (canRedo) {
              redo();
            }
          } else {
            // Ctrl+Z: Undo
            event.preventDefault();
            if (canUndo) {
              undo();
            }
          }
          break;

        case 'y':
          // Ctrl+Y / Cmd+Y: Redo
          event.preventDefault();
          if (canRedo) {
            redo();
          }
          break;

        default:
          // No matching shortcut
          break;
      }
    },
    [onSave, undo, redo, canUndo, canRedo]
  );

  useEffect(() => {
    // Add global keyboard event listener
    window.addEventListener('keydown', handleKeyDown);

    // Cleanup on unmount
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);
}

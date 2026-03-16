/**
 * PluginDashboard — shows registered engine plugins with sync controls.
 */

import { useState } from 'react';
import {
  useCatalogPlugins,
  usePushSync,
  usePullSync,
  useGetDiff,
  useFullSync,
} from '../../api/catalogHooks';
import type { SyncDiff, SyncResult } from '../../types';
import { SyncDiffView } from '../SyncDiffView/SyncDiffView';
import './PluginDashboard.css';

export function PluginDashboard() {
  const { data: plugins = [], isLoading } = useCatalogPlugins();
  const pushMutation = usePushSync();
  const pullMutation = usePullSync();
  const diffMutation = useGetDiff();
  const syncMutation = useFullSync();

  const [activeDiff, setActiveDiff] = useState<{ engine: string; diff: SyncDiff } | null>(null);
  const [lastResult, setLastResult] = useState<{ engine: string; result: SyncResult } | null>(null);
  const [engineError, setEngineError] = useState<{ engine: string; message: string } | null>(null);

  const busy = (engine: string) =>
    (pushMutation.isPending && pushMutation.variables === engine) ||
    (pullMutation.isPending && pullMutation.variables === engine) ||
    (diffMutation.isPending && diffMutation.variables === engine) ||
    (syncMutation.isPending && syncMutation.variables === engine);

  const handleDiff = (engine: string) => {
    setEngineError(null);
    diffMutation.mutate(engine, {
      onSuccess: (diff) => setActiveDiff({ engine, diff }),
      onError: (e) => setEngineError({ engine, message: e.message }),
    });
  };

  const handlePush = (engine: string) => {
    setEngineError(null);
    pushMutation.mutate(engine, {
      onError: (e) => setEngineError({ engine, message: e.message }),
    });
  };

  const handlePull = (engine: string) => {
    setEngineError(null);
    pullMutation.mutate(engine, {
      onError: (e) => setEngineError({ engine, message: e.message }),
    });
  };

  const handleFullSync = (engine: string) => {
    setEngineError(null);
    setLastResult(null);
    syncMutation.mutate(engine, {
      onSuccess: (result) => setLastResult({ engine, result }),
      onError: (e) => setEngineError({ engine, message: e.message }),
    });
  };

  if (isLoading) return <div className="plugin-loading">Loading plugins…</div>;

  return (
    <div className="plugin-dashboard">
      <h3 className="plugin-dashboard-title">Engine Plugins</h3>

      {plugins.length === 0 ? (
        <div className="plugin-empty">No plugins registered.</div>
      ) : (
        <table className="plugin-table">
          <thead>
            <tr>
              <th>Engine</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {plugins.map((plugin) => (
              <tr key={plugin.engine}>
                <td className="plugin-name">{plugin.engine}</td>
                <td>
                  <span className={`status-dot ${plugin.connected ? 'connected' : 'disconnected'}`}>
                    {plugin.connected ? '● Connected' : '○ Disconnected'}
                  </span>
                </td>
                <td className="plugin-actions">
                  <button
                    className="btn-sm"
                    disabled={busy(plugin.engine)}
                    onClick={() => handleDiff(plugin.engine)}
                  >Diff</button>
                  <button
                    className="btn-sm"
                    disabled={busy(plugin.engine)}
                    onClick={() => handlePush(plugin.engine)}
                  >Push</button>
                  <button
                    className="btn-sm"
                    disabled={busy(plugin.engine)}
                    onClick={() => handlePull(plugin.engine)}
                  >Pull</button>
                  <button
                    className="btn-sm primary"
                    disabled={busy(plugin.engine)}
                    onClick={() => handleFullSync(plugin.engine)}
                  >Full Sync</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {engineError && (
        <div className="plugin-error">
          <strong>{engineError.engine}:</strong> {engineError.message}
        </div>
      )}

      {lastResult && (
        <div className="sync-result">
          <strong>{lastResult.engine} sync complete</strong>
          <span> — pushed {lastResult.result.push.entries_pushed}</span>
          <span>, pulled {lastResult.result.pull.entries_pulled}</span>
          <span>, merged {lastResult.result.pull.entries_merged}</span>
          <span>, conflicts resolved {lastResult.result.conflicts_resolved}</span>
        </div>
      )}

      {activeDiff && (
        <SyncDiffView
          diff={activeDiff.diff}
          onClose={() => setActiveDiff(null)}
        />
      )}
    </div>
  );
}

export default PluginDashboard;

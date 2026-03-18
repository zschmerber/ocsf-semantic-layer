import { Component, type ReactNode, type ErrorInfo } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
}

/**
 * Error boundary that wraps guide overlay components.
 * If a guide component throws, this catches the error and renders
 * nothing (or a provided fallback), ensuring the Tab_UI is never
 * blocked by a guide error.
 *
 * Requirement 11.4: Guide_Layer shall never throw an unhandled
 * exception that prevents the Tab_UI from rendering.
 */
export class GuideErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error('[GuideErrorBoundary] Guide component error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback ?? null;
    }
    return this.props.children;
  }
}

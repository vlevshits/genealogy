import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';
import { Button } from './button';
import { useI18n } from '../../i18n';

interface ErrorFallbackProps {
  error: Error | null;
  onReset?: () => void;
}

const ErrorFallback: React.FC<ErrorFallbackProps> = ({ onReset }) => {
  let t: (key: string) => string;
  try {
    const i18n = useI18n();
    t = i18n.t;
  } catch {
    t = (k: string) => {
      switch (k) {
        case 'common.somethingWentWrong':
          return 'Something went wrong';
        case 'common.errorBoundaryDescription':
          return 'An unexpected error occurred while rendering the page.';
        case 'common.reloadPage':
          return 'Reload page';
        default:
          return k;
      }
    };
  }

  const handleReload = () => {
    if (onReset) {
      onReset();
    }
    if (typeof window !== 'undefined') {
      window.location.reload();
    }
  };

  return (
    <div
      role="alert"
      className="min-h-screen flex items-center justify-center bg-surface p-4"
    >
      <div className="max-w-md w-full rounded-2xl border border-border bg-surface p-6 shadow-sm text-center space-y-4">
        <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-danger/10 text-danger">
          <AlertTriangle className="h-6 w-6" aria-hidden="true" />
        </div>
        <div className="space-y-1">
          <h1 className="text-lg font-semibold text-primary">
            {t('common.somethingWentWrong')}
          </h1>
          <p className="text-sm text-secondary">
            {t('common.errorBoundaryDescription')}
          </p>
        </div>
        <div className="pt-2">
          <Button
            variant="primary"
            size="md"
            leftIcon={<RefreshCw className="h-4 w-4" />}
            onClick={handleReload}
          >
            {t('common.reloadPage')}
          </Button>
        </div>
      </div>
    </div>
  );
};

export interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  initialError?: Error | null;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: Boolean(props.initialError),
      error: props.initialError ?? null,
    };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
    if (process.env.NODE_ENV !== 'production') {
      console.error('Uncaught error caught by ErrorBoundary:', error, errorInfo);
    }
  }

  handleReset = (): void => {
    this.setState({ hasError: false, error: null });
  };

  render(): ReactNode {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }
      return (
        <ErrorFallback
          error={this.state.error}
          onReset={this.handleReset}
        />
      );
    }

    return this.props.children;
  }
}

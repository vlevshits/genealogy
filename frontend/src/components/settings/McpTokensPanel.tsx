import React, { useEffect, useState } from 'react';
import {
  Check,
  Copy,
  KeyRound,
  Plus,
  Terminal,
  Trash2,
} from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  CardBody,
  CardHeader,
  Checkbox,
  EmptyState,
  TextInput,
  useToast,
} from '../ui';
import { FormField } from '../form';
import {
  mcpApi,
  McpPersonalAccessTokenDto,
  McpTokenScope,
} from '../../api/client';
import { useAuth } from '../../lib/auth-context';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';

export interface McpTokensPanelProps {
  initialTokens?: McpPersonalAccessTokenDto[];
  userRole?: 'admin' | 'editor' | 'viewer';
}

export const McpTokensPanel: React.FC<McpTokensPanelProps> = ({
  initialTokens,
  userRole,
}) => {
  const { t, locale } = useI18n();
  const { user } = useAuth();
  const { addToast } = useToast();

  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const canWrite = effectiveRole === 'admin' || effectiveRole === 'editor';
  const isAdmin = effectiveRole === 'admin';

  const [tokens, setTokens] = useState<McpPersonalAccessTokenDto[]>(
    initialTokens ?? []
  );
  const [tokenName, setTokenName] = useState('');
  const [selectedScopes, setSelectedScopes] = useState<McpTokenScope[]>([
    'read:tree',
    ...(canWrite ? (['write:draft'] as McpTokenScope[]) : []),
  ]);
  const [creating, setCreating] = useState(false);
  const [freshRawToken, setFreshRawToken] = useState<string | null>(null);
  const [copiedToken, setCopiedToken] = useState(false);
  const [copiedUrl, setCopiedUrl] = useState(false);

  useEffect(() => {
    if (initialTokens !== undefined) return;
    let cancelled = false;
    mcpApi
      .listTokens()
      .then((res) => {
        if (!cancelled) setTokens(res);
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [initialTokens]);

  const mcpEndpointUrl =
    typeof window !== 'undefined'
      ? `${window.location.origin}/api/v1/mcp`
      : 'http://localhost:8080/api/v1/mcp';

  const toggleScope = (scope: McpTokenScope) => {
    setSelectedScopes((prev) =>
      prev.includes(scope)
        ? prev.filter((s) => s !== scope)
        : [...prev, scope]
    );
  };

  const handleCreateToken = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!tokenName.trim() || selectedScopes.length === 0 || creating) return;
    setCreating(true);
    try {
      const res = await mcpApi.createToken({
        name: tokenName.trim(),
        scopes: selectedScopes,
      });
      setTokens((prev) => [res.token, ...prev]);
      setFreshRawToken(res.raw_token);
      setTokenName('');
      addToast({
        type: 'success',
        title: t('settings.mcp.createdToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.error'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setCreating(false);
    }
  };

  const handleRevokeToken = async (id: string) => {
    try {
      await mcpApi.revokeToken(id);
      setTokens((prev) => prev.filter((tok) => tok.id !== id));
      addToast({
        type: 'success',
        title: t('settings.mcp.revokedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.error'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleCopyText = async (text: string, isToken: boolean) => {
    try {
      if (typeof navigator !== 'undefined' && navigator.clipboard) {
        await navigator.clipboard.writeText(text);
      }
    } catch {
      // Ignore clipboard permission errors in test environments
    }
    if (isToken) {
      setCopiedToken(true);
      setTimeout(() => setCopiedToken(false), 2000);
    } else {
      setCopiedUrl(true);
      setTimeout(() => setCopiedUrl(false), 2000);
    }
  };

  const formatDate = (iso: string | null): string => {
    if (!iso) return t('settings.security.neverUsed');
    try {
      return new Intl.DateTimeFormat(locale === 'ru' ? 'ru-RU' : 'en-US', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  };

  return (
    <Card data-testid="mcp-tokens-panel">
      <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div className="space-y-1 min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <Terminal className="h-4.5 w-4.5 text-brand" />
            <h2 className="text-base font-semibold text-primary">
              {t('settings.mcp.title')}
            </h2>
          </div>
          <p className="text-xs text-secondary">
            {t('settings.mcp.subtitle')}
          </p>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          <code className="rounded-lg bg-surface-subtle px-2.5 py-1.5 font-mono text-xs text-secondary border border-subtle">
            {mcpEndpointUrl}
          </code>
          <Button
            size="sm"
            variant="secondary"
            data-testid="copy-mcp-endpoint-btn"
            leftIcon={
              copiedUrl ? (
                <Check className="h-3.5 w-3.5" />
              ) : (
                <Copy className="h-3.5 w-3.5" />
              )
            }
            onClick={() => void handleCopyText(mcpEndpointUrl, false)}
          >
            {copiedUrl ? t('common.copied') : t('common.copy')}
          </Button>
        </div>
      </CardHeader>

      <CardBody className="space-y-5">
        {/* One-time Raw Token Banner */}
        {freshRawToken && (
          <div
            data-testid="mcp-fresh-token-banner"
            className="rounded-xl border border-success/40 bg-surface-subtle p-4 space-y-2"
          >
            <div className="text-xs font-semibold text-success">
              {t('settings.mcp.rawTokenOnceWarning')}
            </div>
            <div className="flex flex-wrap items-center justify-between gap-2">
              <code
                data-testid="mcp-raw-token-value"
                className="font-mono text-xs sm:text-sm font-semibold text-primary break-all"
              >
                {freshRawToken}
              </code>
              <Button
                size="sm"
                variant="primary"
                data-testid="copy-mcp-raw-token-btn"
                leftIcon={
                  copiedToken ? (
                    <Check className="h-3.5 w-3.5" />
                  ) : (
                    <Copy className="h-3.5 w-3.5" />
                  )
                }
                onClick={() => void handleCopyText(freshRawToken, true)}
              >
                {copiedToken ? t('common.copied') : t('common.copy')}
              </Button>
            </div>
          </div>
        )}

        {/* Create New Token Form */}
        <form
          onSubmit={handleCreateToken}
          className="rounded-xl border border-subtle bg-surface-subtle p-4 space-y-3"
        >
          <div className="grid grid-cols-1 md:grid-cols-12 gap-4 items-start">
            <div className="md:col-span-4">
              <FormField
                name="mcp_token_name"
                label={t('settings.mcp.tokenNameLabel')}
                required
              >
                <TextInput
                  data-testid="mcp-token-name-input"
                  value={tokenName}
                  onChange={(e) => setTokenName(e.target.value)}
                  placeholder={t('settings.mcp.tokenNamePlaceholder')}
                  aria-label={t('settings.mcp.tokenNameLabel')}
                />
              </FormField>
            </div>

            <div className="md:col-span-5 space-y-1.5">
              <div className="text-xs font-medium text-secondary">
                {t('settings.mcp.scopesLabel')}
              </div>
              <div className="flex flex-col gap-2">
                <Checkbox
                  checked={selectedScopes.includes('read:tree')}
                  onCheckedChange={() => toggleScope('read:tree')}
                  label={t('settings.mcp.scopeReadTree')}
                />
                {canWrite && (
                  <>
                    <Checkbox
                      checked={selectedScopes.includes('write:draft')}
                      onCheckedChange={() => toggleScope('write:draft')}
                      label={t('settings.mcp.scopeWriteDraft')}
                    />
                    <Checkbox
                      checked={selectedScopes.includes('write:direct')}
                      onCheckedChange={() => toggleScope('write:direct')}
                      label={t('settings.mcp.scopeWriteDirect')}
                    />
                  </>
                )}
                {isAdmin && (
                  <Checkbox
                    checked={selectedScopes.includes('admin:maintenance')}
                    onCheckedChange={() => toggleScope('admin:maintenance')}
                    label={t('settings.mcp.scopeAdminMaintenance')}
                  />
                )}
              </div>
            </div>

            <div className="md:col-span-3 flex justify-end md:pt-5">
              <Button
                type="submit"
                size="sm"
                variant="primary"
                loading={creating}
                disabled={!tokenName.trim() || selectedScopes.length === 0}
                data-testid="create-mcp-token-btn"
                leftIcon={<Plus className="h-4 w-4" />}
                className="whitespace-nowrap"
              >
                {t('settings.mcp.createTokenBtn')}
              </Button>
            </div>
          </div>
        </form>

        {/* Active Tokens List */}
        {tokens.length === 0 ? (
          <EmptyState
            icon={<KeyRound className="h-6 w-6" />}
            title={t('settings.mcp.emptyTitle')}
            description={t('settings.mcp.emptyDesc')}
          />
        ) : (
          <div className="divide-y divide-border">
            {tokens.map((tok) => (
              <div
                key={tok.id}
                data-testid={`mcp-token-row-${tok.id}`}
                className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 py-3 first:pt-0 last:pb-0"
              >
                <div className="space-y-1 min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-sm font-semibold text-primary">
                      {tok.name}
                    </span>
                    <code className="font-mono text-xs text-tertiary">
                      {tok.token_prefix}…
                    </code>
                    {(tok.scopes ?? []).map((sc) => (
                      <Badge key={sc} variant="neutral" size="sm">
                        {sc}
                      </Badge>
                    ))}
                  </div>
                  <div className="text-xs text-secondary">
                    {t('settings.security.enrolledLabel')}:{' '}
                    {formatDate(tok.created_at)} ·{' '}
                    {t('settings.security.lastUsedLabel')}:{' '}
                    {formatDate(tok.last_used_at)}
                  </div>
                </div>

                <Button
                  size="sm"
                  variant="danger"
                  data-testid={`revoke-mcp-token-btn-${tok.id}`}
                  leftIcon={<Trash2 className="h-3.5 w-3.5" />}
                  onClick={() => void handleRevokeToken(tok.id)}
                >
                  {t('settings.mcp.revokeBtn')}
                </Button>
              </div>
            ))}
          </div>
        )}
      </CardBody>
    </Card>
  );
};

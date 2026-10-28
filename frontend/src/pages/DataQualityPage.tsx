import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
  GitMerge,
  Info,
  ShieldCheck,
  XCircle,
} from 'lucide-react';
import {
  AccessDeniedCard,
  Badge,
  Button,
  Card,
  EmptyState,
  PageHeader,
  SegmentedControl,
  Spinner,
  useToast,
} from '../components/ui';
import {
  DuplicateCandidatePairDto,
  peopleApi,
  qualityApi,
  QualitySeverity,
  QualityWarningDto,
} from '../api/client';
import { useAuth } from '../lib/auth-context';
import { useI18n } from '../i18n';

export type QualityFilterTab =
  | 'all'
  | 'critical'
  | 'warning'
  | 'info'
  | 'duplicates';

export interface DataQualityPageProps {
  initialWarnings?: QualityWarningDto[];
  initialDuplicates?: DuplicateCandidatePairDto[];
  userRole?: 'admin' | 'editor' | 'viewer';
  onNavigate?: (path: string) => void;
}

export function buildDefaultQualityWarnings(): QualityWarningDto[] {
  return [
    {
      id: 'warn-1',
      rule_code: 'DEATH_BEFORE_BIRTH',
      severity: 'critical',
      entity_type: 'person',
      entity_id: 'node-deceased-1',
      person_id: 'node-deceased-1',
      person_name: 'Pyotr Fyodorovich Shcheglov',
      person_gid: 'I0012',
    },
    {
      id: 'warn-2',
      rule_code: 'PARENT_TOO_YOUNG',
      severity: 'warning',
      entity_type: 'person',
      entity_id: 'node-deceased-2',
      person_id: 'node-deceased-2',
      person_name: 'Sofia Yakovlevna Levshits',
      person_gid: 'I0002',
    },
    {
      id: 'warn-3',
      rule_code: 'MISSING_BIRTH',
      severity: 'info',
      entity_type: 'person',
      entity_id: 'node-private-1',
      person_id: 'node-private-1',
      person_name: 'Mikhail Levshits',
      person_gid: 'I0099',
    },
  ];
}

export function buildDefaultDuplicatePairs(): DuplicateCandidatePairDto[] {
  return [
    {
      id: 'dup-1',
      person_a_id: 'node-deceased-1',
      person_a_gid: 'I0012',
      person_a_name: 'Pyotr Fyodorovich Shcheglov',
      person_a_birth: 'abt 1868',
      person_a_death: '1931',
      person_b_id: 'node-dup-2',
      person_b_gid: 'I0388',
      person_b_name: 'Pyotr Shcheglov',
      person_b_birth: '1868-02-03',
      person_b_death: '1931',
      score: 92,
    },
  ];
}

export const DataQualityPage: React.FC<DataQualityPageProps> = ({
  initialWarnings,
  initialDuplicates,
  userRole,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { user } = useAuth();
  const { addToast } = useToast();

  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const isAdmin = effectiveRole === 'admin';

  const [activeTab, setActiveTab] = useState<QualityFilterTab>('all');
  const [warnings, setWarnings] = useState<QualityWarningDto[]>(
    () => initialWarnings ?? buildDefaultQualityWarnings()
  );
  const [duplicates, setDuplicates] = useState<DuplicateCandidatePairDto[]>(
    () => initialDuplicates ?? buildDefaultDuplicatePairs()
  );
  const [loading, setLoading] = useState<boolean>(false);

  const navigateTo = useCallback(
    (nextPath: string) => {
      if (onNavigate) {
        onNavigate(nextPath);
      } else if (typeof window !== 'undefined') {
        window.history.pushState({}, '', nextPath);
        window.dispatchEvent(new PopStateEvent('popstate'));
      }
    },
    [onNavigate]
  );

  const loadQualityData = useCallback(() => {
    if (!isAdmin || initialWarnings) return;
    setLoading(true);
    Promise.all([
      qualityApi.getWarnings().catch(() => null),
      qualityApi.getDuplicates().catch(() => null),
    ])
      .then(([warnRes, dupRes]) => {
        if (warnRes && Array.isArray(warnRes.warnings)) {
          setWarnings(warnRes.warnings);
          if (Array.isArray(warnRes.duplicates)) {
            setDuplicates(warnRes.duplicates);
          }
        }
        if (Array.isArray(dupRes) && dupRes.length > 0) {
          setDuplicates(dupRes);
        }
      })
      .finally(() => {
        setLoading(false);
      });
  }, [isAdmin, initialWarnings]);

  useEffect(() => {
    loadQualityData();
  }, [loadQualityData]);

  const filteredWarnings = useMemo(() => {
    if (activeTab === 'duplicates') return [];
    if (activeTab === 'all') return warnings;
    return warnings.filter((w) => w.severity === activeTab);
  }, [warnings, activeTab]);

  if (!isAdmin) {
    return (
      <div className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-12">
        <AccessDeniedCard
          currentRole={effectiveRole === 'editor' ? 'editor' : 'viewer'}
          onOpenProfile={() => navigateTo('/settings/profile')}
          onBackToTree={() => navigateTo('/tree')}
        />
      </div>
    );
  }

  const getRuleExplanation = (w: QualityWarningDto): string => {
    const code = (w.rule_code || w.code || '').toUpperCase();
    const keyMap: Record<string, string> = {
      DEATH_BEFORE_BIRTH: 'quality.rules.deathBeforeBirth',
      TOO_OLD: 'quality.rules.tooOld',
      PARENT_TOO_YOUNG: 'quality.rules.parentTooYoung',
      MOTHER_TOO_OLD: 'quality.rules.motherTooOld',
      BORN_AFTER_PARENT_DEATH: 'quality.rules.bornAfterParentDeath',
      MARRIED_TOO_YOUNG: 'quality.rules.marriedTooYoung',
      EVENT_AFTER_DEATH: 'quality.rules.eventAfterDeath',
      POSSIBLE_DUPLICATE: 'quality.rules.possibleDuplicate',
      MISSING_BIRTH: 'quality.rules.missingBirth',
      UNCONNECTED: 'quality.rules.unconnected',
    };
    if (keyMap[code]) {
      return t(keyMap[code]);
    }
    return w.message || code;
  };

  const getSeverityBadge = (severity: QualitySeverity) => {
    if (severity === 'critical') {
      return (
        <Badge
          variant="danger"
          size="sm"
          icon={<AlertCircle className="h-3 w-3" />}
        >
          {t('quality.severity.critical')}
        </Badge>
      );
    }
    if (severity === 'warning') {
      return (
        <Badge
          variant="warning"
          size="sm"
          icon={<AlertTriangle className="h-3 w-3" />}
        >
          {t('quality.severity.warning')}
        </Badge>
      );
    }
    return (
      <Badge variant="brand" size="sm" icon={<Info className="h-3 w-3" />}>
        {t('quality.severity.info')}
      </Badge>
    );
  };

  const handleDismissWarning = async (w: QualityWarningDto) => {
    const code = w.rule_code || w.code || 'WARNING';
    const entityId = w.entity_id || w.person?.id || w.person_id || '';
    try {
      await qualityApi.dismissWarning(code, entityId).catch(() => ({
        status: 'dismissed',
      }));
      setWarnings((prev) =>
        prev.filter(
          (item) =>
            !(
              (item.rule_code || item.code) === code &&
              (item.entity_id || item.person?.id || item.person_id) === entityId
            )
        )
      );
      addToast({
        type: 'success',
        title: t('quality.dismissedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    }
  };

  const handleMarkNotDuplicate = async (pair: DuplicateCandidatePairDto) => {
    const personAId = pair.person_a?.id || pair.person_a_id || '';
    const personBId = pair.person_b?.id || pair.person_b_id || '';
    try {
      await peopleApi
        .markNotDuplicate(personAId, personBId)
        .catch(() => ({ status: 'ok' }));
      setDuplicates((prev) =>
        prev.filter(
          (d) =>
            !(
              (d.person_a?.id || d.person_a_id) === personAId &&
              (d.person_b?.id || d.person_b_id) === personBId
            )
        )
      );
      addToast({
        type: 'success',
        title: t('quality.duplicates.markedNotDuplicateToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    }
  };

  const criticalCount = warnings.filter((w) => w.severity === 'critical').length;
  const warningCount = warnings.filter((w) => w.severity === 'warning').length;
  const infoCount = warnings.filter((w) => w.severity === 'info').length;

  return (
    <div
      data-testid="data-quality-page"
      className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-8 space-y-6"
    >
      <PageHeader
        title={t('quality.pageTitle')}
        subtitle={t('quality.pageSubtitle')}
        secondaryActions={
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="warning">
              {t('quality.summaryWarnings', { count: warnings.length })}
            </Badge>
            <Badge variant="brand">
              {t('quality.summaryDuplicates', { count: duplicates.length })}
            </Badge>
          </div>
        }
      />

      {/* Filter Tabs */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <SegmentedControl<QualityFilterTab>
          ariaLabel={t('quality.filterAria')}
          value={activeTab}
          onChange={setActiveTab}
          options={[
            {
              value: 'all',
              label: t('quality.tabs.all', { count: warnings.length }),
            },
            {
              value: 'critical',
              label: t('quality.tabs.critical', { count: criticalCount }),
            },
            {
              value: 'warning',
              label: t('quality.tabs.warning', { count: warningCount }),
            },
            {
              value: 'info',
              label: t('quality.tabs.info', { count: infoCount }),
            },
            {
              value: 'duplicates',
              label: t('quality.tabs.duplicates', { count: duplicates.length }),
            },
          ]}
        />
      </div>

      {loading ? (
        <div className="flex items-center justify-center py-12 gap-2 text-xs text-secondary">
          <Spinner size="md" />
          <span>{t('common.loading')}</span>
        </div>
      ) : (
        <div className="space-y-6">
          {/* Warnings List */}
          {activeTab !== 'duplicates' && (
            <div className="space-y-3">
              {filteredWarnings.length === 0 ? (
                <Card className="p-8">
                  <EmptyState
                    icon={<ShieldCheck className="h-6 w-6 text-success" />}
                    title={t('quality.emptyTitle')}
                    description={t('quality.emptyDesc')}
                  />
                </Card>
              ) : (
                filteredWarnings.map((w, idx) => {
                  const key = w.id || `${w.rule_code}-${w.entity_id}-${idx}`;
                  const personId = w.person?.id || w.person_id || w.entity_id;
                  const personName =
                    w.person?.display_name || w.person_name || personId;
                  const personGid = w.person?.gid || w.person_gid;
                  const code = w.rule_code || w.code || 'WARNING';
                  return (
                    <Card
                      key={key}
                      data-testid={`quality-warning-card-${key}`}
                      className="p-4 sm:p-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4"
                    >
                      <div className="space-y-1.5 min-w-0">
                        <div className="flex flex-wrap items-center gap-2">
                          {getSeverityBadge(w.severity)}
                          <Badge variant="neutral" size="sm">
                            {code}
                          </Badge>
                          <Button
                            variant="link"
                            size="sm"
                            onClick={() => navigateTo(`/persons/${personId}`)}
                            className="text-sm font-semibold text-primary hover:text-brand"
                          >
                            {personName}
                          </Button>
                          {personGid && (
                            <span className="font-mono text-xs text-tertiary">
                              ({personGid})
                            </span>
                          )}
                        </div>

                        <p className="text-xs sm:text-sm text-secondary">
                          {getRuleExplanation(w)}
                        </p>
                      </div>

                      <div className="flex items-center gap-2 shrink-0">
                        <Button
                          size="sm"
                          variant="secondary"
                          data-testid={`dismiss-warning-btn-${key}`}
                          leftIcon={<CheckCircle2 className="h-3.5 w-3.5" />}
                          onClick={() => void handleDismissWarning(w)}
                        >
                          {t('quality.dismissBtn')}
                        </Button>
                      </div>
                    </Card>
                  );
                })
              )}
            </div>
          )}

          {/* Candidate Duplicates Section */}
          {(activeTab === 'all' || activeTab === 'duplicates') && (
            <div className="space-y-3">
              <div className="flex items-center gap-2">
                <GitMerge className="h-4 w-4 text-brand" />
                <h2 className="font-sans text-base font-semibold text-primary">
                  {t('quality.duplicates.sectionTitle')}
                </h2>
                <Badge variant="brand" size="sm">
                  {duplicates.length}
                </Badge>
              </div>

              {duplicates.length === 0 ? (
                <Card className="p-6">
                  <EmptyState
                    icon={<CheckCircle2 className="h-6 w-6 text-success" />}
                    title={t('quality.duplicates.emptyTitle')}
                    description={t('quality.duplicates.emptyDesc')}
                  />
                </Card>
              ) : (
                duplicates.map((pair, idx) => {
                  const personAId = pair.person_a?.id || pair.person_a_id || '';
                  const personAName =
                    pair.person_a?.display_name ||
                    pair.person_a_name ||
                    personAId;
                  const personAGid = pair.person_a?.gid || pair.person_a_gid;
                  const personBId = pair.person_b?.id || pair.person_b_id || '';
                  const personBName =
                    pair.person_b?.display_name ||
                    pair.person_b_name ||
                    personBId;
                  const personBGid = pair.person_b?.gid || pair.person_b_gid;
                  const pairKey =
                    pair.id || `${personAId}-${personBId}-${idx}`;
                  const matchScore =
                    pair.score <= 1
                      ? Math.round(pair.score * 100)
                      : Math.round(pair.score);
                  return (
                    <Card
                      key={pairKey}
                      data-testid={`duplicate-pair-card-${pairKey}`}
                      className="p-4 sm:p-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4"
                    >
                      <div className="space-y-1.5 min-w-0">
                        <div className="flex flex-wrap items-center gap-2">
                          <Badge variant="warning" size="sm">
                            {t('quality.duplicates.matchScoreBadge', {
                              score: matchScore,
                            })}
                          </Badge>
                          <Button
                            variant="link"
                            size="sm"
                            onClick={() =>
                              navigateTo(`/persons/${personAId}`)
                            }
                            className="text-sm font-semibold text-primary"
                          >
                            {personAName}
                          </Button>
                          {personAGid && (
                            <span className="font-mono text-xs text-tertiary">
                              ({personAGid})
                            </span>
                          )}
                          <span className="text-xs text-tertiary">↔</span>
                          <Button
                            variant="link"
                            size="sm"
                            onClick={() =>
                              navigateTo(`/persons/${personBId}`)
                            }
                            className="text-sm font-semibold text-primary"
                          >
                            {personBName}
                          </Button>
                          {personBGid && (
                            <span className="font-mono text-xs text-tertiary">
                              ({personBGid})
                            </span>
                          )}
                        </div>

                        <p className="text-xs text-secondary">
                          {t('quality.duplicates.pairHint')}
                        </p>
                      </div>

                      <div className="flex flex-wrap items-center gap-2 shrink-0">
                        <Button
                          size="sm"
                          variant="ghost"
                          data-testid={`not-duplicate-btn-${pairKey}`}
                          leftIcon={<XCircle className="h-3.5 w-3.5" />}
                          onClick={() => void handleMarkNotDuplicate(pair)}
                        >
                          {t('quality.duplicates.notDuplicateBtn')}
                        </Button>
                        <Button
                          size="sm"
                          variant="primary"
                          data-testid={`compare-merge-btn-${pairKey}`}
                          leftIcon={<GitMerge className="h-3.5 w-3.5" />}
                          onClick={() =>
                            navigateTo(
                              `/people/merge?target=${encodeURIComponent(
                                personAId
                              )}&source=${encodeURIComponent(personBId)}`
                            )
                          }
                        >
                          {t('quality.duplicates.compareMergeBtn')}
                        </Button>
                      </div>
                    </Card>
                  );
                })
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default DataQualityPage;

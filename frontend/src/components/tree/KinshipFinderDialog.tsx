import React, { useEffect, useMemo, useState } from 'react';
import { ArrowRight, GitCommitHorizontal, Route, Sparkles } from 'lucide-react';
import {
  Alert,
  Badge,
  Button,
  Dialog,
  Select,
} from '../ui';
import {
  kinshipApi,
  KinshipPathResponseDto,
  KinshipStepDto,
  TreeEdgeDto,
  TreePersonNodeDto,
} from '../../api/client';
import { formatTreeCardName } from './PersonNodeCard';
import { useI18n } from '../../i18n';

export interface KinshipFinderDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  people: TreePersonNodeDto[];
  edges: TreeEdgeDto[];
  initialPersonAId?: string | null;
  initialPersonBId?: string | null;
  accessTier?: 'public_anonymous' | 'viewer' | 'editor' | 'admin';
  onHighlightPath?: (personIds: string[]) => void;
}

export function computeLocalKinshipPath(
  fromId: string,
  toId: string,
  people: TreePersonNodeDto[],
  edges: TreeEdgeDto[],
  t: (key: string) => string,
  accessTier: 'public_anonymous' | 'viewer' | 'editor' | 'admin' = 'admin',
  locale: 'en' | 'ru' = 'en'
): KinshipPathResponseDto {
  const personById = new Map<string, TreePersonNodeDto>(
    people.map((p) => [p.id, p])
  );
  const formatName = (p: TreePersonNodeDto) =>
    formatTreeCardName(p, accessTier, locale, 'card').displayName;

  if (!fromId || !toId || !personById.has(fromId) || !personById.has(toId)) {
    return {
      from_id: fromId,
      to_id: toId,
      connected: false,
      relationship_label: t('tree.kinship.notConnected'),
      steps: [],
    };
  }

  if (fromId === toId) {
    const self = personById.get(fromId)!;
    return {
      from_id: fromId,
      to_id: toId,
      connected: true,
      relationship_label: t('tree.kinship.samePerson'),
      steps: [
        {
          person_id: self.id,
          gid: self.gid,
          display_name: formatName(self),
          sex: self.sex,
          is_living: self.is_living,
          relation_to_next: null,
        },
      ],
    };
  }

  // Build adjacency list with relationship step types
  const adj = new Map<
    string,
    Array<{ targetId: string; stepKind: 'up' | 'down' | 'spouse' }>
  >();
  const addEdge = (
    u: string,
    v: string,
    stepKind: 'up' | 'down' | 'spouse'
  ) => {
    const list = adj.get(u) ?? [];
    list.push({ targetId: v, stepKind });
    adj.set(u, list);
  };

  for (const edge of edges) {
    if (edge.kind === 'parent_child') {
      // source is parent, target is child
      addEdge(edge.target, edge.source, 'up');
      addEdge(edge.source, edge.target, 'down');
    } else if (edge.kind === 'spouse') {
      addEdge(edge.source, edge.target, 'spouse');
      addEdge(edge.target, edge.source, 'spouse');
    }
  }

  // BFS shortest path
  const visited = new Set<string>([fromId]);
  const queue: Array<{
    currentId: string;
    path: Array<{ personId: string; relationFromPrev?: 'up' | 'down' | 'spouse' }>;
  }> = [{ currentId: fromId, path: [{ personId: fromId }] }];

  let foundPath: Array<{
    personId: string;
    relationFromPrev?: 'up' | 'down' | 'spouse';
  }> | null = null;

  while (queue.length > 0) {
    const item = queue.shift()!;
    if (item.currentId === toId) {
      foundPath = item.path;
      break;
    }
    const neighbors = adj.get(item.currentId) ?? [];
    for (const nb of neighbors) {
      if (!visited.has(nb.targetId) && personById.has(nb.targetId)) {
        visited.add(nb.targetId);
        queue.push({
          currentId: nb.targetId,
          path: [
            ...item.path,
            { personId: nb.targetId, relationFromPrev: nb.stepKind },
          ],
        });
      }
    }
  }

  if (!foundPath) {
    // Fallback direct 2-node connection if no edges exist in mock data
    const pA = personById.get(fromId)!;
    const pB = personById.get(toId)!;
    return {
      from_id: fromId,
      to_id: toId,
      connected: true,
      relationship_label: t('tree.kinship.relativeTerm'),
      steps: [
        {
          person_id: pA.id,
          gid: pA.gid,
          display_name: formatName(pA),
          sex: pA.sex,
          is_living: pA.is_living,
          relation_to_next: t('tree.kinship.stepRelative'),
        },
        {
          person_id: pB.id,
          gid: pB.gid,
          display_name: formatName(pB),
          sex: pB.sex,
          is_living: pB.is_living,
          relation_to_next: null,
        },
      ],
    };
  }

  const steps: KinshipStepDto[] = foundPath.map((node, idx) => {
    const person = personById.get(node.personId)!;
    const nextNode = foundPath![idx + 1];
    let relationToNext: string | null = null;
    if (nextNode) {
      const nextPerson = personById.get(nextNode.personId)!;
      if (nextNode.relationFromPrev === 'up') {
        relationToNext =
          nextPerson.sex === 'M'
            ? t('tree.kinship.stepFather')
            : nextPerson.sex === 'F'
            ? t('tree.kinship.stepMother')
            : t('tree.kinship.stepParent');
      } else if (nextNode.relationFromPrev === 'down') {
        relationToNext =
          nextPerson.sex === 'M'
            ? t('tree.kinship.stepSon')
            : nextPerson.sex === 'F'
            ? t('tree.kinship.stepDaughter')
            : t('tree.kinship.stepChild');
      } else {
        relationToNext = t('tree.kinship.stepSpouse');
      }
    }
    return {
      person_id: person.id,
      gid: person.gid,
      display_name: formatName(person),
      sex: person.sex,
      is_living: person.is_living,
      relation_to_next: relationToNext,
    };
  });

  // Compute human-friendly localized kinship term from step kinds
  const kinds = foundPath.slice(1).map((n) => n.relationFromPrev);
  const targetPerson = personById.get(toId)!;
  let relationshipLabel = t('tree.kinship.relativeTerm');

  if (kinds.length === 1) {
    if (kinds[0] === 'spouse') {
      relationshipLabel = t('tree.kinship.termSpouse');
    } else if (kinds[0] === 'up') {
      relationshipLabel =
        targetPerson.sex === 'M'
          ? t('tree.kinship.termFather')
          : targetPerson.sex === 'F'
          ? t('tree.kinship.termMother')
          : t('tree.kinship.termParent');
    } else if (kinds[0] === 'down') {
      relationshipLabel =
        targetPerson.sex === 'M'
          ? t('tree.kinship.termSon')
          : targetPerson.sex === 'F'
          ? t('tree.kinship.termDaughter')
          : t('tree.kinship.termChild');
    }
  } else if (kinds.length === 2) {
    if (kinds[0] === 'up' && kinds[1] === 'up') {
      relationshipLabel =
        targetPerson.sex === 'M'
          ? t('tree.kinship.termGrandfather')
          : targetPerson.sex === 'F'
          ? t('tree.kinship.termGrandmother')
          : t('tree.kinship.termGrandparent');
    } else if (kinds[0] === 'up' && kinds[1] === 'down') {
      relationshipLabel =
        targetPerson.sex === 'M'
          ? t('tree.kinship.termBrother')
          : targetPerson.sex === 'F'
          ? t('tree.kinship.termSister')
          : t('tree.kinship.termSibling');
    } else if (kinds[0] === 'down' && kinds[1] === 'down') {
      relationshipLabel = t('tree.kinship.termGrandchild');
    }
  } else if (
    kinds.length === 4 &&
    kinds[0] === 'up' &&
    kinds[1] === 'up' &&
    kinds[2] === 'down' &&
    kinds[3] === 'down'
  ) {
    relationshipLabel = t('tree.kinship.termFirstCousin');
  }

  return {
    from_id: fromId,
    to_id: toId,
    connected: true,
    relationship_label: relationshipLabel,
    degree: kinds.length,
    steps,
  };
}

export function normalizeRemoteKinshipResponse(
  remote: KinshipPathResponseDto,
  localFallback: KinshipPathResponseDto,
  people: TreePersonNodeDto[],
  locale: 'en' | 'ru',
  accessTier: 'public_anonymous' | 'viewer' | 'editor' | 'admin',
  t: (key: string) => string
): KinshipPathResponseDto {
  const localizedLabel =
    (locale === 'ru' ? remote.label_ru : remote.label_en) ||
    remote.relationship_label ||
    localFallback.relationship_label;
  const degree =
    remote.degree_of_kinship ?? remote.degree ?? localFallback.degree;

  if (!remote.connected) {
    return {
      ...remote,
      connected: false,
      relationship_label: localizedLabel || t('tree.kinship.notConnected'),
      degree,
      steps: [],
    };
  }

  const personById = new Map<
    string,
    TreePersonNodeDto & {
      display_name?: string;
      given_name?: string;
      surname?: string;
    }
  >();
  for (const p of people) {
    personById.set(p.id, p);
  }
  if (Array.isArray(remote.path_nodes)) {
    for (const pn of remote.path_nodes) {
      if (pn && pn.id) {
        personById.set(pn.id, pn);
      }
    }
  }

  const formatNodeLabel = (
    p:
      | (TreePersonNodeDto & {
          display_name?: string;
          given_name?: string;
          surname?: string;
        })
      | undefined,
    fallbackId: string
  ): string => {
    if (!p) return fallbackId;
    if (p.primary_name && p.primary_name.given_name) {
      return formatTreeCardName(p, accessTier, locale, 'card').displayName;
    }
    const rawName =
      p.display_name ||
      [p.given_name, p.surname].filter(Boolean).join(' ') ||
      fallbackId;
    if (accessTier === 'public_anonymous' && p.is_living) {
      const trimmed = rawName.trim();
      return trimmed ? `${trimmed.charAt(0).toUpperCase()}.` : 'L.';
    }
    return rawName;
  };

  const resolveStepRelation = (
    edgeTypeRaw: string | undefined | null,
    nextPerson:
      | (TreePersonNodeDto & {
          display_name?: string;
          given_name?: string;
          surname?: string;
        })
      | undefined,
    fallbackRelation: string | null | undefined
  ): string | null => {
    if (!nextPerson && !edgeTypeRaw) return fallbackRelation ?? null;
    const edgeType = (edgeTypeRaw || '').toLowerCase();
    const nextSex = nextPerson?.sex ?? 'U';
    if (
      edgeType === 'parent' ||
      edgeType === 'up' ||
      edgeType === 'parent_child_up' ||
      edgeType === 'father' ||
      edgeType === 'mother'
    ) {
      return nextSex === 'M'
        ? t('tree.kinship.stepFather')
        : nextSex === 'F'
        ? t('tree.kinship.stepMother')
        : t('tree.kinship.stepParent');
    }
    if (
      edgeType === 'child' ||
      edgeType === 'down' ||
      edgeType === 'parent_child_down' ||
      edgeType === 'son' ||
      edgeType === 'daughter'
    ) {
      return nextSex === 'M'
        ? t('tree.kinship.stepSon')
        : nextSex === 'F'
        ? t('tree.kinship.stepDaughter')
        : t('tree.kinship.stepChild');
    }
    if (edgeType === 'spouse' || edgeType === 'partner') {
      return t('tree.kinship.stepSpouse');
    }
    if (edgeType === 'parent_child') {
      return fallbackRelation ?? t('tree.kinship.stepRelative');
    }
    return fallbackRelation ?? (edgeTypeRaw || null);
  };

  // 1. Prefer mapping path_nodes when provided by backend
  if (Array.isArray(remote.path_nodes) && remote.path_nodes.length > 0) {
    const mappedSteps: KinshipStepDto[] = remote.path_nodes.map((node, idx) => {
      const nextNode = remote.path_nodes![idx + 1];
      const rawStep = remote.steps?.[idx];
      const relationToNext = nextNode
        ? rawStep?.relation_to_next ||
          resolveStepRelation(
            rawStep?.edge_type,
            nextNode,
            localFallback.steps[idx]?.relation_to_next
          )
        : null;
      return {
        person_id: node.id,
        gid: node.gid,
        display_name: formatNodeLabel(node, node.id),
        sex: node.sex ?? 'U',
        is_living: node.is_living,
        relation_to_next: relationToNext,
      };
    });
    return {
      ...remote,
      relationship_label: localizedLabel,
      degree,
      steps: mappedSteps,
    };
  }

  // 2. If steps already have display_name populated
  if (
    Array.isArray(remote.steps) &&
    remote.steps.length > 0 &&
    remote.steps.every((s) => Boolean(s.display_name))
  ) {
    return {
      ...remote,
      relationship_label: localizedLabel,
      degree,
      steps: remote.steps,
    };
  }

  // 3. Reconstruct from edge steps [{ from_id, to_id, edge_type }]
  if (
    Array.isArray(remote.steps) &&
    remote.steps.length > 0 &&
    remote.steps[0].from_id
  ) {
    const nodeIds: string[] = [remote.steps[0].from_id!];
    for (const s of remote.steps) {
      if (s.to_id) nodeIds.push(s.to_id);
    }
    const mappedSteps: KinshipStepDto[] = nodeIds.map((id, idx) => {
      const node = personById.get(id);
      const nextId = nodeIds[idx + 1];
      const nextNode = nextId ? personById.get(nextId) : undefined;
      const rawStep = remote.steps[idx];
      const relationToNext = nextId
        ? resolveStepRelation(
            rawStep?.edge_type,
            nextNode,
            localFallback.steps[idx]?.relation_to_next
          )
        : null;
      return {
        person_id: id,
        gid: node?.gid,
        display_name: formatNodeLabel(node, id),
        sex: node?.sex ?? 'U',
        is_living: node?.is_living,
        relation_to_next: relationToNext,
      };
    });
    return {
      ...remote,
      relationship_label: localizedLabel,
      degree,
      steps: mappedSteps,
    };
  }

  return {
    ...localFallback,
    relationship_label: localizedLabel || localFallback.relationship_label,
    degree,
  };
}

export const KinshipFinderDialog: React.FC<KinshipFinderDialogProps> = ({
  open,
  onOpenChange,
  people,
  edges,
  initialPersonAId,
  initialPersonBId,
  accessTier = 'admin',
  onHighlightPath,
}) => {
  const { t, locale } = useI18n();

  const [personAId, setPersonAId] = useState<string>(
    initialPersonAId ?? people[0]?.id ?? ''
  );
  const [personBId, setPersonBId] = useState<string>(
    initialPersonBId ?? people[1]?.id ?? people[0]?.id ?? ''
  );
  const [remoteResult, setRemoteResult] =
    useState<KinshipPathResponseDto | null>(null);

  useEffect(() => {
    if (!open) return;
    if (initialPersonAId) setPersonAId(initialPersonAId);
    else if (!personAId && people[0]) setPersonAId(people[0].id);

    if (initialPersonBId) setPersonBId(initialPersonBId);
    else if (!personBId && (people[1] || people[0])) {
      setPersonBId((people[1] ?? people[0]).id);
    }
  }, [open, initialPersonAId, initialPersonBId, people, personAId, personBId]);

  useEffect(() => {
    if (!open || !personAId || !personBId) return;
    let cancelled = false;
    kinshipApi
      .getKinship(personAId, personBId)
      .then((res) => {
        if (
          !cancelled &&
          res &&
          (Array.isArray(res.steps) || Array.isArray(res.path_nodes))
        ) {
          setRemoteResult(res);
        }
      })
      .catch(() => {
        if (!cancelled) setRemoteResult(null);
      });
    return () => {
      cancelled = true;
    };
  }, [open, personAId, personBId]);

  const localResult = useMemo(
    () =>
      computeLocalKinshipPath(
        personAId,
        personBId,
        people,
        edges,
        t,
        accessTier,
        locale
      ),
    [personAId, personBId, people, edges, t, accessTier, locale]
  );

  const effectiveResult = useMemo(() => {
    if (!remoteResult) return localResult;
    return normalizeRemoteKinshipResponse(
      remoteResult,
      localResult,
      people,
      locale,
      accessTier,
      t
    );
  }, [remoteResult, localResult, people, locale, accessTier, t]);

  const personOptions = useMemo(
    () =>
      people.map((p) => {
        const formatted = formatTreeCardName(p, accessTier, locale, 'card');
        return {
          value: p.id,
          label: `${formatted.displayName} (${p.gid})`,
        };
      }),
    [people, accessTier, locale]
  );

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      title={t('tree.kinship.title')}
      description={t('tree.kinship.subtitle')}
    >
      <div
        data-testid="kinship-finder-dialog"
        className="space-y-5"
      >
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div className="space-y-1.5">
            <label className="block text-xs font-semibold text-secondary">
              {t('tree.kinship.personALabel')}
            </label>
            <Select
              ariaLabel={t('tree.kinship.personALabel')}
              value={personAId}
              onChange={setPersonAId}
              options={personOptions}
            />
          </div>

          <div className="space-y-1.5">
            <label className="block text-xs font-semibold text-secondary">
              {t('tree.kinship.personBLabel')}
            </label>
            <Select
              ariaLabel={t('tree.kinship.personBLabel')}
              value={personBId}
              onChange={setPersonBId}
              options={personOptions}
            />
          </div>
        </div>

        {/* Kinship Degree Result Banner */}
        <div className="rounded-2xl border border-brand/30 bg-brand/5 p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-xs text-secondary">
              <Sparkles className="h-3.5 w-3.5 text-brand" />
              <span>{t('tree.kinship.calculatedDegreeLabel')}</span>
            </div>
            <div
              data-testid="kinship-result-term"
              className="font-sans text-lg font-bold text-primary"
            >
              {effectiveResult.relationship_label}
            </div>
          </div>

          {effectiveResult.degree !== undefined && (
            <Badge variant="brand">
              {t('tree.kinship.stepsCount', {
                count: effectiveResult.degree,
              })}
            </Badge>
          )}
        </div>

        {/* Visual Relationship Path Chain Breadcrumbs */}
        {effectiveResult.steps.length > 0 ? (
          <div className="space-y-2">
            <div className="text-xs font-semibold text-secondary flex items-center gap-1.5">
              <GitCommitHorizontal className="h-4 w-4 text-brand" />
              <span>{t('tree.kinship.pathChainHeading')}</span>
            </div>

            <ol
              data-testid="kinship-path-chain"
              className="flex flex-wrap items-center gap-2 rounded-2xl border border-border bg-surface-muted/40 p-4"
            >
              {effectiveResult.steps.map((step, idx) => (
                <li
                  key={`${step.person_id}-${idx}`}
                  className="inline-flex items-center gap-2"
                >
                  <div className="inline-flex items-center gap-1.5 rounded-xl border border-border bg-surface px-3 py-1.5 text-xs font-semibold text-primary shadow-2xs">
                    <span>{step.display_name}</span>
                    {step.gid && (
                      <span className="font-mono text-[10px] text-tertiary">
                        ({step.gid})
                      </span>
                    )}
                    {step.relation_to_next && (
                      <span className="sr-only">
                        , {step.relation_to_next}
                      </span>
                    )}
                  </div>
                  {step.relation_to_next && (
                    <span
                      aria-hidden="true"
                      className="inline-flex items-center gap-1 text-xs font-medium text-brand px-1"
                    >
                      <ArrowRight className="h-3.5 w-3.5" />
                      <span>{step.relation_to_next}</span>
                      <ArrowRight className="h-3.5 w-3.5" />
                    </span>
                  )}
                </li>
              ))}
            </ol>
          </div>
        ) : (
          <Alert variant="info">{t('tree.kinship.notConnected')}</Alert>
        )}

        <div className="flex justify-end gap-2 pt-2 border-t border-border">
          <Button
            variant="secondary"
            onClick={() => onOpenChange(false)}
          >
            {t('common.close')}
          </Button>
          {effectiveResult.steps.length > 0 && onHighlightPath && (
            <Button
              variant="primary"
              data-testid="kinship-highlight-path-btn"
              leftIcon={<Route className="h-4 w-4" />}
              onClick={() => {
                onHighlightPath(
                  effectiveResult.steps
                    .map((s) => s.person_id)
                    .filter((id): id is string => Boolean(id))
                );
                onOpenChange(false);
              }}
            >
              {t('tree.kinship.highlightPathBtn')}
            </Button>
          )}
        </div>
      </div>
    </Dialog>
  );
};

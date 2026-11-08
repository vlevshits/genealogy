import React from 'react';
import {
  Calendar,
  ChevronRight,
  Heart,
  MapPin,
  UserPlus,
  Users,
} from 'lucide-react';
import { Avatar, Badge, Button, Card, EmptyState } from '../ui';
import {
  ProfileChildSummaryDto,
  ProfileFamiliesDto,
  ProfileRelativeSummaryDto,
  QuickAddRelation,
} from '../../api/client';
import { useI18n } from '../../i18n';
import {
  normalizePedigreeType,
  normalizeRelationshipType,
  normalizeSex,
} from '../../lib/domain-enums';

export interface ProfileFamilyTabProps {
  families: ProfileFamiliesDto;
  canEdit: boolean;
  accessTier?: string;
  onSelectRelative: (relativeId: string) => void;
  onQuickAdd: (relation: QuickAddRelation) => void;
}

export function formatRelativeDisplayName(
  rel: ProfileRelativeSummaryDto,
  accessTier: string = 'admin'
): { name: string; isMaskedLiving: boolean } {
  const isPublic = accessTier === 'public_anonymous';
  const rawGiven = (rel.given_name || rel.display_name || '').trim();
  const isMasked =
    rel.is_living &&
    (isPublic ||
      rawGiven === 'Living' ||
      /^[A-Za-zА-Яа-яЁё]\.$/.test(rawGiven));

  if (isMasked) {
    if (/^[A-Za-zА-Яа-яЁё]\.$/.test(rawGiven)) {
      return { name: rawGiven, isMaskedLiving: true };
    }
    const ch = (rawGiven.charAt(0) || 'L').toUpperCase();
    return { name: `${ch}.`, isMaskedLiving: true };
  }

  if (rel.display_name && rel.display_name.trim()) {
    return { name: rel.display_name.trim(), isMaskedLiving: false };
  }

  const full = [rel.given_name, rel.patronymic, rel.surname]
    .filter(Boolean)
    .join(' ');
  return { name: full || rel.gid, isMaskedLiving: false };
}

export const ProfileFamilyTab: React.FC<ProfileFamilyTabProps> = ({
  families,
  canEdit,
  accessTier = 'admin',
  onSelectRelative,
  onQuickAdd,
}) => {
  const { t } = useI18n();

  const renderRelativeCard = (
    rel: ProfileRelativeSummaryDto,
    roleBadgeLabel?: string,
    pedigreeRelation?: string
  ) => {
    const { name, isMaskedLiving } = formatRelativeDisplayName(rel, accessTier);
    const lifespanText = isMaskedLiving
      ? null
      : !rel.is_living
      ? `${rel.birth_year ?? '?'} – † ${rel.death_year ?? ''}`.trim()
      : rel.birth_year
      ? `${t('profile.hero.bornPrefix')} ${rel.birth_year}`
      : null;

    return (
      <div
        key={rel.id}
        onClick={() => onSelectRelative(rel.id)}
        className="flex items-center justify-between gap-3 rounded-2xl border border-border bg-surface p-3.5 hover:border-brand/60 hover:bg-surface-muted/40 transition-colors cursor-pointer"
      >
        <div className="flex items-center gap-3 min-w-0">
          <Avatar
            name={isMaskedLiving ? '?' : name}
            src={isMaskedLiving ? undefined : rel.avatar_url ?? undefined}
            size="md"
          />
          <div className="min-w-0 space-y-0.5">
            <div className="flex flex-wrap items-center gap-1.5">
              <span className="font-sans text-sm font-semibold text-primary truncate">
                {name}
              </span>
              {!rel.is_living && (
                <span
                  aria-label={t('person.quickAdd.livingNo')}
                  title={t('person.quickAdd.livingNo')}
                  className="inline-flex h-4 w-4 items-center justify-center rounded bg-surface-muted font-serif text-[11px] font-semibold text-secondary"
                >
                  †
                </span>
              )}
              {roleBadgeLabel && (
                <Badge variant="neutral" size="sm">
                  {roleBadgeLabel}
                </Badge>
              )}
              {pedigreeRelation && normalizePedigreeType(pedigreeRelation) !== 'birth' && (
                <Badge variant="brand" size="sm">
                  {t(`domain.pedigree.${normalizePedigreeType(pedigreeRelation)}`)}
                </Badge>
              )}
            </div>

            <div className="flex items-center gap-2 text-xs text-tertiary">
              <span className="font-mono">{rel.gid}</span>
              {lifespanText && <span>· {lifespanText}</span>}
            </div>
          </div>
        </div>

        <ChevronRight className="h-4 w-4 text-tertiary shrink-0" />
      </div>
    );
  };

  const parentFamilies = families.parent_families || [];
  const spouseFamilies = families.spouse_families || [];

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
      {/* Group 1: Parents & Siblings */}
      <Card className="p-5 sm:p-6 space-y-5">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Users className="h-4 w-4 text-brand shrink-0" />
            <h2 className="font-sans text-base font-semibold text-primary">
              {t('profile.family.parentsAndSiblingsTitle')}
            </h2>
          </div>

          {canEdit && (
            <div className="flex flex-wrap items-center gap-1.5">
              <Button
                size="sm"
                variant="secondary"
                leftIcon={<UserPlus className="h-3.5 w-3.5" />}
                onClick={() => onQuickAdd('father')}
              >
                {t('person.actions.addFather')}
              </Button>
              <Button
                size="sm"
                variant="secondary"
                leftIcon={<UserPlus className="h-3.5 w-3.5" />}
                onClick={() => onQuickAdd('mother')}
              >
                {t('person.actions.addMother')}
              </Button>
              <Button
                size="sm"
                variant="secondary"
                leftIcon={<UserPlus className="h-3.5 w-3.5" />}
                onClick={() => onQuickAdd('sibling')}
              >
                {t('person.actions.addSibling')}
              </Button>
            </div>
          )}
        </div>

        {parentFamilies.length === 0 ? (
          <EmptyState
            icon={<Users className="h-7 w-7 text-tertiary" />}
            title={t('profile.family.noParentsTitle')}
            description={t('profile.family.noParentsDesc')}
          />
        ) : (
          <div className="space-y-5">
            {parentFamilies.map((pf) => {
              const parentsList: Array<{
                rel: ProfileRelativeSummaryDto;
                roleLabel: string;
              }> = [];
              if (pf.father) {
                parentsList.push({
                  rel: pf.father,
                  roleLabel: t('profile.family.fatherRole'),
                });
              }
              if (pf.mother) {
                parentsList.push({
                  rel: pf.mother,
                  roleLabel: t('profile.family.motherRole'),
                });
              }
              if (
                parentsList.length === 0 &&
                pf.parents &&
                pf.parents.length > 0
              ) {
                for (const p of pf.parents) {
                  parentsList.push({
                    rel: p,
                    roleLabel:
                      normalizeSex(p.sex) === 'M'
                        ? t('profile.family.fatherRole')
                        : normalizeSex(p.sex) === 'F'
                        ? t('profile.family.motherRole')
                        : t('profile.family.parentRole'),
                  });
                }
              }

              return (
                <div
                  key={pf.family_id}
                  className="space-y-3 rounded-2xl border border-border/70 bg-surface-muted/30 p-4"
                >
                  <div className="flex items-center justify-between text-xs text-tertiary">
                    <span className="font-semibold uppercase tracking-wider text-secondary">
                      {t('profile.family.parentsSubheading')}
                    </span>
                    <span className="font-mono">{pf.gid}</span>
                  </div>

                  {parentsList.length > 0 ? (
                    <div className="space-y-2">
                      {parentsList.map(({ rel, roleLabel }) =>
                        renderRelativeCard(rel, roleLabel, pf.child_relation)
                      )}
                    </div>
                  ) : (
                    <p className="text-xs text-tertiary">
                      {t('profile.family.noParentsTitle')}
                    </p>
                  )}

                  {pf.siblings && pf.siblings.length > 0 && (
                    <div className="pt-2 space-y-2">
                      <div className="text-xs font-semibold uppercase tracking-wider text-secondary">
                        {t('profile.family.siblingsSubheading')} (
                        {pf.siblings.length})
                      </div>
                      <div className="space-y-2">
                        {pf.siblings.map((sib) =>
                          renderRelativeCard(
                            sib,
                            normalizeSex(sib.sex) === 'M'
                              ? t('profile.family.brotherRole')
                              : normalizeSex(sib.sex) === 'F'
                              ? t('profile.family.sisterRole')
                              : t('profile.family.siblingRole')
                          )
                        )}
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </Card>

      {/* Group 2: Spouses & Children */}
      <Card className="p-5 sm:p-6 space-y-5">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Heart className="h-4 w-4 text-accent shrink-0" />
            <h2 className="font-sans text-base font-semibold text-primary">
              {t('profile.family.spousesAndChildrenTitle')}
            </h2>
          </div>

          {canEdit && (
            <div className="flex flex-wrap items-center gap-1.5">
              <Button
                size="sm"
                variant="secondary"
                leftIcon={<UserPlus className="h-3.5 w-3.5" />}
                onClick={() => onQuickAdd('spouse')}
              >
                {t('person.actions.addSpouse')}
              </Button>
              <Button
                size="sm"
                variant="secondary"
                leftIcon={<UserPlus className="h-3.5 w-3.5" />}
                onClick={() => onQuickAdd('child')}
              >
                {t('person.actions.addChild')}
              </Button>
            </div>
          )}
        </div>

        {spouseFamilies.length === 0 ? (
          <EmptyState
            icon={<Heart className="h-7 w-7 text-tertiary" />}
            title={t('profile.family.noSpousesTitle')}
            description={t('profile.family.noSpousesDesc')}
          />
        ) : (
          <div className="space-y-5">
            {spouseFamilies.map((sf) => {
              const marriageDate =
                sf.marriage_date ?? sf.marriage_event?.date ?? null;
              const marriagePlace =
                sf.marriage_place ?? sf.marriage_event?.place ?? null;

              const resolveChildDto = (
                ch: ProfileChildSummaryDto
              ): {
                rel: ProfileRelativeSummaryDto | undefined;
                pedigree: string;
              } => {
                const rel = ch.person ?? ch.relative;
                const pedigree =
                  ch.rel_to_target ||
                  ch.rel_to_partner1 ||
                  ch.rel_to_partner2 ||
                  'birth';
                return { rel, pedigree };
              };

              return (
                <div
                  key={sf.family_id}
                  className="space-y-3 rounded-2xl border border-border/70 bg-surface-muted/30 p-4"
                >
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="flex items-center gap-2">
                      <Badge variant="brand" size="sm">
                        {t(`domain.relationships.${normalizeRelationshipType(sf.relationship_type)}`)}
                      </Badge>
                      {(marriageDate || marriagePlace) && (
                        <span className="inline-flex items-center gap-1.5 text-xs text-secondary">
                          {marriageDate && (
                            <span className="inline-flex items-center gap-1">
                              <Calendar className="h-3 w-3 text-tertiary" />
                              {marriageDate}
                            </span>
                          )}
                          {marriagePlace && (
                            <span className="inline-flex items-center gap-1">
                              <MapPin className="h-3 w-3 text-tertiary" />
                              {marriagePlace}
                            </span>
                          )}
                        </span>
                      )}
                    </div>
                    <span className="font-mono text-xs text-tertiary">
                      {sf.gid}
                    </span>
                  </div>

                  {sf.spouse ? (
                    renderRelativeCard(
                      sf.spouse,
                      t('profile.family.spouseRole')
                    )
                  ) : (
                    <div className="text-xs text-tertiary italic">
                      {t('profile.family.singleParentUnion')}
                    </div>
                  )}

                  {sf.children && sf.children.length > 0 && (
                    <div className="pt-2 space-y-2">
                      <div className="text-xs font-semibold uppercase tracking-wider text-secondary">
                        {t('profile.family.childrenSubheading')} (
                        {sf.children.length})
                      </div>
                      <div className="space-y-2">
                        {sf.children.map((ch) => {
                          const { rel, pedigree } = resolveChildDto(ch);
                          if (!rel) return null;
                          return renderRelativeCard(
                            rel,
                            rel.sex === 'M'
                              ? t('profile.family.sonRole')
                              : rel.sex === 'F'
                              ? t('profile.family.daughterRole')
                              : t('profile.family.childRole'),
                            pedigree
                          );
                        })}
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </Card>
    </div>
  );
};

import type {
  DuplicateCandidatePairDto,
  QualityWarningDto,
} from '../../api/client';

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
      reasons: ['shared_parents', 'patronymic_match', 'vital_dates', 'detached_record'],
      is_detached: true,
    },
  ];
}

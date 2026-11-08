import { EventType, PlaceType } from '../types/common';

/**
 * Normalizes any raw event_type string (including uppercase GEDCOM tags like
 * "BIRT", "DEAT", "MARR", "EVEN" or English aliases) into a canonical frontend
 * EventType key matching `domain.events.*`.
 */
export function normalizeEventType(raw?: string | null): EventType {
  const cleaned = (raw ?? '').trim().toLowerCase();
  switch (cleaned) {
    case 'birt':
    case 'birth':
    case 'born':
      return 'birth';
    case 'chr':
    case 'chri':
    case 'bapm':
    case 'bapt':
    case 'baptism':
    case 'christening':
      return 'baptism';
    case 'deat':
    case 'death':
    case 'died':
    case 'deceased':
      return 'death';
    case 'buri':
    case 'burial':
    case 'buried':
      return 'burial';
    case 'crem':
    case 'cremation':
      return 'cremation';
    case 'marr':
    case 'marriage':
    case 'married':
      return 'marriage';
    case 'marl':
    case 'marb':
    case 'marc':
    case 'marriage_license':
      return 'marriage_license';
    case 'enga':
    case 'engagement':
    case 'engaged':
      return 'engagement';
    case 'div':
    case 'divf':
    case 'divorce':
    case 'divorced':
    case 'anul':
      return 'divorce';
    case 'resi':
    case 'residence':
      return 'residence';
    case 'occu':
    case 'occupation':
      return 'occupation';
    case 'educ':
    case 'education':
      return 'education';
    case 'grad':
    case 'graduation':
      return 'graduation';
    case 'mili':
    case 'milt':
    case 'military':
    case 'military_service':
      return 'military';
    case 'emig':
    case 'emigration':
      return 'emigration';
    case 'immi':
    case 'immigration':
      return 'immigration';
    case 'natu':
    case 'naturalization':
      return 'naturalization';
    case 'cens':
    case 'census':
      return 'census';
    case 'reli':
    case 'religion':
      return 'religion';
    case 'adop':
    case 'adoption':
      return 'adoption';
    case 'reti':
    case 'retirement':
      return 'retirement';
    case 'even':
    case 'event':
    case 'fact':
    case 'custom':
    case 'other':
    case '':
      return 'custom';
    default:
      return cleaned as EventType;
  }
}

/**
 * Normalizes any raw parent-child pedigree relation string into a canonical
 * key matching `domain.pedigree.*`.
 */
export function normalizePedigreeType(
  raw?: string | null
): 'birth' | 'adopted' | 'foster' | 'step' | 'unknown' {
  const cleaned = (raw ?? '').trim().toLowerCase();
  switch (cleaned) {
    case 'birth':
    case 'biological':
    case 'natural':
    case 'bio':
    case 'birt':
    case '':
      return 'birth';
    case 'adop':
    case 'adopted':
    case 'adoption':
      return 'adopted';
    case 'foster':
    case 'fostered':
      return 'foster';
    case 'step':
    case 'stepchild':
      return 'step';
    default:
      return 'unknown';
  }
}

/**
 * Normalizes any raw family union relationship_type string into a canonical
 * key matching `domain.relationships.*`.
 */
export function normalizeRelationshipType(
  raw?: string | null
): 'married' | 'civil_union' | 'partners' | 'divorced' | 'engaged' | 'unknown' {
  const cleaned = (raw ?? '').trim().toLowerCase();
  switch (cleaned) {
    case 'marr':
    case 'married':
    case 'marriage':
    case 'spouse':
      return 'married';
    case 'civil':
    case 'civil_union':
    case 'unmarried':
      return 'civil_union';
    case 'partner':
    case 'partners':
      return 'partners';
    case 'div':
    case 'divorced':
    case 'divorce':
      return 'divorced';
    case 'enga':
    case 'engaged':
    case 'engagement':
      return 'engaged';
    default:
      return 'unknown';
  }
}

/**
 * Normalizes any raw place_type string into a canonical PlaceType key matching
 * `places.types.*`.
 */
export function normalizePlaceType(raw?: string | null): PlaceType {
  const cleaned = (raw ?? '').trim().toLowerCase();
  switch (cleaned) {
    case 'country':
    case 'nation':
      return 'country';
    case 'region':
    case 'state':
    case 'province':
    case 'oblast':
    case 'guberniya':
    case 'krai':
      return 'region';
    case 'district':
    case 'county':
    case 'uyezd':
    case 'raion':
    case 'volost':
      return 'district';
    case 'city':
    case 'town':
    case 'municipality':
      return 'city';
    case 'village':
    case 'hamlet':
    case 'settlement':
    case 'selo':
    case 'derevnya':
    case 'sloboda':
    case 'mestechko':
      return 'village';
    case 'address':
    case 'building':
    case 'church':
    case 'parish':
    case 'cemetery':
      return 'address';
    default:
      return 'other';
  }
}

/**
 * Normalizes any raw sex value into a canonical SexCode ('M' | 'F' | 'U').
 */
export function normalizeSex(raw?: string | null): 'M' | 'F' | 'U' {
  const cleaned = (raw ?? '').trim().toLowerCase();
  switch (cleaned) {
    case 'm':
    case 'male':
    case 'мужской':
    case 'м':
      return 'M';
    case 'f':
    case 'female':
    case 'женский':
    case 'ж':
      return 'F';
    default:
      return 'U';
  }
}

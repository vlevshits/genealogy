export type ControlSize = 'sm' | 'md' | 'lg';
export type Sex = 'male' | 'female' | 'other' | 'unknown';
export type RelationshipType = 'married' | 'civil_union' | 'partners' | 'unknown';
export type ChildRelation = 'birth' | 'adopted' | 'foster' | 'step' | 'unknown';
export type PlaceType =
  | 'country'
  | 'region'
  | 'district'
  | 'city'
  | 'village'
  | 'parish'
  | 'cemetery'
  | 'address'
  | 'other';

export type PersonEventType =
  | 'birth'
  | 'baptism'
  | 'death'
  | 'burial'
  | 'cremation'
  | 'residence'
  | 'occupation'
  | 'education'
  | 'military'
  | 'emigration'
  | 'immigration'
  | 'naturalization'
  | 'census'
  | 'religion'
  | 'custom';

export type FamilyEventType =
  | 'engagement'
  | 'marriage'
  | 'divorce'
  | 'residence'
  | 'custom';

export type EventType = PersonEventType | FamilyEventType;
export type UserRole = 'admin' | 'editor' | 'viewer' | 'guest';
export type ThemeMode = 'light' | 'dark' | 'system';
export type Locale = 'en' | 'ru';

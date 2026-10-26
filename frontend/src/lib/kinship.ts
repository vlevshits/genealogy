import { Locale, Sex } from '../types/common';

export type KinshipCode =
  | 'self'
  | 'parent'
  | 'child'
  | 'grandparent'
  | 'grandchild'
  | 'great_grandparent'
  | 'great_grandchild'
  | 'sibling'
  | 'spouse'
  | 'first_cousin'
  | 'second_cousin'
  | 'pibling'
  | 'great_pibling'
  | 'nibling'
  | 'cousin_once_removed'
  | 'parent_in_law_husband'
  | 'parent_in_law_wife';

interface KinshipEntry {
  en: { male: string; female: string; neutral: string };
  ru: { male: string; female: string; neutral: string };
}

const KINSHIP_DICTIONARY: Record<KinshipCode, KinshipEntry> = {
  self: {
    en: { male: 'Self (Home Person)', female: 'Self (Home Person)', neutral: 'Self (Home Person)' },
    ru: { male: 'Исходная персона', female: 'Исходная персона', neutral: 'Исходная персона' },
  },
  parent: {
    en: { male: 'Father', female: 'Mother', neutral: 'Parent' },
    ru: { male: 'Отец', female: 'Мать', neutral: 'Родитель' },
  },
  child: {
    en: { male: 'Son', female: 'Daughter', neutral: 'Child' },
    ru: { male: 'Сын', female: 'Дочь', neutral: 'Ребёнок' },
  },
  grandparent: {
    en: { male: 'Grandfather', female: 'Grandmother', neutral: 'Grandparent' },
    ru: { male: 'Дедушка', female: 'Бабушка', neutral: 'Дедушка / Бабушка' },
  },
  grandchild: {
    en: { male: 'Grandson', female: 'Granddaughter', neutral: 'Grandchild' },
    ru: { male: 'Внук', female: 'Внучка', neutral: 'Внук / Внучка' },
  },
  great_grandparent: {
    en: { male: 'Great-Grandfather', female: 'Great-Grandmother', neutral: 'Great-Grandparent' },
    ru: { male: 'Прадедушка', female: 'Прабабушка', neutral: 'Прадедушка / Прабабушка' },
  },
  great_grandchild: {
    en: { male: 'Great-Grandson', female: 'Great-Granddaughter', neutral: 'Great-Grandchild' },
    ru: { male: 'Правнук', female: 'Правнучка', neutral: 'Правнук / Правнучка' },
  },
  sibling: {
    en: { male: 'Brother', female: 'Sister', neutral: 'Sibling' },
    ru: { male: 'Брат', female: 'Сестра', neutral: 'Брат / Сестра' },
  },
  spouse: {
    en: { male: 'Husband', female: 'Wife', neutral: 'Spouse / Partner' },
    ru: { male: 'Муж', female: 'Жена', neutral: 'Супруг(а)' },
  },
  first_cousin: {
    en: { male: 'First Cousin', female: 'First Cousin', neutral: 'First Cousin' },
    ru: { male: 'Двоюродный брат', female: 'Двоюродная сестра', neutral: 'Двоюродный брат / сестра' },
  },
  second_cousin: {
    en: { male: 'Second Cousin', female: 'Second Cousin', neutral: 'Second Cousin' },
    ru: { male: 'Троюродный брат', female: 'Троюродная сестра', neutral: 'Троюродный брат / сестра' },
  },
  pibling: {
    en: { male: 'Uncle', female: 'Aunt', neutral: 'Uncle / Aunt' },
    ru: { male: 'Дядя', female: 'Тётя', neutral: 'Дядя / Тётя' },
  },
  great_pibling: {
    en: { male: 'Great-Uncle', female: 'Great-Aunt', neutral: 'Great-Uncle / Great-Aunt' },
    ru: { male: 'Двоюродный дедушка', female: 'Двоюродная бабушка', neutral: 'Двоюродный дедушка / бабушка' },
  },
  nibling: {
    en: { male: 'Nephew', female: 'Niece', neutral: 'Nephew / Niece' },
    ru: { male: 'Племянник', female: 'Племянница', neutral: 'Племянник / Племянница' },
  },
  cousin_once_removed: {
    en: {
      male: 'First cousin once removed',
      female: 'First cousin once removed',
      neutral: 'First cousin once removed',
    },
    ru: {
      male: 'Двоюродный племянник',
      female: 'Двоюродная тётя',
      neutral: 'Двоюродный родственник',
    },
  },
  parent_in_law_husband: {
    en: { male: 'Father-in-law', female: 'Mother-in-law', neutral: 'Parent-in-law' },
    ru: { male: 'Свёкор', female: 'Свекровь', neutral: 'Родитель мужа' },
  },
  parent_in_law_wife: {
    en: { male: 'Father-in-law', female: 'Mother-in-law', neutral: 'Parent-in-law' },
    ru: { male: 'Тесть', female: 'Тёща', neutral: 'Родитель жены' },
  },
};

export function formatKinshipLabel(
  code: KinshipCode,
  sex: Sex = 'unknown',
  locale: Locale = 'en'
): string {
  const entry = KINSHIP_DICTIONARY[code];
  if (!entry) return code;
  const langGroup = entry[locale] ?? entry.en;
  if (sex === 'male') return langGroup.male;
  if (sex === 'female') return langGroup.female;
  return langGroup.neutral;
}

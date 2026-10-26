import { Individual } from '../types/person';
import { Place } from '../types/place';
import { LifeEvent } from '../types/event';
import { Sex } from '../types/common';

export const SAMPLE_PLACES: Place[] = [
  {
    id: 'p-1',
    gid: 'P0001',
    name: 'Санкт-Петербург',
    place_type: 'city',
    hierarchy_names: ['Санкт-Петербург', 'Санкт-Петербургская губерния', 'Российская империя'],
    latitude: 59.9343,
    longitude: 30.3351,
    events_count: 42,
    version: 1,
  },
  {
    id: 'p-2',
    gid: 'P0002',
    name: 'Тверь',
    place_type: 'city',
    hierarchy_names: ['Тверь', 'Тверская губерния', 'Российская империя'],
    latitude: 56.8587,
    longitude: 35.9176,
    events_count: 19,
    version: 1,
  },
  {
    id: 'p-3',
    gid: 'P0003',
    name: 'Москва',
    place_type: 'city',
    hierarchy_names: ['Москва', 'Московская область', 'Россия'],
    latitude: 55.7558,
    longitude: 37.6173,
    events_count: 68,
    version: 1,
  },
  {
    id: 'p-4',
    gid: 'P0004',
    name: 'Витебск',
    place_type: 'city',
    hierarchy_names: ['Витебск', 'Витебская губерния', 'Российская империя'],
    latitude: 55.1904,
    longitude: 30.2049,
    events_count: 15,
    version: 1,
  },
  {
    id: 'p-5',
    gid: 'P0005',
    name: 'Одесса',
    place_type: 'city',
    hierarchy_names: ['Одесса', 'Херсонская губерния', 'Российская империя'],
    latitude: 46.4825,
    longitude: 30.7233,
    events_count: 11,
    version: 1,
  },
];

export const SAMPLE_PEOPLE: Individual[] = [
  {
    id: 'ind-1',
    gid: 'I0001',
    given_name: 'Александр',
    patronymic: 'Петрович',
    surname: 'Левшиц',
    sex: 'male',
    is_living: false,
    is_private: false,
    birth_date: {
      mod: 'exact',
      start: { y: 1912, m: 7, d: 14 },
      end: null,
      cal: 'julian',
      text: '14.07.1912 (ст. ст.)',
    },
    death_date: {
      mod: 'exact',
      start: { y: 1989, m: 11, d: 3 },
      end: null,
      cal: 'gregorian',
      text: '03.11.1989',
    },
    birth_sort: 19120727,
    death_sort: 19891103,
    birth_place: 'Витебск, Витебская губерния',
    death_place: 'Ленинград',
    biography:
      'Инженер-мостостроитель, выпускник ЛИИЖТ. Прошёл Великую Отечественную войну в составе инженерно-сапёрного батальона, награждён орденом Красной Звезды.',
    version: 3,
  },
  {
    id: 'ind-2',
    gid: 'I0002',
    given_name: 'Вера',
    patronymic: 'Ивановна',
    surname: 'Левшиц',
    birth_surname: 'Морозова',
    sex: 'female',
    is_living: false,
    is_private: false,
    birth_date: {
      mod: 'about',
      start: { y: 1916, m: null, d: null },
      end: null,
      cal: 'gregorian',
      text: 'ок. 1916',
    },
    death_date: {
      mod: 'exact',
      start: { y: 2001, m: 4, d: 19 },
      end: null,
      cal: 'gregorian',
      text: '19.04.2001',
    },
    birth_sort: 19160000,
    death_sort: 20010419,
    birth_place: 'Тверь, Тверская губерния',
    death_place: 'Санкт-Петербург',
    version: 2,
  },
  {
    id: 'ind-3',
    gid: 'I0003',
    given_name: 'Пётр',
    patronymic: 'Фёдорович',
    surname: 'Щеглов',
    sex: 'male',
    is_living: false,
    is_private: false,
    birth_date: {
      mod: 'before',
      start: { y: 1868, m: 5, d: null },
      end: null,
      cal: 'julian',
      text: 'до мая 1868',
    },
    death_date: {
      mod: 'between',
      start: { y: 1930, m: null, d: null },
      end: { y: 1932, m: null, d: null },
      cal: 'gregorian',
      text: 'между 1930 и 1932 гг.',
    },
    birth_sort: 18680499,
    death_sort: 19300000,
    birth_place: 'Тверь, Тверская губерния',
    version: 1,
  },
  {
    id: 'ind-4',
    gid: 'I0004',
    given_name: 'Елена',
    patronymic: 'Александровна',
    surname: 'Кузнецова',
    birth_surname: 'Левшиц',
    sex: 'female',
    is_living: true,
    is_private: false,
    birth_date: {
      mod: 'exact',
      start: { y: 1948, m: 9, d: 22 },
      end: null,
      cal: 'gregorian',
      text: '22.09.1948',
    },
    birth_sort: 19480922,
    birth_place: 'Ленинград',
    version: 1,
  },
  {
    id: 'ind-5',
    gid: 'I0005',
    given_name: 'Михаил',
    patronymic: 'Сергеевич',
    surname: 'Кузнецов',
    sex: 'male',
    is_living: true,
    is_private: true,
    is_redacted: true,
    birth_date: {
      mod: 'exact',
      start: { y: 1978, m: 3, d: 11 },
      end: null,
      cal: 'gregorian',
      text: '11.03.1978',
    },
    birth_sort: 19780311,
    birth_place: 'Москва',
    version: 1,
  },
];

export const SAMPLE_EVENTS: LifeEvent[] = [
  {
    id: 'ev-1',
    gid: 'E0001',
    individual_id: 'ind-1',
    event_type: 'birth',
    date: SAMPLE_PEOPLE[0].birth_date,
    date_sort: 19120727,
    place: SAMPLE_PLACES[3],
    description: 'Родился в семье ремесленника Петра Яковлевича Левшица',
    source_text: 'Метрическая книга Витебской синагоги, 1912 г., л. 34 об.',
    has_scan: true,
    scan_count: 1,
    is_private: false,
    age_at_event: 0,
    version: 1,
  },
  {
    id: 'ev-2',
    gid: 'E0002',
    individual_id: 'ind-1',
    event_type: 'education',
    date: {
      mod: 'exact',
      start: { y: 1935, m: 6, d: null },
      end: null,
      cal: 'gregorian',
      text: 'июнь 1935 г.',
    },
    date_sort: 19350600,
    place: SAMPLE_PLACES[0],
    description: 'Окончил Ленинградский институт инженеров железнодорожного транспорта (ЛИИЖТ)',
    source_text: 'Диплом № 14829, личный архив семьи',
    has_scan: true,
    scan_count: 2,
    is_private: false,
    age_at_event: 22,
    version: 1,
  },
  {
    id: 'ev-3',
    gid: 'E0003',
    individual_id: 'ind-1',
    event_type: 'military',
    date: {
      mod: 'between',
      start: { y: 1941, m: null, d: null },
      end: { y: 1945, m: null, d: null },
      cal: 'gregorian',
      text: 'между 1941 и 1945 гг.',
    },
    date_sort: 19410000,
    place: SAMPLE_PLACES[0],
    description: 'Служба в 42-м отдельном инженерно-сапёрном батальоне Ленинградского фронта',
    source_text: 'ЦАМО РФ, ф. 33, оп. 686044, д. 1912',
    has_scan: false,
    is_private: false,
    age_at_event: 29,
    version: 1,
  },
  {
    id: 'ev-4',
    gid: 'E0004',
    individual_id: 'ind-1',
    event_type: 'death',
    date: SAMPLE_PEOPLE[0].death_date,
    date_sort: 19891103,
    place: SAMPLE_PLACES[0],
    description: 'Похоронен на Преображенском кладбище, участок 14',
    source_text: 'Свидетельство о смерти II-АК № 409182',
    has_scan: true,
    scan_count: 1,
    is_private: false,
    age_at_event: 77,
    version: 1,
  },
];

const FIRST_NAMES_M = ['Александр', 'Пётр', 'Иван', 'Михаил', 'Николай', 'Сергей', 'Дмитрий', 'Фёдор', 'Яков', 'Лев'];
const FIRST_NAMES_F = ['Вера', 'Елена', 'Анна', 'Мария', 'Ольга', 'Софья', 'Екатерина', 'Наталья', 'Татьяна', 'Дарья'];
const PATRONYMICS_M = ['Александрович', 'Петрович', 'Иванович', 'Михайлович', 'Николаевич', 'Сергеевич', 'Фёдорович', 'Яковлевич'];
const PATRONYMICS_F = ['Александровна', 'Петровна', 'Ивановна', 'Михайловна', 'Николаевна', 'Сергеевна', 'Фёдоровна', 'Яковлевна'];
const SURNAMES_M = ['Левшиц', 'Кузнецов', 'Морозов', 'Щеглов', 'Соколов', 'Волков', 'Попов', 'Лебедев', 'Новиков', 'Орлов'];
const SURNAMES_F = ['Левшиц', 'Кузнецова', 'Морозова', 'Щеглова', 'Соколова', 'Волкова', 'Попова', 'Лебедева', 'Новикова', 'Орлова'];
const CITIES = ['Санкт-Петербург', 'Тверь', 'Москва', 'Витебск', 'Одесса', 'Казань', 'Нижний Новгород', 'Ярославль'];

export interface DirectoryPersonRow {
  id: string;
  gid: string;
  given_name: string;
  patronymic: string;
  surname: string;
  birth_surname?: string;
  sex: Sex;
  birth_year: number;
  death_year: number | null;
  birth_place: string;
  is_living: boolean;
  events_count: number;
}

let cached50kRows: DirectoryPersonRow[] | null = null;

export function generateLargePeopleDataset(count = 50000): DirectoryPersonRow[] {
  if (cached50kRows && cached50kRows.length === count) {
    return cached50kRows;
  }
  const rows: DirectoryPersonRow[] = new Array(count);
  for (let i = 0; i < count; i++) {
    const isFemale = i % 2 === 1;
    const sex: Sex = isFemale ? 'female' : 'male';
    const given_name = isFemale
      ? FIRST_NAMES_F[i % FIRST_NAMES_F.length]
      : FIRST_NAMES_M[i % FIRST_NAMES_M.length];
    const patronymic = isFemale
      ? PATRONYMICS_F[(i * 3) % PATRONYMICS_F.length]
      : PATRONYMICS_M[(i * 3) % PATRONYMICS_M.length];
    const surname = isFemale
      ? SURNAMES_F[(i * 7) % SURNAMES_F.length]
      : SURNAMES_M[(i * 7) % SURNAMES_M.length];
    const birth_year = 1780 + ((i * 13) % 235);
    const is_living = birth_year >= 1945;
    const death_year = is_living ? null : Math.min(2024, birth_year + 52 + (i % 35));
    const birth_place = CITIES[i % CITIES.length];
    rows[i] = {
      id: `person-${i + 1}`,
      gid: `I${String(i + 1).padStart(5, '0')}`,
      given_name,
      patronymic,
      surname,
      birth_surname: isFemale && i % 4 === 1 ? SURNAMES_F[(i + 2) % SURNAMES_F.length] : undefined,
      sex,
      birth_year,
      death_year,
      birth_place,
      is_living,
      events_count: 2 + (i % 6),
    };
  }
  if (count === 50000) {
    cached50kRows = rows;
  }
  return rows;
}

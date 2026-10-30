import { describe, it, expect } from 'vitest';
import en from '../../src/i18n/en.json';
import ru from '../../src/i18n/ru.json';
import { formatKinshipLabel } from '../../src/lib/kinship';
import { matchesCrossScript } from '../../src/lib/utils';
import {
  formatLifeSpanText,
  formatPersonName,
  formatVitalEventLabel,
} from '../../src/lib/formatters';
import { ApiError } from '../../src/api/client';
import { getLocalizedErrorMessage } from '../../src/lib/error-formatter';
import { translate } from '../../src/i18n';

function collectKeys(obj: Record<string, unknown>, prefix = ''): string[] {
  const keys: string[] = [];
  for (const [k, v] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${k}` : k;
    if (v && typeof v === 'object' && !Array.isArray(v)) {
      keys.push(...collectKeys(v as Record<string, unknown>, path));
    } else {
      keys.push(path);
    }
  }
  return keys.sort();
}

describe('i18n Key Parity & Localization Helpers', () => {
  it('maintains 100% key parity between en.json and ru.json', () => {
    const enKeys = collectKeys(en as Record<string, unknown>);
    const ruKeys = collectKeys(ru as Record<string, unknown>);
    expect(enKeys).toEqual(ruKeys);
    expect(enKeys.length).toBeGreaterThan(100);
  });

  it('has non-empty string values for every key in both en.json and ru.json', () => {
    const checkValues = (obj: Record<string, unknown>, locale: string) => {
      for (const [k, v] of Object.entries(obj)) {
        if (v && typeof v === 'object') {
          checkValues(v as Record<string, unknown>, locale);
        } else {
          expect(typeof v, `${locale}.${k} should be a string`).toBe('string');
          expect(
            (v as string).trim().length,
            `${locale}.${k} should not be empty`
          ).toBeGreaterThan(0);
        }
      }
    };
    checkValues(en as Record<string, unknown>, 'en');
    checkValues(ru as Record<string, unknown>, 'ru');
  });

  it('formats kinship terms with gender agreement in Russian and English', () => {
    expect(formatKinshipLabel('first_cousin', 'male', 'ru')).toBe('Двоюродный брат');
    expect(formatKinshipLabel('first_cousin', 'female', 'ru')).toBe('Двоюродная сестра');
    expect(formatKinshipLabel('first_cousin', 'female', 'en')).toBe('First Cousin');
    expect(formatKinshipLabel('second_cousin', 'female', 'ru')).toBe('Троюродная сестра');
    expect(formatKinshipLabel('parent_in_law_husband', 'female', 'ru')).toBe('Свекровь');
    expect(formatKinshipLabel('parent_in_law_wife', 'female', 'ru')).toBe('Тёща');
  });

  it('matches cross-script Cyrillic and Latin queries for PersonPicker', () => {
    expect(matchesCrossScript('Левшиц Александр Петрович', 'levshits')).toBe(true);
    expect(matchesCrossScript('Кузнецова Елена Александровна', 'kuznetsova elena')).toBe(true);
    expect(matchesCrossScript('Щеглов Пётр Фёдорович', 'shcheglov')).toBe(true);
  });

  it('contains zero forbidden marketing buzzwords or developer tech-stack names in en.json and ru.json', () => {
    const forbiddenTerms = [
      'Next-Gen',
      'Zero-Cloud',
      'Платформа нового поколения',
      'Платформа семейного архива нового поколения',
      'Нулевая зависимость от облака',
      'Полная автономность без облаков',
      'Axum',
      'React 19',
      'Tailwind',
      'PostgreSQL',
      'pg_trgm',
      'Swagger',
    ];

    const enRaw = JSON.stringify(en);
    const ruRaw = JSON.stringify(ru);

    for (const term of forbiddenTerms) {
      expect(enRaw, `en.json should not contain forbidden term "${term}"`).not.toContain(term);
      expect(ruRaw, `ru.json should not contain forbidden term "${term}"`).not.toContain(term);
    }
  });

  it('forbids leading icon symbols (+, arrows, checkmarks) or decorative emojis in en.json and ru.json that cause duplicated UI icons', () => {
    const leadingIconRegex = /^\s*[+＋←→↑↓↔✓✔✕✖✗✘]\s+/;
    const emojiRegex = /[\u{1F300}-\u{1FAFF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]/u;

    const checkNoIconPrefixes = (
      obj: Record<string, unknown>,
      locale: string,
      prefix = ''
    ) => {
      for (const [k, v] of Object.entries(obj)) {
        const keyPath = prefix ? `${prefix}.${k}` : k;
        if (v && typeof v === 'object') {
          checkNoIconPrefixes(v as Record<string, unknown>, locale, keyPath);
        } else if (typeof v === 'string') {
          expect(
            leadingIconRegex.test(v),
            `${locale}.${keyPath} ("${v}") must not start with an icon symbol`
          ).toBe(false);
          expect(
            emojiRegex.test(v),
            `${locale}.${keyPath} ("${v}") must not contain decorative UI emojis`
          ).toBe(false);
        }
      }
    };

    checkNoIconPrefixes(en as Record<string, unknown>, 'en');
    checkNoIconPrefixes(ru as Record<string, unknown>, 'ru');
  });

  it('formats Slavic tripartite names and maiden surnames per locale rules', () => {
    const malePerson = {
      given_name: 'Александр',
      patronymic: 'Иванович',
      surname: 'Кузнецов',
    };
    expect(formatPersonName(malePerson, { locale: 'ru', format: 'full' })).toBe(
      'Кузнецов Александр Иванович'
    );
    expect(formatPersonName(malePerson, { locale: 'ru', format: 'compact' })).toBe(
      'Александр Кузнецов'
    );

    const femalePersonRu = {
      given_name: 'Вера',
      patronymic: 'Ивановна',
      surname: 'Левшиц',
      birth_surname: 'Морозова',
    };
    expect(
      formatPersonName(femalePersonRu, { locale: 'ru', format: 'full' })
    ).toBe('Левшиц (Морозова) Вера Ивановна');

    const femalePersonEn = {
      given_name: 'Vera',
      patronymic: 'Ivanovna',
      surname: 'Levshits',
      birth_surname: 'Morozova',
    };
    expect(
      formatPersonName(femalePersonEn, { locale: 'en', format: 'full' })
    ).toBe('Vera Ivanovna Levshits (née Morozova)');
    expect(
      formatPersonName(femalePersonEn, { locale: 'en', format: 'directory' })
    ).toBe('Levshits (née Morozova), Vera Ivanovna');
  });

  it('agrees Russian vital event verbs with biological sex', () => {
    expect(formatVitalEventLabel('birth', 'M', { locale: 'ru' })).toBe('родился');
    expect(formatVitalEventLabel('birth', 'F', { locale: 'ru' })).toBe('родилась');
    expect(formatVitalEventLabel('birth', 'U', { locale: 'ru' })).toBe('род.');
    expect(formatVitalEventLabel('death', 'M', { locale: 'ru' })).toBe('умер');
    expect(formatVitalEventLabel('death', 'F', { locale: 'ru' })).toBe('умерла');
    expect(formatVitalEventLabel('baptism', 'M', { locale: 'ru' })).toBe('крещён');
    expect(formatVitalEventLabel('baptism', 'F', { locale: 'ru' })).toBe('крещена');
    expect(formatVitalEventLabel('burial', 'M', { locale: 'ru' })).toBe('похоронен');
    expect(formatVitalEventLabel('burial', 'F', { locale: 'ru' })).toBe('похоронена');
  });

  it('formats lifespans with dagger † for deceased and zero Living text for living', () => {
    expect(
      formatLifeSpanText({
        birthYear: 1994,
        isLiving: true,
        locale: 'en',
      })
    ).toBe('b. 1994');
    expect(
      formatLifeSpanText({
        birthYear: 1994,
        isLiving: true,
        locale: 'ru',
      })
    ).toBe('р. 1994');
    expect(
      formatLifeSpanText({
        birthYear: 1912,
        deathYear: 1985,
        isLiving: false,
        locale: 'en',
      })
    ).toBe('1912 – †1985');
    expect(
      formatLifeSpanText({
        isLiving: true,
        isRedacted: true,
        locale: 'en',
      })
    ).toBe('');
  });

  it('localizes server ApiError codes, HTTP status fallbacks, and browser network errors in en and ru', () => {
    const tRu = (path: string, params?: Record<string, string | number>) =>
      translate('ru', path, params);
    const tEn = (path: string, params?: Record<string, string | number>) =>
      translate('en', path, params);

    const netErr = new ApiError(0, {
      code: 'network_error',
      message: 'Network request failed',
    });
    expect(getLocalizedErrorMessage(netErr, tRu)).toBe(
      'Не удалось связаться с сервером. Проверьте сетевое подключение.'
    );
    expect(getLocalizedErrorMessage(netErr, tEn)).toBe(
      'Failed to connect to server. Check your network connection.'
    );

    const rawFetchError = new TypeError('Failed to fetch');
    expect(getLocalizedErrorMessage(rawFetchError, tRu)).toBe(
      'Не удалось связаться с сервером. Проверьте сетевое подключение.'
    );

    const invalidCredsErr = new ApiError(401, {
      code: 'invalid_credentials',
      message: 'Invalid credentials',
    });
    expect(getLocalizedErrorMessage(invalidCredsErr, tRu)).toBe(
      'Неверный адрес электронной почты или пароль.'
    );

    const unmapped403 = new ApiError(403, {
      code: 'custom_unmapped_code',
      message: 'Forbidden: viewer cannot edit',
    });
    expect(getLocalizedErrorMessage(unmapped403, tRu)).toBe(
      'У вас нет прав для выполнения этого действия.'
    );
  });

  it('localizes WebAuthn DOMExceptions and security verification errors in en and ru', () => {
    const tRu = (path: string, params?: Record<string, string | number>) =>
      translate('ru', path, params);
    const tEn = (path: string, params?: Record<string, string | number>) =>
      translate('en', path, params);

    // DOMException: NotAllowedError (user cancelled / timed out)
    const notAllowed = new DOMException('The operation was aborted', 'NotAllowedError');
    expect(getLocalizedErrorMessage(notAllowed, tEn)).toBe(
      'Biometric verification was cancelled or timed out. Please try again.'
    );
    expect(getLocalizedErrorMessage(notAllowed, tRu)).toBe(
      'Биометрическая верификация отменена или истекло время ожидания. Попробуйте снова.'
    );

    // DOMException: InvalidStateError (already registered)
    const invalidState = new DOMException('Credential already registered', 'InvalidStateError');
    expect(getLocalizedErrorMessage(invalidState, tEn)).toBe(
      'This passkey is already registered on this device.'
    );
    expect(getLocalizedErrorMessage(invalidState, tRu)).toBe(
      'Этот ключ доступа уже зарегистрирован на этом устройстве.'
    );

    // DOMException: SecurityError (insecure origin / HTTPS required)
    const securityErr = new DOMException('Insecure context', 'SecurityError');
    expect(getLocalizedErrorMessage(securityErr, tEn)).toBe(
      'Passkeys require a secure connection (HTTPS or localhost) and a matching domain.'
    );
    expect(getLocalizedErrorMessage(securityErr, tRu)).toBe(
      'Для ключей доступа требуется защищенное соединение (HTTPS или localhost) и совпадение домена.'
    );

    // DOMException: NotSupportedError (platform authenticator missing)
    const notSupported = new DOMException('Not supported', 'NotSupportedError');
    expect(getLocalizedErrorMessage(notSupported, tEn)).toBe(
      'Your browser or device does not support passkey authentication.'
    );
    expect(getLocalizedErrorMessage(notSupported, tRu)).toBe(
      'Ваш браузер или устройство не поддерживает аутентификацию по ключу доступа.'
    );

    // ApiError: user_verification_required
    const uvErr = new ApiError(400, {
      code: 'user_verification_required',
      message: 'User verification required',
    });
    expect(getLocalizedErrorMessage(uvErr, tEn)).toBe(
      'User verification (biometrics or device PIN) is required.'
    );
    expect(getLocalizedErrorMessage(uvErr, tRu)).toBe(
      'Требуется подтверждение пользователя (биометрия или PIN-код устройства).'
    );

    // ApiError: passkey_verification_failed
    const verifyFailedErr = new ApiError(400, {
      code: 'passkey_verification_failed',
      message: 'Passkey assertion verification failed',
    });
    expect(getLocalizedErrorMessage(verifyFailedErr, tEn)).toBe(
      'Passkey verification failed. Please try again.'
    );
    expect(getLocalizedErrorMessage(verifyFailedErr, tRu)).toBe(
      'Ошибка проверки ключа доступа. Попробуйте снова.'
    );

    // ApiError: passkey_replay_detected
    const replayErr = new ApiError(401, {
      code: 'passkey_replay_detected',
      message: 'Passkey signature counter replay detected',
    });
    expect(getLocalizedErrorMessage(replayErr, tEn)).toBe(
      'Passkey security verification failed. Please try signing in again.'
    );
    expect(getLocalizedErrorMessage(replayErr, tRu)).toBe(
      'Проверка безопасности ключа доступа не удалась. Попробуйте войти снова.'
    );

    // DOMException: TimeoutError
    const timeoutErr = new DOMException('Operation timed out', 'TimeoutError');
    expect(getLocalizedErrorMessage(timeoutErr, tEn)).toBe(
      'Biometric verification was cancelled or timed out. Please try again.'
    );
    expect(getLocalizedErrorMessage(timeoutErr, tRu)).toBe(
      'Биометрическая верификация отменена или истекло время ожидания. Попробуйте снова.'
    );

    // DOMException: AbortError
    const abortErr = new DOMException('Registration aborted', 'AbortError');
    expect(getLocalizedErrorMessage(abortErr, tEn)).toBe(
      'Biometric verification was cancelled or timed out. Please try again.'
    );
    expect(getLocalizedErrorMessage(abortErr, tRu)).toBe(
      'Биометрическая верификация отменена или истекло время ожидания. Попробуйте снова.'
    );
  });

  it('Milestone 2.6: tree.connection.* keys have 100% EN/RU parity and zero glyphs or emojis', () => {
    const enConn = (en as Record<string, any>).tree?.connection;
    const ruConn = (ru as Record<string, any>).tree?.connection;

    expect(enConn, 'en.json must define tree.connection').toBeDefined();
    expect(ruConn, 'ru.json must define tree.connection').toBeDefined();

    const enConnKeys = Object.keys(enConn).sort();
    const ruConnKeys = Object.keys(ruConn).sort();
    expect(enConnKeys).toEqual(ruConnKeys);

    // Required connection keys per spec
    const requiredKeys = [
      'marriage',
      'parent_child',
      'jump_to',
      'find_kinship',
      'close',
      'select_hint',
      'relationship_type',
      'view_person',
      'father',
      'mother',
      'parent',
      'son',
      'daughter',
      'child',
      'spouse',
    ];
    for (const rk of requiredKeys) {
      expect(enConnKeys).toContain(rk);
      expect(ruConnKeys).toContain(rk);
    }

    const leadingIconRegex = /^\s*[+＋←→↑↓↔✓✔✕✖✗✘]\s+/;
    const emojiRegex = /[\u{1F300}-\u{1FAFF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]/u;

    for (const key of enConnKeys) {
      const enVal = String(enConn[key]);
      const ruVal = String(ruConn[key]);

      expect(enVal.trim().length, `en tree.connection.${key} must not be empty`).toBeGreaterThan(0);
      expect(ruVal.trim().length, `ru tree.connection.${key} must not be empty`).toBeGreaterThan(0);

      expect(enVal, `en tree.connection.${key} must not have leading icon`).not.toMatch(leadingIconRegex);
      expect(ruVal, `ru tree.connection.${key} must not have leading icon`).not.toMatch(leadingIconRegex);

      expect(enVal, `en tree.connection.${key} must not have emoji`).not.toMatch(emojiRegex);
      expect(ruVal, `ru tree.connection.${key} must not have emoji`).not.toMatch(emojiRegex);
    }
  });

  it('Milestone 2.7: tree.kinship.* keys have 100% EN/RU parity and zero glyphs or emojis', () => {
    const enKinship = (en as Record<string, any>).tree?.kinship;
    const ruKinship = (ru as Record<string, any>).tree?.kinship;

    expect(enKinship, 'en.json must define tree.kinship').toBeDefined();
    expect(ruKinship, 'ru.json must define tree.kinship').toBeDefined();

    const enKinshipKeys = Object.keys(enKinship).sort();
    const ruKinshipKeys = Object.keys(ruKinship).sort();
    expect(enKinshipKeys).toEqual(ruKinshipKeys);

    // Required Milestone 2.7 kinship keys per spec
    const requiredKeys = [
      'openBtn',
      'title',
      'subtitle',
      'personALabel',
      'personBLabel',
      'calculatedDegreeLabel',
      'stepsCount',
      'pathChainHeading',
      'highlightPathBtn',
      'notConnected',
      'samePerson',
      'tracesCount',
      'traceTab',
      'highlightSelected',
      'highlightAll',
      'clearHighlights',
      'legendTitle',
      'bloodRelation',
      'affinalRelation',
    ];
    for (const rk of requiredKeys) {
      expect(enKinshipKeys).toContain(rk);
      expect(ruKinshipKeys).toContain(rk);
    }

    const leadingIconRegex = /^\s*[+＋←→↑↓↔✓✔✕✖✗✘]\s+/;
    const emojiRegex = /[\u{1F300}-\u{1FAFF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]/u;

    for (const key of enKinshipKeys) {
      const enVal = String(enKinship[key]);
      const ruVal = String(ruKinship[key]);

      expect(enVal.trim().length, `en tree.kinship.${key} must not be empty`).toBeGreaterThan(0);
      expect(ruVal.trim().length, `ru tree.kinship.${key} must not be empty`).toBeGreaterThan(0);

      expect(enVal, `en tree.kinship.${key} must not have leading icon`).not.toMatch(leadingIconRegex);
      expect(ruVal, `ru tree.kinship.${key} must not have leading icon`).not.toMatch(leadingIconRegex);

      expect(enVal, `en tree.kinship.${key} must not have emoji`).not.toMatch(emojiRegex);
      expect(ruVal, `ru tree.kinship.${key} must not have emoji`).not.toMatch(emojiRegex);
    }
  });

  it('Milestone 2.8: mediaArchive.* associated people and face detection keys have 100% EN/RU parity and zero glyphs or emojis', () => {
    const enMedia = (en as Record<string, any>).mediaArchive;
    const ruMedia = (ru as Record<string, any>).mediaArchive;

    expect(enMedia, 'en.json must define mediaArchive').toBeDefined();
    expect(ruMedia, 'ru.json must define mediaArchive').toBeDefined();

    const requiredKeys = [
      'linkPersonTitle',
      'linkPersonSubtitle',
      'linkPersonSearchPlaceholder',
      'linkPersonEmpty',
      'linkPersonSubmit',
      'linkPersonBtn',
      'peopleInMediaHeading',
      'taggedGroupHeading',
      'associatedGroupHeading',
      'profilePhotoBadge',
      'associatedBadge',
      'tagFaceQuickBtn',
      'unlinkPersonAria',
      'unlinkPersonBtn',
      'targetPersonBanner',
      'cancelTargetPerson',
      'autoDetectFacesBtn',
      'detectingFacesBtn',
      'suggestedFaceBadge',
      'suggestedFaceTooltip',
      'autoDetectFoundCount',
      'autoDetectNoneFound',
      'personLinkedToast',
      'personUnlinkedToast',
    ];

    const leadingIconRegex = /^\s*[+＋←→↑↓↔✓✔✕✖✗✘]\s+/;
    const emojiRegex = /[\u{1F300}-\u{1FAFF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]/u;

    for (const rk of requiredKeys) {
      expect(enMedia, `en.json must define mediaArchive.${rk}`).toHaveProperty(rk);
      expect(ruMedia, `ru.json must define mediaArchive.${rk}`).toHaveProperty(rk);

      const enVal = String(enMedia[rk]);
      const ruVal = String(ruMedia[rk]);

      expect(enVal.trim().length, `en mediaArchive.${rk} must not be empty`).toBeGreaterThan(0);
      expect(ruVal.trim().length, `ru mediaArchive.${rk} must not be empty`).toBeGreaterThan(0);

      expect(enVal, `en mediaArchive.${rk} must not have leading icon`).not.toMatch(leadingIconRegex);
      expect(ruVal, `ru mediaArchive.${rk} must not have leading icon`).not.toMatch(leadingIconRegex);

      expect(enVal, `en mediaArchive.${rk} must not have emoji`).not.toMatch(emojiRegex);
      expect(ruVal, `ru mediaArchive.${rk} must not have emoji`).not.toMatch(emojiRegex);
    }
  });

  it('localizes marriage_license and other GEDCOM event types in both locales', () => {
    expect(translate('ru', 'domain.events.marriage_license')).toBe('Регистрация брака / ЗАГС');
    expect(translate('en', 'domain.events.marriage_license')).toBe('Marriage License / Civil Registration');
    expect(formatVitalEventLabel('marriage_license', 'M', { locale: 'ru' })).toBe('регистрация брака');
    expect(formatVitalEventLabel('marriage_license', 'M', { locale: 'en' })).toBe('marriage license');
  });
});



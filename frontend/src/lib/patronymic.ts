/**
 * Deterministic Slavic patronymic derivation from a father's given name and child sex.
 * Supports Russian Cyrillic (including irregular rules: Илья -> Ильич/Ильинична,
 * Никита -> Никитич/Никитична, Павел -> Павлович/Павловна, Лев -> Львович/Львовна,
 * Михаил -> Михайлович/Михайловна) and Latin transliterated names.
 */

const CYRILLIC_IRREGULAR: Record<string, { m: string; f: string }> = {
  илья: { m: 'Ильич', f: 'Ильинична' },
  никита: { m: 'Никитич', f: 'Никитична' },
  кузьма: { m: 'Кузьмич', f: 'Кузьминична' },
  лука: { m: 'Лукич', f: 'Лукинична' },
  фома: { m: 'Фомич', f: 'Фоминична' },
  савва: { m: 'Саввич', f: 'Саввична' },
  яков: { m: 'Яковлевич', f: 'Яковлевна' },
  павел: { m: 'Павлович', f: 'Павловна' },
  лев: { m: 'Львович', f: 'Львовна' },
  михаил: { m: 'Михайлович', f: 'Михайловна' },
  гавриил: { m: 'Гаврилович', f: 'Гавриловна' },
  даниил: { m: 'Данилович', f: 'Даниловна' },
  василий: { m: 'Васильевич', f: 'Васильевна' },
  георгий: { m: 'Георгиевич', f: 'Георгиевна' },
  дмитрий: { m: 'Дмитриевич', f: 'Дмитриевна' },
  анатолий: { m: 'Анатольевич', f: 'Анатольевна' },
  николай: { m: 'Николаевич', f: 'Николаевна' },
  сергей: { m: 'Сергеевич', f: 'Сергеевна' },
  андрей: { m: 'Андреевич', f: 'Андреевна' },
  алексей: { m: 'Алексеевич', f: 'Алексеевна' },
  евгений: { m: 'Евгеньевич', f: 'Евгеньевна' },
  григорий: { m: 'Григорьевич', f: 'Григорьевна' },
  юрий: { m: 'Юрьевич', f: 'Юрьевна' },
  игорь: { m: 'Игоревич', f: 'Игоревна' },
};

const LATIN_IRREGULAR: Record<string, { m: string; f: string }> = {
  ilya: { m: 'Ilyich', f: 'Ilyinichna' },
  nikita: { m: 'Nikitich', f: 'Nikitichna' },
  pavel: { m: 'Pavlovich', f: 'Pavlovna' },
  lev: { m: 'Lvovich', f: 'Lvovna' },
  mikhail: { m: 'Mikhailovich', f: 'Mikhailovna' },
  yakov: { m: 'Yakovlevich', f: 'Yakovlevna' },
};

function capitalizeWord(str: string): string {
  if (!str) return '';
  return str.charAt(0).toUpperCase() + str.slice(1);
}

export function computeSlavicPatronymic(
  fatherGivenName: string,
  childSex: 'M' | 'F' | 'U'
): string {
  const cleaned = fatherGivenName.trim();
  if (!cleaned || childSex === 'U') {
    return '';
  }

  const lower = cleaned.toLowerCase();
  const isCyrillic = /[а-яё]/i.test(cleaned);

  if (isCyrillic) {
    const irregular = CYRILLIC_IRREGULAR[lower];
    if (irregular) {
      return childSex === 'M' ? irregular.m : irregular.f;
    }

    // Standard Russian morphology rules
    if (lower.endsWith('ий')) {
      const stem = capitalizeWord(lower.slice(0, -2));
      return childSex === 'M' ? `${stem}ьевич` : `${stem}ьевна`;
    }
    if (lower.endsWith('ей') || lower.endsWith('ай')) {
      const stem = capitalizeWord(lower.slice(0, -1));
      return childSex === 'M' ? `${stem}евич` : `${stem}евна`;
    }
    if (lower.endsWith('ь')) {
      const stem = capitalizeWord(lower.slice(0, -1));
      return childSex === 'M' ? `${stem}евич` : `${stem}евна`;
    }
    if (lower.endsWith('а') || lower.endsWith('я')) {
      const stem = capitalizeWord(lower.slice(0, -1));
      return childSex === 'M' ? `${stem}ич` : `${stem}ична`;
    }
    const stem = capitalizeWord(lower);
    return childSex === 'M' ? `${stem}ович` : `${stem}овна`;
  }

  // Latin transliterated names
  const latinIrregular = LATIN_IRREGULAR[lower];
  if (latinIrregular) {
    return childSex === 'M' ? latinIrregular.m : latinIrregular.f;
  }

  if (lower.endsWith('iy') || lower.endsWith('ii')) {
    const stem = capitalizeWord(lower.slice(0, -2));
    return childSex === 'M' ? `${stem}yevich` : `${stem}yevna`;
  }
  if (lower.endsWith('ey') || lower.endsWith('ay')) {
    const stem = capitalizeWord(lower.slice(0, -1));
    return childSex === 'M' ? `${stem}evich` : `${stem}evna`;
  }

  const stem = capitalizeWord(lower);
  return childSex === 'M' ? `${stem}ovich` : `${stem}ovna`;
}

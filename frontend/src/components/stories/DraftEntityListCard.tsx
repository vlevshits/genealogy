import React, { useEffect, useState } from 'react';
import { Edit3, Quote, Trash2, Users } from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  Dialog,
  IconButton,
  Select,
  TextInput,
} from '../ui';
import { FormField } from '../form';
import { DraftPersonNodeDto, DraftUnionDto } from '../../api/client';
import { useI18n } from '../../i18n';

export interface DraftEntityListCardProps {
  people: DraftPersonNodeDto[];
  unions: DraftUnionDto[];
  editingPerson: DraftPersonNodeDto | null;
  onEditPerson: (person: DraftPersonNodeDto | null) => void;
  onSavePerson: (updated: DraftPersonNodeDto) => void;
  onRemovePerson: (tempId: string) => void;
}

export const DraftEntityListCard: React.FC<DraftEntityListCardProps> = ({
  people,
  unions,
  editingPerson,
  onEditPerson,
  onSavePerson,
  onRemovePerson,
}) => {
  const { t } = useI18n();

  const [givenName, setGivenName] = useState('');
  const [surname, setSurname] = useState('');
  const [patronymic, setPatronymic] = useState('');
  const [maidenName, setMaidenName] = useState('');
  const [sex, setSex] = useState<'M' | 'F' | 'U'>('U');
  const [birthDate, setBirthDate] = useState('');
  const [birthPlace, setBirthPlace] = useState('');

  useEffect(() => {
    if (editingPerson) {
      setGivenName(editingPerson.given_name || '');
      setSurname(editingPerson.surname || '');
      setPatronymic(editingPerson.patronymic || '');
      setMaidenName(editingPerson.maiden_name || '');
      setSex(editingPerson.sex || 'U');
      setBirthDate(editingPerson.birth_date || '');
      setBirthPlace(editingPerson.birth_place || '');
    }
  }, [editingPerson]);

  const handleSaveModal = (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingPerson || !givenName.trim()) return;
    onSavePerson({
      ...editingPerson,
      given_name: givenName.trim(),
      surname: surname.trim(),
      patronymic: patronymic.trim() || null,
      maiden_name: maidenName.trim() || null,
      sex,
      birth_date: birthDate.trim() || null,
      birth_place: birthPlace.trim() || null,
    });
    onEditPerson(null);
  };

  return (
    <>
      <Card
        data-testid="draft-entity-list-card"
        className="p-4 sm:p-5 space-y-4"
      >
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Users className="h-4 w-4 text-brand" />
            <h3 className="text-sm font-semibold text-primary">
              {t('stories.entities.title', {
                peopleCount: people.length,
                unionsCount: unions.length,
              })}
            </h3>
          </div>
        </div>

        <div className="space-y-3 max-h-[420px] overflow-y-auto pr-1">
          {people.map((person) => {
            const score =
              person.confidence_score <= 1
                ? Math.round(person.confidence_score * 100)
                : Math.round(person.confidence_score);
            const isHigh = score >= 85;
            const isMedium = score >= 60 && score < 85;

            return (
              <div
                key={person.temp_id}
                data-testid={`draft-entity-item-${person.temp_id}`}
                className="rounded-xl border border-subtle bg-surface-subtle p-3.5 space-y-2"
              >
                <div className="flex items-start justify-between gap-2">
                  <div className="space-y-0.5 min-w-0">
                    <div className="flex flex-wrap items-center gap-1.5">
                      <span className="text-sm font-semibold text-primary">
                        {person.given_name}{' '}
                        {person.patronymic ? `${person.patronymic} ` : ''}
                        {person.surname}
                      </span>
                      {person.maiden_name && (
                        <span className="text-xs text-secondary">
                          ({person.maiden_name})
                        </span>
                      )}
                      <Badge
                        variant={
                          isHigh ? 'success' : isMedium ? 'warning' : 'brand'
                        }
                        size="sm"
                      >
                        {score}%
                      </Badge>
                    </div>
                    <div className="text-xs text-secondary">
                      {person.birth_date || t('stories.canvas.unknownDate')}
                      {person.birth_place ? ` · ${person.birth_place}` : ''}
                    </div>
                  </div>

                  <div className="flex items-center gap-1 shrink-0">
                    <IconButton
                      size="sm"
                      variant="ghost"
                      label={t('common.edit')}
                      data-testid={`edit-draft-person-btn-${person.temp_id}`}
                      onClick={() => onEditPerson(person)}
                    >
                      <Edit3 className="h-3.5 w-3.5" />
                    </IconButton>
                    {people.length > 1 && (
                      <IconButton
                        size="sm"
                        variant="ghost"
                        label={t('common.delete')}
                        data-testid={`delete-draft-person-btn-${person.temp_id}`}
                        onClick={() => onRemovePerson(person.temp_id)}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </IconButton>
                    )}
                  </div>
                </div>

                {person.source_snippet && (
                  <div className="flex items-start gap-1.5 text-[11px] text-tertiary bg-surface rounded-lg p-2 border border-subtle">
                    <Quote className="h-3 w-3 shrink-0 mt-0.5 text-brand" />
                    <span className="italic">{person.source_snippet}</span>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </Card>

      {/* Inline Edit Person Dialog */}
      <Dialog
        open={Boolean(editingPerson)}
        onOpenChange={(open) => {
          if (!open) onEditPerson(null);
        }}
        title={t('stories.entities.editModalTitle')}
        description={t('stories.entities.editModalDesc')}
        size="md"
        footer={
          <div className="flex items-center justify-end gap-2">
            <Button
              size="sm"
              variant="ghost"
              onClick={() => onEditPerson(null)}
            >
              {t('common.cancel')}
            </Button>
            <Button
              size="sm"
              variant="primary"
              data-testid="save-draft-person-modal-btn"
              onClick={handleSaveModal}
            >
              {t('common.save')}
            </Button>
          </div>
        }
      >
        <form onSubmit={handleSaveModal} className="space-y-3">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <FormField
              name="given_name"
              label={t('stories.entities.givenNameLabel')}
              required
            >
              <TextInput
                value={givenName}
                onChange={(e) => setGivenName(e.target.value)}
                aria-label={t('stories.entities.givenNameLabel')}
              />
            </FormField>
            <FormField
              name="surname"
              label={t('stories.entities.surnameLabel')}
            >
              <TextInput
                value={surname}
                onChange={(e) => setSurname(e.target.value)}
                aria-label={t('stories.entities.surnameLabel')}
              />
            </FormField>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <FormField
              name="patronymic"
              label={t('stories.entities.patronymicLabel')}
            >
              <TextInput
                value={patronymic}
                onChange={(e) => setPatronymic(e.target.value)}
                aria-label={t('stories.entities.patronymicLabel')}
              />
            </FormField>
            <FormField
              name="sex"
              label={t('merge.fields.sex')}
            >
              <Select
                value={sex}
                onChange={(v) => setSex(v as 'M' | 'F' | 'U')}
                ariaLabel={t('merge.fields.sex')}
                options={[
                  { value: 'F', label: t('people.sexFemale') },
                  { value: 'M', label: t('people.sexMale') },
                  { value: 'U', label: t('people.sexUnknown') },
                ]}
              />
            </FormField>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <FormField
              name="birth_date"
              label={t('merge.fields.birthDate')}
            >
              <TextInput
                value={birthDate}
                onChange={(e) => setBirthDate(e.target.value)}
                aria-label={t('merge.fields.birthDate')}
              />
            </FormField>
            <FormField
              name="birth_place"
              label={t('merge.fields.birthPlace')}
            >
              <TextInput
                value={birthPlace}
                onChange={(e) => setBirthPlace(e.target.value)}
                aria-label={t('merge.fields.birthPlace')}
              />
            </FormField>
          </div>
        </form>
      </Dialog>
    </>
  );
};

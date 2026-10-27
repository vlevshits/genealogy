import React, { useCallback, useEffect, useState } from 'react';
import {
  Calendar,
  Camera,
  FileText,
  Users,
} from 'lucide-react';
import {
  Alert,
  Button,
  Spinner,
  Tabs,
  useToast,
} from '../components/ui';
import {
  DeleteImpactDialog,
  LifeEventDialog,
  PersonEditDialog,
  QuickAddRelativeDialog,
} from '../components/person';
import { PublicGuestBanner } from '../components/tree/PublicGuestBanner';
import { formatTreeCardName } from '../components/tree/PersonNodeCard';
import {
  NoteEditDialog,
  ProfileChronologyTab,
  ProfileFamilyTab,
  ProfileHeroHeader,
  ProfileMediaTab,
  ProfileNotesTab,
} from '../components/profile';
import {
  deleteLifeEvent,
  deleteNote,
  fetchPersonProfile,
  LifeEventDto,
  NoteDto,
  PersonProfileResponseDto,
  ProfileLifeEventDto,
  QuickAddRelation,
  setHomePerson,
  UpdatePersonInput,
  updatePerson,
} from '../api/client';
import { useAuth } from '../lib/auth-context';
import { useI18n } from '../i18n';

export type ProfileTabKey = 'chronology' | 'family' | 'media' | 'notes';

export interface PersonProfilePageProps {
  personId: string;
  initialProfile?: PersonProfileResponseDto | null;
  initialTab?: ProfileTabKey;
  accessTier?: 'public_anonymous' | 'viewer' | 'editor' | 'admin' | string;
  onNavigate?: (path: string) => void;
  onSignIn?: () => void;
  onPersonLoaded?: (name: string) => void;
}

export function buildDefaultPersonProfile(
  personId: string = '00000000-0000-0000-0000-000000000001',
  accessTier: string = 'admin'
): PersonProfileResponseDto {
  const isAnonymous = accessTier === 'public_anonymous';
  const isLivingSample = personId === 'I0042' || personId.includes('living');

  const canEdit = accessTier === 'editor' || accessTier === 'admin';
  const canMerge = accessTier === 'admin';

  if (isAnonymous && isLivingSample) {
    return {
      person: {
        id: personId,
        gid: 'I0042',
        sex: 'F',
        is_living: true,
        is_private: false,
        primary_name: {
          given_name: 'E.',
          patronymic: null,
          surname: '',
        },
        birth: null,
        death: null,
        avatar_url: null,
        notes: null,
        version: 1,
      },
      kinship: null,
      events: [],
      families: {
        parent_families: [],
        spouse_families: [],
      },
      media: [],
      notes: [],
      sources_summary: [],
      capabilities: {
        can_edit: false,
        can_delete: false,
        can_merge: false,
        is_home_person: false,
      },
    };
  }

  return {
    person: {
      id: personId,
      gid: 'I0001',
      sex: 'M',
      is_living: false,
      is_private: false,
      primary_name: {
        given_name: 'Alexander',
        patronymic: 'Mikhailovich',
        surname: 'Levshits',
      },
      birth_surname: null,
      biography:
        '## Early Life & Engineering Career\nBorn in Vitebsk in 1912. Graduated from the Leningrad Polytechnic Institute and worked as a bridge and railway engineer.\n\n- Preserved the pre-war family correspondence archive.\n- Compiled the first hand-drawn pedigree chart in 1974.',
      birth: { date: '1912-03-19', place: 'Vitebsk, Belarus' },
      death: { date: '1989-11-04', place: 'Leningrad, USSR' },
      birth_sort: 19120319,
      death_sort: 19891104,
      avatar_url: null,
      notes: null,
      alt_names: [
        {
          id: 'alt-1',
          name_type: 'transliteration',
          given_name: 'Александр',
          patronymic: 'Михайлович',
          surname: 'Левшиц',
          lang: 'ru',
        },
      ],
      version: 1,
    },
    kinship: isAnonymous
      ? null
      : {
          home_person_id: 'node-living-1',
          target_person_id: personId,
          relation_key: 'grandfather',
          code: 'grandfather',
          label_en: 'Grandfather',
          label_ru: 'Дедушка',
          steps: 2,
          degree: 2,
        },
    events: [
      {
        id: 'ev-birth-1',
        gid: 'E0001',
        individual_id: personId,
        event_type: 'birth',
        date: {
          mod: 'exact',
          start: { y: 1912, m: 3, d: 19 },
          end: null,
          cal: 'gregorian',
          text: '19 Mar 1912',
        },
        date_sort: 19120319,
        gedcom_date: '19 MAR 1912',
        place_id: 'pl-vitebsk',
        place_full_name: 'Vitebsk, Belarus',
        latitude: 55.1904,
        longitude: 30.2049,
        description: 'Born in the family home on Zamkovaya Street.',
        source_text: 'Vitebsk Synagogue Metrical Book, 1912, entry #48',
        media_ids: ['m-doc-1'],
        scans_count: 1,
        is_private: false,
        age_at_event: 0,
        version: 1,
      },
      {
        id: 'ev-mar-1',
        gid: 'E0002',
        family_id: 'fam-1',
        event_type: 'marriage',
        date: {
          mod: 'exact',
          start: { y: 1938, m: 6, d: 12 },
          end: null,
          cal: 'gregorian',
          text: '12 Jun 1938',
        },
        date_sort: 19380612,
        gedcom_date: '12 JUN 1938',
        place_id: 'pl-spb',
        place_full_name: 'Leningrad, USSR',
        latitude: 59.9343,
        longitude: 30.3351,
        description: 'Civil marriage registration in Vasileostrovsky District.',
        source_text: 'Leningrad ZAGS Civil Registry Archive, 1938, act #214',
        media_ids: [],
        scans_count: 0,
        spouse_id: 'node-deceased-2',
        spouse_name: 'Sofia Yakovlevna Levshits',
        is_private: false,
        age_at_event: 26,
        version: 1,
      },
      {
        id: 'ev-death-1',
        gid: 'E0003',
        individual_id: personId,
        event_type: 'death',
        date: {
          mod: 'exact',
          start: { y: 1989, m: 11, d: 4 },
          end: null,
          cal: 'gregorian',
          text: '4 Nov 1989',
        },
        date_sort: 19891104,
        gedcom_date: '4 NOV 1989',
        place_id: 'pl-spb',
        place_full_name: 'Leningrad, USSR',
        latitude: 59.9343,
        longitude: 30.3351,
        description: 'Buried at Preobrazhenskoye Cemetery.',
        source_text: 'Leningrad ZAGS Civil Registry Archive, 1938, act #214',
        media_ids: [],
        scans_count: 0,
        is_private: false,
        age_at_event: 77,
        version: 1,
      },
    ],
    families: {
      parent_families: [
        {
          family_id: 'fam-parents-1',
          gid: 'F0000',
          father: {
            id: 'node-father-0',
            gid: 'I0000',
            given_name: 'Mikhail',
            patronymic: 'Isaakovich',
            surname: 'Levshits',
            sex: 'M',
            is_living: false,
            birth_year: 1884,
            death_year: 1952,
            avatar_url: null,
          },
          mother: null,
          siblings: [],
          version: 1,
        },
      ],
      spouse_families: [
        {
          family_id: 'fam-1',
          gid: 'F0001',
          relationship_type: 'married',
          spouse: {
            id: 'node-deceased-2',
            gid: 'I0002',
            given_name: 'Sofia',
            patronymic: 'Yakovlevna',
            surname: 'Levshits',
            sex: 'F',
            is_living: false,
            birth_year: 1915,
            death_year: 1996,
            avatar_url: null,
          },
          marriage_date: '12 Jun 1938',
          marriage_place: 'Leningrad, USSR',
          children: [
            {
              person: {
                id: 'node-living-1',
                gid: 'I0042',
                given_name: isAnonymous ? 'E.' : 'Elena',
                patronymic: isAnonymous ? null : 'Alexandrovna',
                surname: isAnonymous ? '' : 'Levshits',
                sex: 'F',
                is_living: true,
                birth_year: isAnonymous ? null : 1994,
                death_year: null,
                avatar_url: null,
              },
              sort_order: 0,
              rel_to_partner1: 'birth',
              rel_to_partner2: 'birth',
            },
          ],
          version: 1,
        },
      ],
    },
    media: [
      {
        id: 'm-photo-1',
        gid: 'M0001',
        title: 'Alexander Levshits Portrait (1955)',
        mime_type: 'image/webp',
        file_url: '/api/v1/media/00000000-0000-0000-0000-000000000101/file',
        is_private: false,
        only_living_persons: false,
        linked_person_ids: [personId],
        face_tags: [
          {
            id: 'ft-1',
            person_id: personId,
            person_name: 'Alexander Levshits',
            is_living: false,
            x: 0.25,
            y: 0.25,
            width: 0.2,
            height: 0.2,
          },
        ],
      },
      {
        id: 'm-doc-1',
        gid: 'M0004',
        title: 'Vitebsk Birth Certificate Scan (1912)',
        mime_type: 'application/pdf',
        file_url: '/api/v1/media/00000000-0000-0000-0000-000000000104/file',
        is_private: false,
        only_living_persons: false,
        linked_person_ids: [personId],
        face_tags: [],
      },
    ],
    notes: [
      {
        id: 'note-1',
        entity_type: 'individual',
        entity_id: personId,
        body: '### Archival Verification\nVerified birth date against **Vitebsk Rabbinate Metrical Book** (Fond 2496, Inventory 1).',
        is_private: false,
        sort_order: 0,
        version: 1,
      },
    ],
    sources_summary: [
      {
        source_text: 'Vitebsk Synagogue Metrical Book, 1912, entry #48',
        event_ids: ['ev-birth-1'],
        events_count: 1,
      },
      {
        source_text: 'Leningrad ZAGS Civil Registry Archive, 1938, act #214',
        event_ids: ['ev-mar-1', 'ev-death-1'],
        events_count: 2,
      },
    ],
    capabilities: {
      can_edit: canEdit,
      can_delete: canEdit,
      can_merge: canMerge,
      is_home_person: false,
    },
  };
}

export const PersonProfilePage: React.FC<PersonProfilePageProps> = ({
  personId,
  initialProfile = null,
  initialTab = 'chronology',
  accessTier: propAccessTier,
  onNavigate,
  onSignIn,
  onPersonLoaded,
}) => {
  const { t } = useI18n();
  const { user, isAuthenticated, updateUser } = useAuth();
  const { addToast } = useToast();

  const resolvedAccessTier =
    propAccessTier ??
    (user?.role
      ? user.role
      : isAuthenticated
      ? 'viewer'
      : 'public_anonymous');

  const [profile, setProfile] = useState<PersonProfileResponseDto>(
    () =>
      initialProfile ?? buildDefaultPersonProfile(personId, resolvedAccessTier)
  );
  const [loading, setLoading] = useState<boolean>(false);
  const [notFoundError, setNotFoundError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<ProfileTabKey>(initialTab);
  const [highlightedEventId, setHighlightedEventId] = useState<string | null>(
    null
  );

  // Authoring dialog states
  const [editPersonOpen, setEditPersonOpen] = useState<boolean>(false);
  const [eventDialogState, setEventDialogState] = useState<{
    open: boolean;
    initialEvent: LifeEventDto | null;
  }>({ open: false, initialEvent: null });
  const [noteDialogState, setNoteDialogState] = useState<{
    open: boolean;
    initialNote: NoteDto | null;
  }>({ open: false, initialNote: null });
  const [quickAddState, setQuickAddState] = useState<{
    open: boolean;
    relation: QuickAddRelation;
  }>({ open: false, relation: 'child' });
  const [deleteDialogOpen, setDeleteDialogOpen] = useState<boolean>(false);

  const loadProfile = useCallback(() => {
    if (initialProfile) return;
    setLoading(true);
    setNotFoundError(null);
    fetchPersonProfile(
      personId,
      'kinship,events,families,media,notes,sources'
    )
      .then((res) => {
        setProfile(res);
      })
      .catch((err) => {
        if (err && typeof err === 'object' && 'status' in err && err.status === 404) {
          setNotFoundError(t('profile.notFoundDesc'));
        }
      })
      .finally(() => {
        setLoading(false);
      });
  }, [initialProfile, personId, t]);

  useEffect(() => {
    if (initialProfile) {
      setProfile(initialProfile);
      return;
    }
    loadProfile();
  }, [initialProfile, loadProfile]);

  useEffect(() => {
    if (typeof window === 'undefined') return;
    const handleAvatarUpdated = (event: Event) => {
      const detail = (
        event as CustomEvent<{ personId: string; avatarUrl: string }>
      ).detail;
      if (detail && detail.personId === profile.person.id) {
        setProfile((prev) => ({
          ...prev,
          person: {
            ...prev.person,
            avatar_url: detail.avatarUrl,
          },
        }));
      }
    };
    window.addEventListener('genealogy:avatar-updated', handleAvatarUpdated);
    return () => {
      window.removeEventListener(
        'genealogy:avatar-updated',
        handleAvatarUpdated
      );
    };
  }, [profile.person.id]);

  const navigateTo = (nextPath: string) => {
    if (onNavigate) {
      onNavigate(nextPath);
    } else if (typeof window !== 'undefined') {
      window.history.pushState({}, '', nextPath);
      window.dispatchEvent(new PopStateEvent('popstate'));
    }
  };

  const { displayName, isMaskedLiving } = formatTreeCardName(
    profile.person,
    resolvedAccessTier
  );

  useEffect(() => {
    if (loading) {
      onPersonLoaded?.('');
      return;
    }
    onPersonLoaded?.(displayName || '');
  }, [displayName, loading, onPersonLoaded]);

  const handleSaveInlineIdentity = async (input: UpdatePersonInput) => {
    const updated = await updatePerson(profile.person.id, input);
    setProfile((prev) => ({
      ...prev,
      person: {
        ...prev.person,
        ...updated,
      },
    }));
    addToast({
      type: 'success',
      title: t('person.edit.savedToast'),
    });
    loadProfile();
  };

  const handleSaveBiography = async (nextBio: string) => {
    const updated = await updatePerson(profile.person.id, {
      biography: nextBio,
      version: profile.person.version ?? 1,
    });
    setProfile((prev) => ({
      ...prev,
      person: {
        ...prev.person,
        ...updated,
        biography: nextBio,
      },
    }));
    addToast({
      type: 'success',
      title: t('profile.biography.savedToast'),
    });
    loadProfile();
  };

  const handleSetHomePerson = async () => {
    try {
      const updatedUser = await setHomePerson(profile.person.id);
      updateUser(updatedUser);
      setProfile((prev) => ({
        ...prev,
        capabilities: {
          ...prev.capabilities,
          is_home_person: true,
        },
      }));
      addToast({
        type: 'success',
        title: t('tree.homePersonUpdatedToast'),
      });
      loadProfile();
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    }
  };

  const handleTogglePrivate = async () => {
    try {
      const nextPrivate = !profile.person.is_private;
      const updated = await updatePerson(profile.person.id, {
        is_private: nextPrivate,
        version: profile.person.version ?? 1,
      });
      setProfile((prev) => ({
        ...prev,
        person: {
          ...prev.person,
          ...updated,
          is_private: nextPrivate,
        },
      }));
      addToast({
        type: 'success',
        title: t('person.edit.savedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    }
  };

  const handleDeleteEvent = async (ev: ProfileLifeEventDto) => {
    try {
      await deleteLifeEvent(ev.id);
      setProfile((prev) => ({
        ...prev,
        events: prev.events.filter((item) => item.id !== ev.id),
      }));
      addToast({
        type: 'success',
        title: t('profile.chronology.eventDeletedToast'),
      });
      loadProfile();
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    }
  };

  const handleDeleteNote = async (note: NoteDto) => {
    try {
      await deleteNote(note.id);
      setProfile((prev) => ({
        ...prev,
        notes: prev.notes.filter((n) => n.id !== note.id),
      }));
      addToast({
        type: 'success',
        title: t('profile.notes.deletedToast'),
      });
      loadProfile();
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    }
  };

  const handleCopyLink = () => {
    if (typeof window !== 'undefined' && navigator.clipboard) {
      void navigator.clipboard.writeText(window.location.href);
    }
    addToast({
      type: 'info',
      title: t('profile.actions.linkCopiedToast'),
    });
  };

  if (notFoundError) {
    return (
      <div className="mx-auto max-w-4xl px-4 sm:px-6 lg:px-8 py-12 space-y-4">
        <Alert variant="danger" title={t('profile.notFoundTitle')}>
          {notFoundError}
        </Alert>
        <Button variant="secondary" onClick={() => navigateTo('/tree')}>
          {t('profile.actions.viewInTree')}
        </Button>
      </div>
    );
  }

  const tabItems = [
    {
      value: 'chronology' as ProfileTabKey,
      label: t('profile.tabs.chronology'),
      icon: <Calendar className="h-4 w-4" />,
      count: isMaskedLiving ? undefined : profile.events.length,
      content: (
        <ProfileChronologyTab
          person={profile.person}
          events={profile.events}
          canEdit={profile.capabilities.can_edit}
          isMaskedLiving={isMaskedLiving}
          highlightedEventId={highlightedEventId}
          onSelectEvent={(id) => setHighlightedEventId(id)}
          onAddEvent={() =>
            setEventDialogState({ open: true, initialEvent: null })
          }
          onEditEvent={(ev) =>
            setEventDialogState({ open: true, initialEvent: ev })
          }
          onDeleteEvent={(ev) => void handleDeleteEvent(ev)}
          onSaveBiography={handleSaveBiography}
          onSelectRelativeId={(relId) => navigateTo(`/persons/${relId}`)}
        />
      ),
    },
    {
      value: 'family' as ProfileTabKey,
      label: t('profile.tabs.family'),
      icon: <Users className="h-4 w-4" />,
      content: (
        <ProfileFamilyTab
          families={profile.families}
          canEdit={profile.capabilities.can_edit}
          accessTier={resolvedAccessTier}
          onSelectRelative={(relId) => navigateTo(`/persons/${relId}`)}
          onQuickAdd={(relation) => setQuickAddState({ open: true, relation })}
        />
      ),
    },
    {
      value: 'media' as ProfileTabKey,
      label: t('profile.tabs.media'),
      icon: <Camera className="h-4 w-4" />,
      count: isMaskedLiving ? undefined : profile.media.length,
      content: (
        <ProfileMediaTab
          media={profile.media}
          canEdit={profile.capabilities.can_edit}
          isMaskedLiving={isMaskedLiving}
          personId={profile.person.id}
          onSelectPersonId={(pid) => navigateTo(`/persons/${pid}`)}
          onMediaUpdated={(updatedAsset) => {
            setProfile((prev) => ({
              ...prev,
              media: prev.media.map((m) =>
                m.id === updatedAsset.id ? updatedAsset : m
              ),
            }));
          }}
          onMediaUploaded={(createdAssets) => {
            setProfile((prev) => ({
              ...prev,
              media: [...createdAssets, ...prev.media],
            }));
          }}
          onAvatarSynchronized={(targetPersonId, nextAvatarUrl) => {
            if (targetPersonId === profile.person.id) {
              setProfile((prev) => ({
                ...prev,
                person: {
                  ...prev.person,
                  avatar_url: nextAvatarUrl,
                },
              }));
            }
          }}
        />
      ),
    },
    {
      value: 'notes' as ProfileTabKey,
      label: t('profile.tabs.notes'),
      icon: <FileText className="h-4 w-4" />,
      count: isMaskedLiving ? undefined : profile.notes.length,
      content: (
        <ProfileNotesTab
          notes={profile.notes}
          sourcesSummary={profile.sources_summary}
          canEdit={profile.capabilities.can_edit}
          isMaskedLiving={isMaskedLiving}
          onAddNote={() =>
            setNoteDialogState({ open: true, initialNote: null })
          }
          onEditNote={(note) =>
            setNoteDialogState({ open: true, initialNote: note })
          }
          onDeleteNote={(note) => void handleDeleteNote(note)}
          onSelectSourceEvent={(eventId) => {
            setHighlightedEventId(eventId);
            setActiveTab('chronology');
          }}
        />
      ),
    },
  ];

  return (
    <div className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-6 sm:py-8 space-y-6">
      {resolvedAccessTier === 'public_anonymous' && (
        <PublicGuestBanner onSignIn={onSignIn} />
      )}

      {loading && (
        <div className="flex items-center gap-2 text-xs text-secondary">
          <Spinner size="sm" />
          <span>{t('common.loading')}</span>
        </div>
      )}

      {/* Hero Identity Header */}
      <ProfileHeroHeader
        person={profile.person}
        kinship={profile.kinship}
        capabilities={profile.capabilities}
        accessTier={resolvedAccessTier}
        mediaCount={profile.media.length}
        onViewInTree={() =>
          navigateTo(`/tree?focus=${encodeURIComponent(profile.person.id)}`)
        }
        onOpenEditModal={() => setEditPersonOpen(true)}
        onSaveInlineIdentity={handleSaveInlineIdentity}
        onAddEvent={() =>
          setEventDialogState({ open: true, initialEvent: null })
        }
        onQuickAdd={(relation) => setQuickAddState({ open: true, relation })}
        onSetHomePerson={() => void handleSetHomePerson()}
        onTogglePrivate={() => void handleTogglePrivate()}
        onDeletePerson={() => setDeleteDialogOpen(true)}
        onOpenHistory={() => navigateTo('/history')}
        onMergePerson={() =>
          navigateTo(
            `/people/merge?primary=${encodeURIComponent(profile.person.id)}`
          )
        }
        onCopyLink={handleCopyLink}
      />

      {/* 4-Tab Profile Content Navigation */}
      <Tabs<ProfileTabKey>
        ariaLabel={t('profile.tabs.ariaLabel')}
        items={tabItems}
        value={activeTab}
        onChange={setActiveTab}
      />

      {/* Authoring Dialogs */}
      <PersonEditDialog
        open={editPersonOpen}
        onOpenChange={setEditPersonOpen}
        person={profile.person}
        onPersonUpdated={(updated) => {
          setProfile((prev) => ({
            ...prev,
            person: {
              ...prev.person,
              ...updated,
            },
          }));
          loadProfile();
        }}
      />

      <LifeEventDialog
        open={eventDialogState.open}
        onOpenChange={(open) =>
          setEventDialogState((prev) => ({ ...prev, open }))
        }
        individualId={profile.person.id}
        initialEvent={eventDialogState.initialEvent}
        onEventSaved={() => loadProfile()}
      />

      <NoteEditDialog
        open={noteDialogState.open}
        onOpenChange={(open) =>
          setNoteDialogState((prev) => ({ ...prev, open }))
        }
        entityType="individual"
        entityId={profile.person.id}
        initialNote={noteDialogState.initialNote}
        onNoteSaved={(saved) => {
          setProfile((prev) => {
            const exists = prev.notes.some((n) => n.id === saved.id);
            return {
              ...prev,
              notes: exists
                ? prev.notes.map((n) => (n.id === saved.id ? saved : n))
                : [...prev.notes, saved],
            };
          });
          loadProfile();
        }}
      />

      <QuickAddRelativeDialog
        open={quickAddState.open}
        onOpenChange={(open) =>
          setQuickAddState((prev) => ({ ...prev, open }))
        }
        anchorPerson={profile.person}
        relation={quickAddState.relation}
        onRelativeAdded={() => loadProfile()}
      />

      <DeleteImpactDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        person={profile.person}
        onPersonDeleted={() => navigateTo('/tree')}
      />
    </div>
  );
};

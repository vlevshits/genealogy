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
  Skeleton,
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
import { RecordHistoryDrawer } from '../components/history';
import { useUndoToast } from '../hooks/useUndoToast';
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
import { getLocalizedErrorMessage } from '../lib/error-formatter';
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

import { buildDefaultPersonProfile } from "./default-person-profile";
export { buildDefaultPersonProfile };

export const PersonProfilePage: React.FC<PersonProfilePageProps> = ({
  personId,
  initialProfile = null,
  initialTab = 'chronology',
  accessTier: propAccessTier,
  onNavigate,
  onSignIn,
  onPersonLoaded,
}) => {
  const { t, locale } = useI18n();
  const { user, isAuthenticated, updateUser } = useAuth();
  const { addToast } = useToast();
  const { showUndoToast } = useUndoToast();

  const resolvedAccessTier =
    propAccessTier ??
    (isAuthenticated && user?.role
      ? user.role
      : isAuthenticated
      ? 'viewer'
      : 'public_anonymous');

  const [profile, setProfile] = useState<PersonProfileResponseDto | null>(
    () => initialProfile ?? null
  );
  const [loading, setLoading] = useState<boolean>(() => !initialProfile);
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
  const [historyDrawerOpen, setHistoryDrawerOpen] = useState<boolean>(false);

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
      .catch(() => {
        setNotFoundError(t('profile.notFoundDesc'));
      })
      .finally(() => {
        setLoading(false);
      });
  }, [initialProfile, personId, t]);

  useEffect(() => {
    if (initialProfile) {
      setProfile(initialProfile);
      setLoading(false);
      return;
    }
    loadProfile();
  }, [initialProfile, loadProfile]);

  useEffect(() => {
    if (typeof window === 'undefined' || !profile) return;
    const handleAvatarUpdated = (event: Event) => {
      const detail = (
        event as CustomEvent<{ personId: string; avatarUrl: string }>
      ).detail;
      if (detail && detail.personId === profile.person.id) {
        setProfile((prev) =>
          prev
            ? {
                ...prev,
                person: {
                  ...prev.person,
                  avatar_url: detail.avatarUrl,
                },
              }
            : prev
        );
      }
    };
    window.addEventListener('genealogy:avatar-updated', handleAvatarUpdated);
    return () => {
      window.removeEventListener(
        'genealogy:avatar-updated',
        handleAvatarUpdated
      );
    };
  }, [profile]);

  const navigateTo = (nextPath: string) => {
    if (onNavigate) {
      onNavigate(nextPath);
    } else if (typeof window !== 'undefined') {
      window.history.pushState({}, '', nextPath);
      window.dispatchEvent(new PopStateEvent('popstate'));
    }
  };

  const formattedIdentity = profile
    ? formatTreeCardName(profile.person, resolvedAccessTier, locale, 'card')
    : { displayName: '', isMaskedLiving: false };
  const { displayName, isMaskedLiving } = formattedIdentity;

  useEffect(() => {
    if (loading || !profile) {
      onPersonLoaded?.('');
      return;
    }
    onPersonLoaded?.(displayName || '');
  }, [displayName, loading, profile, onPersonLoaded]);

  if (loading && !profile) {
    return (
      <div className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-6 sm:py-8 space-y-6">
        <div className="flex items-center gap-2 text-xs text-secondary">
          <Spinner size="sm" />
          <span>{t('common.loading')}</span>
        </div>
        <div className="rounded-3xl border border-border bg-surface p-6 sm:p-8 space-y-4">
          <div className="flex items-center gap-4">
            <Skeleton variant="circle" className="h-24 w-24 rounded-2xl" />
            <div className="flex-1 space-y-2.5">
              <Skeleton variant="text" className="w-1/3 h-6" />
              <Skeleton variant="text" className="w-1/2 h-4" />
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (notFoundError || !profile) {
    return (
      <div className="mx-auto max-w-4xl px-4 sm:px-6 lg:px-8 py-12 space-y-4">
        <Alert variant="danger" title={t('profile.notFoundTitle')}>
          {notFoundError || t('profile.notFoundDesc')}
        </Alert>
        <Button variant="secondary" onClick={() => navigateTo('/tree')}>
          {t('profile.actions.viewInTree')}
        </Button>
      </div>
    );
  }

  const effectiveCanEdit =
    (resolvedAccessTier === 'editor' || resolvedAccessTier === 'admin') &&
    Boolean(profile.capabilities?.can_edit);
  const effectiveCanDelete =
    (resolvedAccessTier === 'editor' || resolvedAccessTier === 'admin') &&
    Boolean(profile.capabilities?.can_delete);
  const effectiveCanMerge =
    resolvedAccessTier === 'admin' && Boolean(profile.capabilities?.can_merge);
  const effectiveCapabilities = {
    can_edit: effectiveCanEdit,
    can_delete: effectiveCanDelete,
    can_merge: effectiveCanMerge,
    is_home_person: Boolean(profile.capabilities?.is_home_person),
  };

  const handleSaveInlineIdentity = async (input: UpdatePersonInput) => {
    const updated = await updatePerson(profile.person.id, input);
    setProfile((prev) =>
      prev
        ? {
            ...prev,
            person: {
              ...prev.person,
              ...updated,
            },
          }
        : prev
    );
    const txId = (updated as unknown as { tx_id?: string }).tx_id;
    if (txId) {
      showUndoToast({
        txId,
        title: t('history.undo_toast_label'),
        description: t('person.edit.savedToast'),
        onUndone: () => loadProfile(),
      });
    } else {
      addToast({
        type: 'success',
        title: t('person.edit.savedToast'),
      });
    }
    loadProfile();
  };

  const handleSaveBiography = async (nextBio: string) => {
    const updated = await updatePerson(profile.person.id, {
      biography: nextBio,
      version: profile.person.version ?? 1,
    });
    setProfile((prev) =>
      prev
        ? {
            ...prev,
            person: {
              ...prev.person,
              ...updated,
              biography: nextBio,
            },
          }
        : prev
    );
    const txId = (updated as unknown as { tx_id?: string }).tx_id;
    if (txId) {
      showUndoToast({
        txId,
        title: t('history.undo_toast_label'),
        description: t('profile.biography.savedToast'),
        onUndone: () => loadProfile(),
      });
    } else {
      addToast({
        type: 'success',
        title: t('profile.biography.savedToast'),
      });
    }
    loadProfile();
  };

  const handleSetHomePerson = async () => {
    try {
      const updatedUser = await setHomePerson(profile.person.id);
      updateUser(updatedUser);
      setProfile((prev) =>
        prev
          ? {
              ...prev,
              capabilities: {
                ...prev.capabilities,
                is_home_person: true,
              },
            }
          : prev
      );
      addToast({
        type: 'success',
        title: t('tree.homePersonUpdatedToast'),
      });
      loadProfile();
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
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
      setProfile((prev) =>
        prev
          ? {
              ...prev,
              person: {
                ...prev.person,
                ...updated,
                is_private: nextPrivate,
              },
            }
          : prev
      );
      const txId = (updated as unknown as { tx_id?: string }).tx_id;
      if (txId) {
        showUndoToast({
          txId,
          title: t('history.undo_toast_label'),
          description: t('person.edit.savedToast'),
          onUndone: () => loadProfile(),
        });
      } else {
        addToast({
          type: 'success',
          title: t('person.edit.savedToast'),
        });
      }
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleDeleteEvent = async (ev: ProfileLifeEventDto) => {
    try {
      const res = await deleteLifeEvent(ev.id);
      setProfile((prev) =>
        prev
          ? {
              ...prev,
              events: prev.events.filter((item) => item.id !== ev.id),
            }
          : prev
      );
      if (res?.tx_id) {
        showUndoToast({
          txId: res.tx_id,
          title: t('history.undo_toast_label'),
          description: t('profile.chronology.eventDeletedToast'),
          onUndone: () => loadProfile(),
        });
      } else {
        addToast({
          type: 'success',
          title: t('profile.chronology.eventDeletedToast'),
        });
      }
      loadProfile();
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleDeleteNote = async (note: NoteDto) => {
    try {
      const res = await deleteNote(note.id);
      setProfile((prev) =>
        prev
          ? {
              ...prev,
              notes: prev.notes.filter((n) => n.id !== note.id),
            }
          : prev
      );
      const txId = (res as { tx_id?: string } | undefined)?.tx_id;
      if (txId) {
        showUndoToast({
          txId,
          title: t('history.undo_toast_label'),
          description: t('profile.notes.deletedToast'),
          onUndone: () => loadProfile(),
        });
      } else {
        addToast({
          type: 'success',
          title: t('profile.notes.deletedToast'),
        });
      }
      loadProfile();
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
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
          canEdit={effectiveCanEdit}
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
          canEdit={effectiveCanEdit}
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
          canEdit={effectiveCanEdit}
          isMaskedLiving={isMaskedLiving}
          personId={profile.person.id}
          onSelectPersonId={(pid) => navigateTo(`/persons/${pid}`)}
          onMediaUpdated={(updatedAsset) => {
            setProfile((prev) =>
              prev
                ? {
                    ...prev,
                    media: prev.media.map((m) =>
                      m.id === updatedAsset.id ? updatedAsset : m
                    ),
                  }
                : prev
            );
          }}
          onMediaUploaded={(createdAssets) => {
            setProfile((prev) =>
              prev
                ? {
                    ...prev,
                    media: [...createdAssets, ...prev.media],
                  }
                : prev
            );
          }}
          onAvatarSynchronized={(targetPersonId, nextAvatarUrl) => {
            if (targetPersonId === profile.person.id) {
              setProfile((prev) =>
                prev
                  ? {
                      ...prev,
                      person: {
                        ...prev.person,
                        avatar_url: nextAvatarUrl,
                      },
                    }
                  : prev
              );
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
          canEdit={effectiveCanEdit}
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
        capabilities={effectiveCapabilities}
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
        onOpenHistory={() => setHistoryDrawerOpen(true)}
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
          setProfile((prev) =>
            prev
              ? {
                  ...prev,
                  person: {
                    ...prev.person,
                    ...updated,
                  },
                }
              : prev
          );
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
            if (!prev) return prev;
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

      <RecordHistoryDrawer
        open={historyDrawerOpen}
        onOpenChange={setHistoryDrawerOpen}
        entityType="person"
        entityId={profile.person.id}
        entityLabel={displayName}
        onRestored={() => loadProfile()}
      />
    </div>
  );
};


import React from "react";
import {
  DeleteImpactDialog,
  LifeEventDialog,
  PersonEditDialog,
  QuickAddRelativeDialog,
} from "../person";
import {
  QuickAddRelation,
  TreePersonNodeDto,
  TreeViewResponse,
} from "../../api/client";
import { PersonDrawer, PersonImmediateFamily } from "./PersonDrawer";
import { PersonBottomSheet } from "./PersonBottomSheet";
import { KinshipFinderDialog, HighlightedTrace } from "./KinshipFinderDialog";

export interface TreeCanvasOverlaysAndDialogsProps {
  isMobile: boolean;
  treeView: TreeViewResponse;
  canEdit: boolean;
  resolvedHomePersonId: string | null;
  focalId: string | null;
  selectedPersonId: string | null;
  selectedPerson: TreePersonNodeDto | null;
  selectedImmediateFamily?: PersonImmediateFamily;
  quickAddState: { anchor: TreePersonNodeDto; relation: QuickAddRelation } | null;
  editTarget: TreePersonNodeDto | null;
  eventTarget: TreePersonNodeDto | null;
  deleteTarget: TreePersonNodeDto | null;
  kinshipDialogOpen: boolean;
  kinshipInitialA: string | null;
  kinshipInitialB: string | null;
  onSelectPersonId: (id: string | null) => void;
  onMakeFocal: (person: TreePersonNodeDto) => void;
  onSetHomePerson: (person: TreePersonNodeDto) => void;
  onSelectRelative: (rel: TreePersonNodeDto) => void;
  onQuickAdd: (anchor: TreePersonNodeDto, relation: QuickAddRelation) => void;
  onCloseQuickAdd: () => void;
  onEditTargetChange: (person: TreePersonNodeDto | null) => void;
  onEventTargetChange: (person: TreePersonNodeDto | null) => void;
  onDeleteTargetChange: (person: TreePersonNodeDto | null) => void;
  onPersonDeleted: (deletedId: string) => void;
  onReloadTree: () => void;
  onOpenProfile: (person: TreePersonNodeDto) => void;
  onKinshipOpenChange: (open: boolean) => void;
  onHighlightPath: (ids: string[], traces?: HighlightedTrace[]) => void;
}

export const TreeCanvasOverlaysAndDialogs: React.FC<TreeCanvasOverlaysAndDialogsProps> = ({
  isMobile,
  treeView,
  canEdit,
  resolvedHomePersonId,
  focalId,
  selectedPersonId,
  selectedPerson,
  selectedImmediateFamily,
  quickAddState,
  editTarget,
  eventTarget,
  deleteTarget,
  kinshipDialogOpen,
  kinshipInitialA,
  kinshipInitialB,
  onSelectPersonId,
  onMakeFocal,
  onSetHomePerson,
  onSelectRelative,
  onQuickAdd,
  onCloseQuickAdd,
  onEditTargetChange,
  onEventTargetChange,
  onDeleteTargetChange,
  onPersonDeleted,
  onReloadTree,
  onOpenProfile,
  onKinshipOpenChange,
  onHighlightPath,
}) => {
  const isHomePerson = Boolean(
    selectedPerson &&
      resolvedHomePersonId &&
      (selectedPerson.id === resolvedHomePersonId ||
        selectedPerson.gid === resolvedHomePersonId)
  );

  return (
    <>
      {!isMobile ? (
        <PersonDrawer
          open={Boolean(selectedPerson)}
          onOpenChange={(open) => {
            if (!open) onSelectPersonId(null);
          }}
          person={selectedPerson}
          family={selectedImmediateFamily}
          accessTier={treeView.access_tier}
          canEdit={canEdit}
          isHomePerson={isHomePerson}
          onMakeFocal={onMakeFocal}
          onSetHomePerson={onSetHomePerson}
          onSelectRelative={onSelectRelative}
          onQuickAdd={onQuickAdd}
          onEditPerson={(p) => onEditTargetChange(p)}
          onAddEvent={(p) => onEventTargetChange(p)}
          onDeletePerson={(p) => onDeleteTargetChange(p)}
          onOpenProfile={onOpenProfile}
        />
      ) : (
        <PersonBottomSheet
          open={Boolean(selectedPerson)}
          onOpenChange={(open) => {
            if (!open) onSelectPersonId(null);
          }}
          person={selectedPerson}
          family={selectedImmediateFamily}
          accessTier={treeView.access_tier}
          canEdit={canEdit}
          isHomePerson={isHomePerson}
          onMakeFocal={onMakeFocal}
          onSetHomePerson={onSetHomePerson}
          onSelectRelative={onSelectRelative}
          onQuickAdd={onQuickAdd}
          onEditPerson={(p) => onEditTargetChange(p)}
          onOpenProfile={onOpenProfile}
        />
      )}

      <QuickAddRelativeDialog
        open={Boolean(quickAddState)}
        onOpenChange={(open) => {
          if (!open) onCloseQuickAdd();
        }}
        anchorPerson={quickAddState?.anchor ?? null}
        relation={quickAddState?.relation ?? "child"}
        existingPeople={treeView.nodes}
        onRelativeAdded={onReloadTree}
      />

      <PersonEditDialog
        open={Boolean(editTarget)}
        onOpenChange={(open) => {
          if (!open) onEditTargetChange(null);
        }}
        person={editTarget}
        onPersonUpdated={onReloadTree}
      />

      <LifeEventDialog
        open={Boolean(eventTarget)}
        onOpenChange={(open) => {
          if (!open) onEventTargetChange(null);
        }}
        individualId={eventTarget?.id ?? null}
        onEventSaved={onReloadTree}
      />

      <DeleteImpactDialog
        open={Boolean(deleteTarget)}
        onOpenChange={(open) => {
          if (!open) onDeleteTargetChange(null);
        }}
        person={deleteTarget}
        onPersonDeleted={onPersonDeleted}
      />

      <KinshipFinderDialog
        open={kinshipDialogOpen}
        onOpenChange={onKinshipOpenChange}
        people={treeView.nodes}
        edges={treeView.edges}
        initialPersonAId={
          kinshipInitialA ??
          resolvedHomePersonId ??
          focalId ??
          treeView.nodes[0]?.id
        }
        initialPersonBId={
          kinshipInitialB ??
          selectedPersonId ??
          treeView.nodes[1]?.id ??
          treeView.nodes[0]?.id
        }
        accessTier={
          treeView.access_tier as
            | "public_anonymous"
            | "viewer"
            | "editor"
            | "admin"
        }
        onHighlightPath={onHighlightPath}
      />
    </>
  );
};

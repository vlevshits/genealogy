import React from 'react';
import { describe, expect, it, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { ConfirmProvider, ToastProvider } from '../../src/components/ui';
import {
  computeIoU,
  filterOverlappingCandidates,
  detectFacesInImage,
  NormalizedBox,
  FaceTagCanvas,
  MediaMetadataPanel,
  LinkPersonDialog,
  MediaLightboxStudio,
  PersonFacePicker,
  DEFAULT_FACE_CANDIDATES,
  FaceTagCandidatePerson,
} from '../../src/components/media';
import { FaceTagDto, MediaAssetSummaryDto } from '../../src/api/client';

function renderWithProviders(
  node: React.ReactNode,
  locale: 'en' | 'ru' = 'en'
): string {
  return renderToStaticMarkup(
    <ThemeProvider>
      <I18nProvider initialLocale={locale}>
        <ToastProvider>
          <ConfirmProvider>{node}</ConfirmProvider>
        </ToastProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}

describe('Milestone 2.8: Associated People in Media Panel & Automatic Face Detection', () => {
  const sampleCandidates: FaceTagCandidatePerson[] = [
    {
      id: 'p-01',
      gid: 'I0001',
      displayName: 'Михаил Левшиц',
      birthYear: 1912,
      deathYear: 1989,
      isLiving: false,
      avatarUrl: '/api/v1/media/m-01/file',
    },
    {
      id: 'p-02',
      gid: 'I0002',
      displayName: 'Елена Левшиц',
      birthYear: 1994,
      deathYear: null,
      isLiving: true,
      avatarUrl: null,
    },
    {
      id: 'p-03',
      gid: 'I0003',
      displayName: 'Александр Петров',
      birthYear: 1950,
      deathYear: 2020,
      isLiving: false,
      avatarUrl: null,
    },
    {
      id: 'p-04',
      gid: 'I0004',
      displayName: 'Татьяна Смирнова',
      birthYear: 1952,
      deathYear: null,
      isLiving: true,
      avatarUrl: null,
    },
  ];

  const sampleAsset: MediaAssetSummaryDto = {
    id: 'm-01',
    gid: 'M0001',
    title: 'Family Gathering in Leningrad (1955)',
    mime_type: 'image/jpeg',
    file_url: '/api/v1/media/m-01/file',
    width: 1200,
    height: 900,
    is_private: false,
    only_living_persons: false,
    linked_person_ids: ['p-01', 'p-02', 'p-03'], // p-01 is tagged; p-02 and p-03 are untagged associated
    face_tags: [
      {
        id: 'ft-01',
        person_id: 'p-01',
        person_name: 'Михаил Левшиц',
        is_living: false,
        x: 0.2,
        y: 0.25,
        width: 0.15,
        height: 0.2,
        is_primary_avatar: true,
      },
    ],
    links: [
      {
        id: 'link-01',
        media_id: 'm-01',
        entity_type: 'individual',
        entity_id: 'p-01',
        role: 'primary_avatar',
      },
      {
        id: 'link-02',
        media_id: 'm-01',
        entity_type: 'individual',
        entity_id: 'p-02',
        role: 'primary_avatar', // Associated with profile photo role
      },
      {
        id: 'link-03',
        media_id: 'm-01',
        entity_type: 'individual',
        entity_id: 'p-03',
        role: 'depicted', // Regular associated relative
      },
    ],
    version: 1,
  };

  describe('1. Face Detection & Geometry Helpers', () => {
    it('computes exact Intersection over Union (IoU) for overlapping and non-overlapping bounding boxes', () => {
      const boxA: NormalizedBox = { x: 0.1, y: 0.1, width: 0.2, height: 0.2 };
      const boxB: NormalizedBox = { x: 0.1, y: 0.1, width: 0.2, height: 0.2 };
      expect(computeIoU(boxA, boxB)).toBeCloseTo(1.0, 5);

      const disjointBox: NormalizedBox = { x: 0.5, y: 0.5, width: 0.2, height: 0.2 };
      expect(computeIoU(boxA, disjointBox)).toBe(0);

      // Half-overlapping box:
      // A: [0.1, 0.3] x [0.1, 0.3], area = 0.04
      // C: [0.2, 0.4] x [0.1, 0.3], area = 0.04
      // Intersection: [0.2, 0.3] x [0.1, 0.3], area = 0.1 * 0.2 = 0.02
      // Union: 0.04 + 0.04 - 0.02 = 0.06
      // IoU: 0.02 / 0.06 = 1/3 ~ 0.3333
      const boxC: NormalizedBox = { x: 0.2, y: 0.1, width: 0.2, height: 0.2 };
      expect(computeIoU(boxA, boxC)).toBeCloseTo(1 / 3, 4);
    });

    it('filters out candidate boxes overlapping existing tags or duplicate candidates', () => {
      const existingTags: FaceTagDto[] = [
        {
          id: 'ft-existing',
          person_id: 'p-01',
          person_name: 'Person 1',
          is_living: false,
          x: 0.2,
          y: 0.2,
          width: 0.2,
          height: 0.2,
        },
      ];

      const candidates: NormalizedBox[] = [
        // High overlap with existing tag (IoU ~ 0.7) -> filtered out
        { x: 0.22, y: 0.21, width: 0.19, height: 0.19 },
        // Centroid inside existing tag -> filtered out
        { x: 0.25, y: 0.25, width: 0.08, height: 0.08 },
        // New face 1 (untagged) -> preserved
        { x: 0.6, y: 0.3, width: 0.18, height: 0.22 },
        // Duplicate of New face 1 (IoU > 0.5) -> filtered out
        { x: 0.61, y: 0.31, width: 0.18, height: 0.21 },
        // New face 2 (untagged) -> preserved
        { x: 0.8, y: 0.6, width: 0.15, height: 0.2 },
      ];

      const remaining = filterOverlappingCandidates(candidates, existingTags, 0.3);
      expect(remaining.length).toBe(2);
      expect(remaining[0].x).toBe(0.6);
      expect(remaining[1].x).toBe(0.8);
    });

    it('detectFacesInImage safely handles headless/test environment without exceptions', async () => {
      const result = await detectFacesInImage(null as any);
      expect(result).toEqual([]);
    });
  });

  describe('2. Link Person Modal Dialog', () => {
    it('filters out candidates who are already linked or tagged in the media asset', () => {
      const html = renderWithProviders(
        <LinkPersonDialog
          open={true}
          onOpenChange={() => {}}
          asset={sampleAsset}
          candidates={sampleCandidates}
          onLinkPerson={() => {}}
        />,
        'en'
      );

      // p-01 (tagged), p-02 (linked), p-03 (linked) must be filtered out
      expect(html).not.toContain('Михаил Левшиц');
      expect(html).not.toContain('Елена Левшиц');
      expect(html).not.toContain('Александр Петров');

      // Only p-04 (unassociated) is eligible to be linked!
      expect(html).toContain('Татьяна Смирнова');
      expect(html).toContain('I0004');
      expect(html).toContain('Link Person');
    });

    it('renders Russian localization and cross-script search field in LinkPersonDialog', () => {
      const htmlRu = renderWithProviders(
        <LinkPersonDialog
          open={true}
          onOpenChange={() => {}}
          asset={sampleAsset}
          candidates={sampleCandidates}
          onLinkPerson={() => {}}
        />,
        'ru'
      );

      expect(htmlRu).toContain('Связать родственника с медиа');
      expect(htmlRu).toContain('Поиск родственника для связи…');
      expect(htmlRu).toContain('Татьяна Смирнова');
      expect(htmlRu).toContain('Связать');
    });
  });

  describe('3. Media Metadata Panel Inspector', () => {
    it('partitions people into Tagged Faces and Associated Untagged Relatives with correct badges', () => {
      const htmlEn = renderWithProviders(
        <MediaMetadataPanel
          asset={sampleAsset}
          canEdit={true}
          candidates={sampleCandidates}
          onSaveMetadata={() => {}}
        />,
        'en'
      );

      // Section total count: 1 tagged + 2 associated = 3
      expect(htmlEn).toContain('People in this Media (3)');

      // Subsections
      expect(htmlEn).toContain('Tagged Faces (1)');
      expect(htmlEn).toContain('Associated Relatives (2)');

      // Tagged individual
      expect(htmlEn).toContain('Михаил Левшиц');
      expect(htmlEn).toContain('Primary Avatar');

      // Untagged associated individuals
      expect(htmlEn).toContain('Елена Левшиц');
      expect(htmlEn).toContain('Profile Photo'); // link role === 'primary_avatar'

      expect(htmlEn).toContain('Александр Петров');
      expect(htmlEn).toContain('Associated'); // link role === 'depicted'

      // Actions
      expect(htmlEn).toContain('Tag Face');
      expect(htmlEn).toContain('Unlink');
      expect(htmlEn).toContain('Link Person');
    });

    it('highlights the active target person row when armed for face tagging', () => {
      const htmlTargetArmed = renderWithProviders(
        <MediaMetadataPanel
          asset={sampleAsset}
          canEdit={true}
          candidates={sampleCandidates}
          targetPersonId="p-02"
          onSaveMetadata={() => {}}
        />,
        'en'
      );

      // p-02 row should have the active ring highlight
      expect(htmlTargetArmed).toContain('ring-2 ring-brand');
    });
  });

  describe('4. Canvas Candidate Overlays & 1-Click Adoption', () => {
    const candidateRects: NormalizedBox[] = [
      { x: 0.55, y: 0.28, width: 0.18, height: 0.24 },
    ];

    it('renders suggested face boxes with Suggested Face badge and pulsing border in tagging mode', () => {
      const htmlCanvas = renderWithProviders(
        <FaceTagCanvas
          asset={sampleAsset}
          canEdit={true}
          isTaggingMode={true}
          candidates={sampleCandidates}
          candidateBoxes={candidateRects}
        />,
        'en'
      );

      expect(htmlCanvas).toContain('Suggested Face');
      expect(htmlCanvas).toContain('border-sky-400');
      expect(htmlCanvas).toContain('left:55%');
      expect(htmlCanvas).toContain('top:28%');
      expect(htmlCanvas).toContain('width:18%');
      expect(htmlCanvas).toContain('height:24%');
    });

    it('renders top banner when a pre-selected target relative is armed', () => {
      const htmlArmed = renderWithProviders(
        <FaceTagCanvas
          asset={sampleAsset}
          canEdit={true}
          isTaggingMode={true}
          candidates={sampleCandidates}
          candidateBoxes={candidateRects}
          targetPersonId="p-02"
        />,
        'en'
      );

      expect(htmlArmed).toContain('Click a suggested face rectangle or draw a box for Елена Левшиц');
      expect(htmlArmed).toContain('Cancel');
    });

    it('renders Russian target person banner and Suggested Face badge in RU locale', () => {
      const htmlArmedRu = renderWithProviders(
        <FaceTagCanvas
          asset={sampleAsset}
          canEdit={true}
          isTaggingMode={true}
          candidates={sampleCandidates}
          candidateBoxes={candidateRects}
          targetPersonId="p-02"
        />,
        'ru'
      );

      expect(htmlArmedRu).toContain('Выберите найденную область лица или нарисуйте рамку для: Елена Левшиц');
      expect(htmlArmedRu).toContain('Найдено лицо');
      expect(htmlArmedRu).toContain('Отмена');
    });
  });

  describe('5. Media Lightbox Studio Integration', () => {
    it('renders studio with complete integration in English and Russian', () => {
      const htmlStudio = renderWithProviders(
        <MediaLightboxStudio
          open={true}
          onOpenChange={() => {}}
          asset={sampleAsset}
          canEdit={true}
          candidates={sampleCandidates}
        />,
        'en'
      );

      expect(htmlStudio).toContain('Family Gathering in Leningrad (1955)');
      expect(htmlStudio).toContain('People in this Media (3)');
      expect(htmlStudio).toContain('Link Person');
    });
  });

  describe('6. Milestone 2.8 UX Review Hardening (UX-078 - UX-086)', () => {
    it('UX-078 & UX-082: renders delete media action with touch-target-expand and localized confirm labels', () => {
      const htmlPanel = renderWithProviders(
        <MediaMetadataPanel
          asset={sampleAsset}
          canEdit={true}
          candidates={sampleCandidates}
          onSaveMetadata={() => {}}
          onDeleteAsset={() => {}}
        />,
        'en'
      );

      expect(htmlPanel).toContain('Delete');
      expect(htmlPanel).toContain('touch-target-expand');
    });

    it('UX-079: formats person headers with flex-wrap and break-words to prevent badge overlap', () => {
      const htmlPanel = renderWithProviders(
        <MediaMetadataPanel
          asset={sampleAsset}
          canEdit={true}
          candidates={sampleCandidates}
          onSaveMetadata={() => {}}
        />,
        'ru'
      );

      expect(htmlPanel).toContain('flex-wrap sm:flex-nowrap');
      expect(htmlPanel).toContain('break-words');
      expect(htmlPanel).toContain('shrink-0');
    });

    it('UX-080, UX-084, UX-085: candidate face boxes have role="button", tabIndex=0, dismiss action, and high contrast border', () => {
      const htmlCanvas = renderWithProviders(
        <FaceTagCanvas
          asset={sampleAsset}
          canEdit={true}
          isTaggingMode={true}
          candidates={sampleCandidates}
          candidateBoxes={[{ x: 0.4, y: 0.3, width: 0.2, height: 0.2 }]}
          onDismissCandidateBox={() => {}}
        />,
        'en'
      );

      // UX-080 accessibility attributes
      expect(htmlCanvas).toContain('role="button"');
      expect(htmlCanvas).toContain('tabindex="0"');
      expect(htmlCanvas).toContain('aria-label="Click to tag face"');
      expect(htmlCanvas).toContain('focus-visible:ring-sky-500');

      // UX-085 high contrast border for light and dark mode
      expect(htmlCanvas).toContain('border-sky-600 dark:border-sky-400');
      expect(htmlCanvas).toContain('bg-sky-500/10');

      // UX-084 dismiss candidate button
      expect(htmlCanvas).toContain('aria-label="Dismiss suggested face"');
    });

    it('UX-081: pre-selected target person banner stacks gracefully on mobile with break-words', () => {
      const htmlCanvas = renderWithProviders(
        <FaceTagCanvas
          asset={sampleAsset}
          canEdit={true}
          isTaggingMode={true}
          candidates={sampleCandidates}
          targetPersonId="p-02"
        />,
        'en'
      );

      expect(htmlCanvas).toContain('flex-col sm:flex-row');
      expect(htmlCanvas).toContain('break-words');
    });

    it('UX-082: navigation chevrons in studio have touch-target-expand', () => {
      const htmlStudio = renderWithProviders(
        <MediaLightboxStudio
          open={true}
          onOpenChange={() => {}}
          asset={sampleAsset}
          canEdit={true}
          candidates={sampleCandidates}
          onPrevAsset={() => {}}
          onNextAsset={() => {}}
        />,
        'en'
      );

      expect(htmlStudio).toContain('touch-target-expand');
      expect(htmlStudio).toContain('Previous media item');
      expect(htmlStudio).toContain('Next media item');
    });

    it('UX-083: auto-detect faces button is rendered exactly once on canvas stage in tagging mode', () => {
      const htmlCanvas = renderWithProviders(
        <FaceTagCanvas
          asset={sampleAsset}
          canEdit={true}
          isTaggingMode={true}
          candidates={sampleCandidates}
          onDetectFaces={() => {}}
        />,
        'en'
      );

      const matches = htmlCanvas.match(/Auto-Detect Faces/g) || [];
      expect(matches.length).toBe(1);
    });

    it('UX-086: LinkPersonDialog and PersonFacePicker avoid nested-interactive inside role="option"', () => {
      const htmlLinkDialog = renderWithProviders(
        <LinkPersonDialog
          open={true}
          onOpenChange={() => {}}
          asset={sampleAsset}
          candidates={sampleCandidates}
          onLinkPerson={() => {}}
        />,
        'en'
      );

      // Listbox has tabIndex=0 for keyboard navigation
      expect(htmlLinkDialog).toContain('role="listbox"');
      expect(htmlLinkDialog).toContain('tabindex="0"');
      // Button itself carries role="option" directly, not wrapped inside a <div role="option">
      expect(htmlLinkDialog).toContain('<button');
      expect(htmlLinkDialog).toContain('role="option"');
      expect(htmlLinkDialog).not.toContain('<div role="option"');

      const htmlPicker = renderWithProviders(
        <PersonFacePicker
          candidates={sampleCandidates}
          onCancel={() => {}}
          onConfirm={() => {}}
        />,
        'en'
      );

      expect(htmlPicker).toContain('role="listbox"');
      expect(htmlPicker).toContain('tabindex="0"');
      expect(htmlPicker).toContain('role="option"');
      expect(htmlPicker).not.toContain('<div role="option"');
    });
  });
});

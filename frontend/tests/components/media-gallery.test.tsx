import React from 'react';
import { describe, expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'node:fs';
import path from 'node:path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { ConfirmProvider, ToastProvider } from '../../src/components/ui';
import {
  DEFAULT_FACE_CANDIDATES,
  DEFAULT_MEDIA_ARCHIVE_ITEMS,
  MediaPage,
  serializeMediaUrlParams,
} from '../../src/pages/MediaPage';
import {
  FaceTagCanvas,
  MediaLightboxStudio,
  MediaMetadataPanel,
  MediaUploadDialog,
  PersonFacePicker,
} from '../../src/components/media';
import { getMediaCropUrl } from '../../src/api/client';

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

describe('Phase 1.9 — Media Archive, Gallery, Face Tagging & Avatar Synchronization', () => {
  it('renders MediaPage masonry gallery, decade timeline chips, asset type filter, and restores URL query parameters', () => {
    const htmlEn = renderWithProviders(
      <MediaPage userRole="editor" />,
      'en'
    );

    expect(htmlEn).toContain('Media Archive &amp; Face Tagging Studio');
    expect(htmlEn).toContain('Mikhail Levshits Archival Portrait (1955)');
    expect(htmlEn).toContain('Unidentified Family Gathering in Tver (1941)');
    expect(htmlEn).toContain('Upload Media');
    expect(htmlEn).toContain('1940s');
    expect(htmlEn).toContain('1960s');

    // URL query filtering: ?decade=1950s
    const html1950s = renderWithProviders(
      <MediaPage initialSearch="?decade=1950s" userRole="editor" />,
      'en'
    );
    expect(html1950s).toContain('Mikhail Levshits Archival Portrait (1955)');
    expect(html1950s).not.toContain(
      'Unidentified Family Gathering in Tver (1941)'
    );

    // URL query filtering: ?untagged=true
    const htmlUntagged = renderWithProviders(
      <MediaPage initialSearch="?untagged=true" userRole="editor" />,
      'ru'
    );
    expect(htmlUntagged).toContain(
      'Unidentified Family Gathering in Tver (1941)'
    );
    expect(htmlUntagged).not.toContain(
      'Mikhail Levshits Archival Portrait (1955)'
    );

    // URL parameter serializer
    const serialized = serializeMediaUrlParams({
      q: 'vitebsk',
      type: 'photos',
      personId: 'p-1',
      decade: '1950s',
      untagged: true,
    });
    expect(serialized).toContain('q=vitebsk');
    expect(serialized).toContain('type=photos');
    expect(serialized).toContain('person_id=p-1');
    expect(serialized).toContain('decade=1950s');
    expect(serialized).toContain('untagged=true');
  });

  it('hides authoring controls (Upload Media, Tag Faces, Save Details) for Viewer role', () => {
    const htmlViewer = renderWithProviders(
      <MediaPage userRole="viewer" />,
      'en'
    );
    expect(htmlViewer).not.toContain('Upload Media');

    const htmlMetadataViewer = renderWithProviders(
      <MediaMetadataPanel
        asset={DEFAULT_MEDIA_ARCHIVE_ITEMS[0]}
        canEdit={false}
        onSaveMetadata={() => {}}
      />,
      'en'
    );
    expect(htmlMetadataViewer).toContain(
      'Mikhail Levshits Archival Portrait (1955)'
    );
    expect(htmlMetadataViewer).not.toContain('Save Details');
    expect(htmlMetadataViewer).not.toContain('Use as Avatar');
  });

  it('renders FaceTagCanvas with normalized 0.0–1.0 percentage bounding boxes, deceased † vital dates, and zero Living pills', () => {
    const asset = DEFAULT_MEDIA_ARCHIVE_ITEMS[0];
    const htmlCanvas = renderWithProviders(
      <FaceTagCanvas
        asset={asset}
        canEdit={true}
        candidates={DEFAULT_FACE_CANDIDATES}
      />,
      'en'
    );

    // Normalized coordinates 0.25, 0.30, 0.20, 0.25 -> percentage CSS
    expect(htmlCanvas).toContain('left:25%');
    expect(htmlCanvas).toContain('top:30%');
    expect(htmlCanvas).toContain('width:20%');
    expect(htmlCanvas).toContain('height:25%');
    expect(htmlCanvas).toContain('Левшиц Михаил Александрович');
    expect(htmlCanvas).toContain('† 1989');
    expect(htmlCanvas).toContain('Tag Faces');
    expect(htmlCanvas).not.toContain('>Living<');
  });

  it('renders PersonFacePicker with deceased † marker, Set as primary profile avatar checkbox, and crop URL helper', () => {
    const htmlPicker = renderWithProviders(
      <PersonFacePicker
        candidates={DEFAULT_FACE_CANDIDATES}
        onConfirm={() => {}}
        onCancel={() => {}}
      />,
      'en'
    );

    expect(htmlPicker).toContain('Who is in this photo?');
    expect(htmlPicker).toContain('Левшиц Михаил Александрович');
    expect(htmlPicker).toContain('b. 1912 · † 1989');
    expect(htmlPicker).toContain('Левшиц Елена Александровна');
    expect(htmlPicker).toContain('b. 1994');
    expect(htmlPicker).not.toContain('>Living<');
    expect(htmlPicker).toContain('Set as primary profile avatar');

    const cropUrl = getMediaCropUrl('m-101', {
      x: 0.22,
      y: 0.18,
      width: 0.18,
      height: 0.22,
    });
    expect(cropUrl).toBe(
      '/api/v1/media/m-101/crop?x=0.2200&y=0.1800&w=0.1800&h=0.2200'
    );
  });

  it('renders MediaLightboxStudio and MediaUploadDialog in both English and Russian', () => {
    const htmlStudioRu = renderWithProviders(
      <MediaLightboxStudio
        open={true}
        onOpenChange={() => {}}
        asset={DEFAULT_MEDIA_ARCHIVE_ITEMS[0]}
        canEdit={true}
      />,
      'ru'
    );
    expect(htmlStudioRu).toContain('Отметить лица');
    expect(htmlStudioRu).toContain('Люди на снимке');
    expect(htmlStudioRu).toContain('Основной портрет');

    const htmlUploadEn = renderWithProviders(
      <MediaUploadDialog open={true} onOpenChange={() => {}} />,
      'en'
    );
    expect(htmlUploadEn).toContain('Upload Family Media &amp; Scans');
    expect(htmlUploadEn).toContain('Private family asset');
  });

  it('uses zero raw <button>, <input>, <select>, or <textarea> HTML elements in media components and page', () => {
    const filesToCheck = [
      path.resolve(__dirname, '../../src/pages/MediaPage.tsx'),
      path.resolve(__dirname, '../../src/components/media/MediaFilterBar.tsx'),
      path.resolve(__dirname, '../../src/components/media/MediaCard.tsx'),
      path.resolve(__dirname, '../../src/components/media/MediaWall.tsx'),
      path.resolve(
        __dirname,
        '../../src/components/media/MediaUploadDialog.tsx'
      ),
      path.resolve(
        __dirname,
        '../../src/components/media/PersonFacePicker.tsx'
      ),
      path.resolve(__dirname, '../../src/components/media/FaceTagCanvas.tsx'),
      path.resolve(
        __dirname,
        '../../src/components/media/MediaMetadataPanel.tsx'
      ),
      path.resolve(
        __dirname,
        '../../src/components/media/MediaLightboxStudio.tsx'
      ),
    ];

    const rawTagRegex = /<(button|input|select|textarea)\b/g;

    for (const filePath of filesToCheck) {
      const source = fs.readFileSync(filePath, 'utf-8');
      const matches = source.match(rawTagRegex);
      expect(
        matches,
        `Found raw HTML form/button element in ${path.basename(filePath)}`
      ).toBeNull();
    }
  });
});

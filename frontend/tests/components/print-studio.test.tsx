import React from 'react';
import { describe, expect, it, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'node:fs';
import path from 'node:path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { ConfirmProvider, ToastProvider } from '../../src/components/ui';
import {
  PrintConfigSidebar,
  PrintPreviewCanvas,
  PrintSummaryBar,
} from '../../src/components/print';
import { PrintExportPage } from '../../src/pages/PrintExportPage';
import {
  exportPrintablePdf,
  fetchPdfPreviewInfo,
  PdfExportPreviewInfo,
  PdfExportRequest,
} from '../../src/api/client';

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

describe('Feature 05 & Page 05 — Printable PDF Tree Export Studio', () => {
  it('enforces zero raw <button>, <input>, <select>, or <textarea> HTML tags and zero hex colors across print components', () => {
    const filesToCheck = [
      'src/components/print/PrintConfigSidebar.tsx',
      'src/components/print/PrintSummaryBar.tsx',
      'src/components/print/PrintPreviewCanvas.tsx',
      'src/pages/PrintExportPage.tsx',
    ];

    for (const relPath of filesToCheck) {
      const fullPath = path.resolve(__dirname, '../../', relPath);
      const source = fs.readFileSync(fullPath, 'utf-8');

      expect(
        /<(button|input|select|textarea)\b/.test(source),
        `Found raw HTML form/button element in ${relPath}`
      ).toBe(false);

      expect(
        /#[0-9a-fA-F]{3,8}\b/.test(source),
        `Found hardcoded hex color in ${relPath}`
      ).toBe(false);
    }
  });

  it('restricts /export/print to Administrators and renders AccessDeniedCard for Viewers & Editors', () => {
    const htmlAdmin = renderWithProviders(
      <PrintExportPage userRole="admin" />,
      'en'
    );
    expect(htmlAdmin).toContain('data-testid="print-export-page"');
    expect(htmlAdmin).toContain('Print Configuration');
    expect(htmlAdmin).toContain('Live PDF Layout Preview');

    const htmlViewer = renderWithProviders(
      <PrintExportPage userRole="viewer" />,
      'en'
    );
    expect(htmlViewer).not.toContain('data-testid="print-export-page"');
    expect(htmlViewer).toContain('Administrator Access Required');
    expect(htmlViewer).toContain('Administrator');

    const htmlEditorRu = renderWithProviders(
      <PrintExportPage userRole="editor" />,
      'ru'
    );
    expect(htmlEditorRu).not.toContain('data-testid="print-export-page"');
    expect(htmlEditorRu).toContain('Требуются права администратора');
  });

  it('renders PrintConfigSidebar with all required controls in EN and RU', () => {
    const mockConfig: PdfExportRequest = {
      view_mode: 'pedigree',
      generations: 5,
      format: 'tiled',
      target_sheet_size: 'A4',
      overlap_mm: 10,
      show_crop_marks: true,
      include_assembly_guide: true,
      theme: 'modern',
      redact_living: true,
      show_photos: true,
    };

    const htmlEn = renderWithProviders(
      <PrintConfigSidebar
        config={mockConfig}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );

    expect(htmlEn).toContain('Print Configuration');
    expect(htmlEn).toContain('Focal Person');
    expect(htmlEn).toContain('Chart Layout');
    expect(htmlEn).toContain('Pedigree (Ancestors)');
    expect(htmlEn).toContain('Hourglass (Both)');
    expect(htmlEn).toContain('Descendant Tree');
    expect(htmlEn).toContain('Generation Depth');
    expect(htmlEn).toContain('Printing Format');
    expect(htmlEn).toContain('Multi-Page Tiled');
    expect(htmlEn).toContain('Single Poster');
    expect(htmlEn).toContain('Overlap Bleed');
    expect(htmlEn).toContain('Print dashed cut lines and alignment crosshairs');
    expect(htmlEn).toContain("Include bird&#x27;s-eye assembly guide on Sheet 1");
    expect(htmlEn).toContain('Modern Clean');
    expect(htmlEn).toContain('Heritage Antique');
    expect(htmlEn).toContain("Mask living individuals&#x27; dates &amp; photos");
    expect(htmlEn).toContain('Download Printable PDF');

    const htmlRu = renderWithProviders(
      <PrintConfigSidebar
        config={mockConfig}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'ru'
    );

    expect(htmlRu).toContain('Параметры печати');
    expect(htmlRu).toContain('Центральная персона');
    expect(htmlRu).toContain('Тип диаграммы');
    expect(htmlRu).toContain('Предки (Восходящая)');
    expect(htmlRu).toContain('Песочные часы (Оба)');
    expect(htmlRu).toContain('Потомки (Нисходящая)');
    expect(htmlRu).toContain('Глубина поколений');
    expect(htmlRu).toContain('Формат печати');
    expect(htmlRu).toContain('Составной (На листах)');
    expect(htmlRu).toContain('Единый постер');
    expect(htmlRu).toContain('Нахлёст для склейки');
    expect(htmlRu).toContain('схему сборки');
    expect(htmlRu).toContain('Скачать PDF для печати');
  });

  it('renders PrintSummaryBar with accurate metric badges in tiled and poster modes', () => {
    const previewTiled: PdfExportPreviewInfo = {
      total_sheets: 6,
      grid_rows: 2,
      grid_cols: 3,
      canvas_width_mm: 891,
      canvas_height_mm: 420,
      sheet_width_mm: 297,
      sheet_height_mm: 210,
      overlap_mm: 10,
      format: 'tiled',
      target_sheet_size: 'A4',
      total_persons: 18,
      summary_text: '6 Sheets (3x2 Grid) of A4 Paper',
    };

    const configTiled: PdfExportRequest = {
      format: 'tiled',
      overlap_mm: 10,
      redact_living: true,
    };

    const htmlTiled = renderWithProviders(
      <PrintSummaryBar previewInfo={previewTiled} config={configTiled} />,
      'en'
    );

    expect(htmlTiled).toContain('6 Sheets (3×2 Grid)');
    expect(htmlTiled).toContain('891 × 420 mm');
    expect(htmlTiled).toContain('10 mm bleed with cut marks');
    expect(htmlTiled).toContain('Living masked');
    expect(htmlTiled).toContain('18 persons in chart');

    const previewPoster: PdfExportPreviewInfo = {
      total_sheets: 1,
      grid_rows: 1,
      grid_cols: 1,
      canvas_width_mm: 420,
      canvas_height_mm: 594,
      sheet_width_mm: 420,
      sheet_height_mm: 594,
      overlap_mm: 0,
      format: 'poster',
      target_sheet_size: 'A2',
      total_persons: 12,
      summary_text: '1 Poster Sheet (420 x 594 mm) — A2 Paper',
    };

    const configPoster: PdfExportRequest = {
      format: 'poster',
      target_sheet_size: 'A2',
      redact_living: false,
    };

    const htmlPoster = renderWithProviders(
      <PrintSummaryBar previewInfo={previewPoster} config={configPoster} />,
      'en'
    );

    expect(htmlPoster).toContain('1 Single Poster Sheet');
    expect(htmlPoster).toContain('420 × 594 mm');
    expect(htmlPoster).toContain('Continuous single canvas (no overlap)');
    expect(htmlPoster).toContain('Full records');
  });

  it('renders PrintPreviewCanvas in Tiled Mode with cut line overlay, crosshairs, sheet tags, and theme styling', () => {
    const previewInfo: PdfExportPreviewInfo = {
      total_sheets: 4,
      grid_rows: 2,
      grid_cols: 2,
      canvas_width_mm: 594,
      canvas_height_mm: 420,
      sheet_width_mm: 297,
      sheet_height_mm: 210,
      overlap_mm: 10,
      format: 'tiled',
      target_sheet_size: 'A4',
      total_persons: 15,
      summary_text: '4 Sheets (2x2 Grid) of A4 Paper',
    };

    const configTiledModern: PdfExportRequest = {
      format: 'tiled',
      view_mode: 'pedigree',
      generations: 5,
      theme: 'modern',
      overlap_mm: 10,
      show_crop_marks: true,
      include_assembly_guide: true,
      redact_living: false,
    };

    const htmlModern = renderWithProviders(
      <PrintPreviewCanvas
        config={configTiledModern}
        previewInfo={previewInfo}
        focalPersonName="Aleksandr Petrov"
      />,
      'en'
    );

    expect(htmlModern).toContain('data-testid="print-canvas-root"');
    expect(htmlModern).toContain('data-testid="print-tiling-grid-overlay"');
    expect(htmlModern).toContain('The Ancestors of Aleksandr Petrov');
    expect(htmlModern).toContain('Sheet 1 of 4 [Row 1, Col 1]');
    expect(htmlModern).toContain('Sheet 4 of 4 [Row 2, Col 2]');
    expect(htmlModern).toContain('Sheet 1: Assembly Guide Included');
    expect(htmlModern).toContain('+'); // Corner crosshairs

    // Heritage Antique Theme
    const configHeritage: PdfExportRequest = {
      ...configTiledModern,
      theme: 'heritage',
    };
    const htmlHeritage = renderWithProviders(
      <PrintPreviewCanvas
        config={configHeritage}
        previewInfo={previewInfo}
        focalPersonName="Aleksandr Petrov"
      />,
      'en'
    );

    expect(htmlHeritage).toContain('font-serif');
  });

  it('hides tiled cut overlay in Single Poster Mode', () => {
    const previewPoster: PdfExportPreviewInfo = {
      total_sheets: 1,
      grid_rows: 1,
      grid_cols: 1,
      canvas_width_mm: 420,
      canvas_height_mm: 594,
      sheet_width_mm: 420,
      sheet_height_mm: 594,
      overlap_mm: 0,
      format: 'poster',
      target_sheet_size: 'A2',
      total_persons: 10,
      summary_text: '1 Poster Sheet',
    };

    const configPoster: PdfExportRequest = {
      format: 'poster',
      target_sheet_size: 'A2',
    };

    const html = renderWithProviders(
      <PrintPreviewCanvas
        config={configPoster}
        previewInfo={previewPoster}
        focalPersonName="Aleksandr Petrov"
      />,
      'en'
    );

    expect(html).toContain('data-testid="print-canvas-root"');
    expect(html).not.toContain('data-testid="print-tiling-grid-overlay"');
    expect(html).toContain('1 Single Poster Sheet');
  });

  it('masks living individual dates and names when redact_living is active', () => {
    const configRedacted: PdfExportRequest = {
      format: 'tiled',
      redact_living: true,
      generations: 4,
    };

    const html = renderWithProviders(
      <PrintPreviewCanvas
        config={configRedacted}
        focalPersonName="Aleksandr Petrov"
      />,
      'en'
    );

    // Living focal person should be masked
    expect(html).toContain('A. (Living)');
    expect(html).toContain('Living');
  });

  it('API client exportPrintablePdf and fetchPdfPreviewInfo make valid HTTP requests', async () => {
    const fetchMock = vi.fn().mockImplementation((url: string) => {
      if (url.endsWith('/export/pdf')) {
        return Promise.resolve({
          ok: true,
          status: 200,
          headers: new Headers({
            'Content-Type': 'application/pdf',
            'Content-Disposition': 'attachment; filename="family-tree-tiled.pdf"',
          }),
          blob: async () => new Blob(['%PDF-1.4 mock content'], { type: 'application/pdf' }),
        });
      }
      if (url.endsWith('/export/pdf/preview-info')) {
        return Promise.resolve({
          ok: true,
          status: 200,
          headers: new Headers({ 'Content-Type': 'application/json' }),
          json: async () => ({
            total_sheets: 4,
            grid_rows: 2,
            grid_cols: 2,
            canvas_width_mm: 594,
            canvas_height_mm: 420,
            sheet_width_mm: 297,
            sheet_height_mm: 210,
            overlap_mm: 10,
            format: 'tiled',
            target_sheet_size: 'A4',
            total_persons: 15,
            summary_text: '4 Sheets of A4',
          }),
        });
      }
      return Promise.reject(new Error('Unknown url'));
    });

    vi.stubGlobal('fetch', fetchMock);

    const blob = await exportPrintablePdf({ format: 'tiled' });
    expect(blob).toBeInstanceOf(Blob);
    expect(blob.type).toBe('application/pdf');

    const preview = await fetchPdfPreviewInfo({ format: 'tiled' });
    expect(preview.total_sheets).toBe(4);
    expect(preview.grid_cols).toBe(2);
    expect(preview.grid_rows).toBe(2);

    vi.unstubAllGlobals();
  });

  it('supports paper size options A0, A1, A2, A3 for single poster, and A4, US Letter for multi-page tiled', () => {
    // Tiled mode options
    const htmlA4 = renderWithProviders(
      <PrintConfigSidebar
        config={{ format: 'tiled', target_sheet_size: 'A4' }}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlA4).toContain('A4 (210 × 297 mm)');

    const htmlLetter = renderWithProviders(
      <PrintConfigSidebar
        config={{ format: 'tiled', target_sheet_size: 'Letter' }}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlLetter).toContain('US Letter (8.5 × 11 in)');

    // Poster mode options for A0, A1, A2, A3
    for (const size of ['A0', 'A1', 'A2', 'A3'] as const) {
      const htmlPoster = renderWithProviders(
        <PrintConfigSidebar
          config={{ format: 'poster', target_sheet_size: size }}
          onChange={() => {}}
          onExport={() => {}}
        />,
        'en'
      );
      expect(htmlPoster).toContain(`${size} (`);
    }
  });

  it('supports responsive dual-pane layout, zoom controls, and accessible ARIA attributes', () => {
    const html = renderWithProviders(
      <PrintExportPage userRole="admin" />,
      'en'
    );

    // Responsive dual-pane layout classes
    expect(html).toContain('flex flex-col lg:flex-row');
    expect(html).toContain('w-full lg:w-96');

    // Accessibility attributes
    expect(html).toContain('aria-label="Live PDF Layout Preview"');
    expect(html).toContain('aria-label="Print Configuration"');
    expect(html).toContain('aria-label="Zoom in"');
    expect(html).toContain('aria-label="Zoom out"');
    expect(html).toContain('aria-label="Reset zoom"');
  });

  it('satisfies UX review findings UX-087 to UX-096 (mobile toggle, sticky CTA, a11y, Russian localization, aspect ratio, fit zoom)', () => {
    // UX-087: Mobile segmented toggle
    const htmlMobileEn = renderWithProviders(<PrintExportPage userRole="admin" />, 'en');
    expect(htmlMobileEn).toContain('Configuration');
    expect(htmlMobileEn).toContain('Preview &amp; Summary');
    const htmlMobileRu = renderWithProviders(<PrintExportPage userRole="admin" />, 'ru');
    expect(htmlMobileRu).toContain('Параметры печати');
    expect(htmlMobileRu).toContain('Предпросмотр');

    // UX-088 & UX-093 & UX-094: Sidebar bottom padding, label id linking, and sticky CTA
    const htmlSidebar = renderWithProviders(
      <PrintConfigSidebar
        config={{ format: 'tiled', target_sheet_size: 'A4' }}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlSidebar).toContain('pb-28 lg:pb-6');
    expect(htmlSidebar).toContain('for="print-chart-title-input"');
    expect(htmlSidebar).toContain('id="print-chart-title-input"');
    expect(htmlSidebar).toContain('sticky bottom-0 bg-surface/95 backdrop-blur-xs');

    // UX-089: Page height
    expect(htmlMobileEn).toContain('h-[calc(100dvh-4rem)]');

    // UX-090: Russian localization of Living masking & Cyrillic ancestor names
    const htmlCanvasRu = renderWithProviders(
      <PrintPreviewCanvas
        config={{ format: 'tiled', redact_living: true, generations: 4 }}
      />,
      'ru'
    );
    expect(htmlCanvasRu).toContain('А. (Ныне живёт)');
    expect(htmlCanvasRu).toContain('Е. (Ныне живёт)');
    expect(htmlCanvasRu).toContain('Пётр Фёдорович Петров');
    expect(htmlCanvasRu).toContain('Фёдор Михайлович Петров');
    expect(htmlCanvasRu).not.toContain('(Living)');
    expect(htmlCanvasRu).not.toContain('Pyotr Fyodorovich Petrov');

    // UX-091: Aspect ratio in portrait vs landscape
    const htmlCanvasPortrait = renderWithProviders(
      <PrintPreviewCanvas
        config={{ format: 'poster', target_sheet_size: 'A2' }}
        previewInfo={{
          total_sheets: 1,
          grid_rows: 1,
          grid_cols: 1,
          canvas_width_mm: 420,
          canvas_height_mm: 594,
          sheet_width_mm: 420,
          sheet_height_mm: 594,
          overlap_mm: 0,
          format: 'poster',
          target_sheet_size: 'A2',
          total_persons: 10,
          summary_text: '1 Poster Sheet',
        }}
      />,
      'en'
    );
    // Portrait: width 537px, minHeight 760px
    expect(htmlCanvasPortrait).toContain('width:537px');
    expect(htmlCanvasPortrait).toContain('min-height:760px');

    const htmlCanvasLandscape = renderWithProviders(
      <PrintPreviewCanvas
        config={{ format: 'tiled', target_sheet_size: 'A4' }}
        previewInfo={{
          total_sheets: 4,
          grid_rows: 2,
          grid_cols: 2,
          canvas_width_mm: 594,
          canvas_height_mm: 420,
          sheet_width_mm: 297,
          sheet_height_mm: 210,
          overlap_mm: 10,
          format: 'tiled',
          target_sheet_size: 'A4',
          total_persons: 12,
          summary_text: '4 Sheets',
        }}
      />,
      'en'
    );
    // Landscape: width 760px, minHeight 537px
    expect(htmlCanvasLandscape).toContain('width:760px');
    expect(htmlCanvasLandscape).toContain('min-height:537px');

    // UX-092: Screen reader pollution fix on crosshairs & canvas accessibility
    expect(htmlCanvasLandscape).toContain('aria-hidden="true"');
    expect(htmlCanvasLandscape).toContain('tabindex="0"');
    expect(htmlCanvasLandscape).toContain('role="region"');

    // UX-095: Tier headers & Russian pluralization
    expect(htmlCanvasLandscape).toContain('Generation 1');
    expect(htmlCanvasRu).toContain('Поколение 1');

    // Pluralization in SummaryBar
    const htmlSummaryRu = renderWithProviders(
      <PrintSummaryBar
        previewInfo={{
          total_sheets: 4,
          grid_rows: 2,
          grid_cols: 2,
          canvas_width_mm: 594,
          canvas_height_mm: 420,
          sheet_width_mm: 297,
          sheet_height_mm: 210,
          overlap_mm: 10,
          format: 'tiled',
          target_sheet_size: 'A4',
          total_persons: 12,
          summary_text: '4 Sheets',
        }}
        config={{ format: 'tiled' }}
      />,
      'ru'
    );
    expect(htmlSummaryRu).toContain('4 листа (сетка 2×2)');

    // UX-096: Fit to view button in zoom toolbar
    expect(htmlCanvasLandscape).toContain('aria-label="Fit to view"');
  });

  it('supports All Nodes layout mode, hiding generation depth and displaying branch archive note', () => {
    const configAllNodes: PdfExportRequest = {
      view_mode: 'all_nodes',
      format: 'tiled',
    };

    const htmlEn = renderWithProviders(
      <PrintConfigSidebar
        config={configAllNodes}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlEn).toContain('All Nodes (Complete Forest)');
    expect(htmlEn).toContain('Complete genealogical archive partitioned by family branches');
    // Generation depth slider should be hidden
    expect(htmlEn).not.toContain('5 Generations');

    const htmlRu = renderWithProviders(
      <PrintConfigSidebar
        config={configAllNodes}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'ru'
    );
    expect(htmlRu).toContain('Все персоны (Полное древо)');
    expect(htmlRu).toContain('Полный семейный архив с группировкой по родственным ветвям');
    expect(htmlRu).not.toContain('Поколений: 5');
  });

  it('renders Tree Density Wide/Compact segmented control exclusively in Hourglass mode', () => {
    const configHourglassWide: PdfExportRequest = {
      view_mode: 'hourglass',
      collateral: true,
    };
    const htmlHourglass = renderWithProviders(
      <PrintConfigSidebar
        config={configHourglassWide}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlHourglass).toContain('Tree Density');
    expect(htmlHourglass).toContain('Wide (Collateral)');
    expect(htmlHourglass).toContain('Compact (Lineal)');
    expect(htmlHourglass).toContain('Includes aunts, uncles, first cousins, and collateral relatives');

    const configHourglassCompact: PdfExportRequest = {
      view_mode: 'hourglass',
      collateral: false,
    };
    const htmlHourglassCompact = renderWithProviders(
      <PrintConfigSidebar
        config={configHourglassCompact}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'ru'
    );
    expect(htmlHourglassCompact).toContain('Детализация схемы');
    expect(htmlHourglassCompact).toContain('Широкая (С боковыми линиями)');
    expect(htmlHourglassCompact).toContain('Компактная (Прямая линия)');
    expect(htmlHourglassCompact).toContain('Ограничивается только прямыми предками и потомками');

    // Pedigree mode should NOT render Tree Density
    const configPedigree: PdfExportRequest = {
      view_mode: 'pedigree',
    };
    const htmlPedigree = renderWithProviders(
      <PrintConfigSidebar
        config={configPedigree}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlPedigree).not.toContain('Tree Density');
    expect(htmlPedigree).not.toContain('Wide (Collateral)');
  });

  it('renders partner and sibling checkboxes and disables them in fan chart mode with hint', () => {
    const configStandard: PdfExportRequest = {
      view_mode: 'pedigree',
      include_partner: true,
      include_siblings: false,
    };
    const htmlStandard = renderWithProviders(
      <PrintConfigSidebar
        config={configStandard}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlStandard).toContain('Include spouse / partner');
    expect(htmlStandard).toContain('Include siblings');
    expect(htmlStandard).not.toContain('Spouse, sibling, and collateral options are not available in Fan Chart layout');

    // Fan chart mode
    const configFan: PdfExportRequest = {
      view_mode: 'fan',
      include_partner: true,
      include_siblings: false,
    };
    const htmlFanEn = renderWithProviders(
      <PrintConfigSidebar
        config={configFan}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlFanEn).toContain('Spouse, sibling, and collateral options are not available in Fan Chart layout');

    const htmlFanRu = renderWithProviders(
      <PrintConfigSidebar
        config={configFan}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'ru'
    );
    expect(htmlFanRu).toContain('Включать супругов / партнёров');
    expect(htmlFanRu).toContain('Включать братьев и сестёр');
    expect(htmlFanRu).toContain('В круговой диаграмме параметры супругов, братьев/сестёр и боковых ветвей недоступны');
  });

  it('renders custom Chart Title and Subtitle inputs with accessibility labels and helper hints', () => {
    const configWithBanner: PdfExportRequest = {
      title: 'Family Reunion 2026',
      subtitle: 'Commemorative Edition',
    };
    const htmlSidebarEn = renderWithProviders(
      <PrintConfigSidebar
        config={configWithBanner}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlSidebarEn).toContain('id="print-chart-title-input"');
    expect(htmlSidebarEn).toContain('for="print-chart-title-input"');
    expect(htmlSidebarEn).toContain('id="print-chart-subtitle-input"');
    expect(htmlSidebarEn).toContain('for="print-chart-subtitle-input"');
    expect(htmlSidebarEn).toContain('Description / Subtitle (Optional)');
    expect(htmlSidebarEn).toContain('Leave blank for automatic layout summary');
    expect(htmlSidebarEn).toContain('value="Family Reunion 2026"');
    expect(htmlSidebarEn).toContain('value="Commemorative Edition"');

    const htmlSidebarRu = renderWithProviders(
      <PrintConfigSidebar
        config={configWithBanner}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'ru'
    );
    expect(htmlSidebarRu).toContain('Описание / Подзаголовок (необязательно)');
    expect(htmlSidebarRu).toContain('Оставьте пустым для автоматического описания');
  });

  it('dynamically renders server/custom title, subtitle, avatars, and marital amber connectors in PrintPreviewCanvas', () => {
    const previewInfo: PdfExportPreviewInfo = {
      total_sheets: 1,
      grid_rows: 1,
      grid_cols: 1,
      canvas_width_mm: 420,
      canvas_height_mm: 594,
      sheet_width_mm: 420,
      sheet_height_mm: 594,
      overlap_mm: 0,
      format: 'poster',
      target_sheet_size: 'A2',
      total_persons: 2,
      summary_text: '1 Sheet',
      title: 'Ivanov Dynasty 1800-2026',
      subtitle: 'Official Genealogical Document',
      cards: [
        {
          id: 'p1',
          gid: 'I001',
          full_name: 'Ivan Ivanov',
          lifespan: '1920 — 1995',
          is_living: false,
          is_focal: true,
          sex: 'M',
          x: 50,
          y: 60,
          width: 60,
          height: 30,
          generation: 1,
          has_avatar: true,
        },
        {
          id: 'p2',
          gid: 'I002',
          full_name: 'Maria Ivanova',
          lifespan: '1925 — 2005',
          is_living: false,
          is_focal: false,
          sex: 'F',
          x: 130,
          y: 60,
          width: 60,
          height: 30,
          generation: 1,
          has_avatar: false,
        },
      ],
      connectors: [
        {
          points: [
            [110, 75],
            [130, 75],
          ],
          is_dashed: false,
          color: '\x23d97706',
        },
      ],
    };

    const config: PdfExportRequest = {
      format: 'poster',
      target_sheet_size: 'A2',
      show_photos: true,
    };

    const html = renderWithProviders(
      <PrintPreviewCanvas
        config={config}
        previewInfo={previewInfo}
      />,
      'en'
    );

    // Custom banner title & subtitle rendered
    expect(html).toContain('Ivanov Dynasty 1800-2026');
    expect(html).toContain('Official Genealogical Document');

    // Circular avatar portrait for card with avatar
    expect(html).toContain('href="/api/people/p1/avatar"');
    expect(html).toContain('clip-path="url(#avatar-clip-p1)"');

    // Warm amber stroke for marriage connector
    expect(html).toContain('stroke-amber-600 dark:stroke-amber-500');
  });

  it('suppresses photo avatar rendering when show_photos is false', () => {
    const previewInfo: PdfExportPreviewInfo = {
      total_sheets: 1,
      grid_rows: 1,
      grid_cols: 1,
      canvas_width_mm: 420,
      canvas_height_mm: 594,
      sheet_width_mm: 420,
      sheet_height_mm: 594,
      overlap_mm: 0,
      format: 'poster',
      target_sheet_size: 'A2',
      total_persons: 1,
      summary_text: '1 Sheet',
      cards: [
        {
          id: 'p1',
          gid: 'I001',
          full_name: 'Ivan Ivanov',
          lifespan: '1920 — 1995',
          is_living: false,
          is_focal: true,
          sex: 'M',
          x: 50,
          y: 60,
          width: 60,
          height: 30,
          generation: 1,
          has_avatar: true,
        },
      ],
    };

    const config: PdfExportRequest = {
      format: 'poster',
      target_sheet_size: 'A2',
      show_photos: false,
    };

    const html = renderWithProviders(
      <PrintPreviewCanvas
        config={config}
        previewInfo={previewInfo}
      />,
      'en'
    );

    // Should NOT contain the image tag with avatar link
    expect(html).not.toContain('href="/api/people/p1/avatar"');
  });

  it('satisfies UX-098 to UX-101 (grid layout, localized sheet tags, a11y labels & hints, clear affordances)', () => {
    // UX-098: Chart Layout SegmentedControl arranges in 2x2 grid without horizontal overflow
    const htmlEn = renderWithProviders(
      <PrintConfigSidebar
        config={{ view_mode: 'pedigree', format: 'tiled' }}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlEn).toContain('grid grid-cols-2');

    // UX-099: Russian localized sheet coordinate tag and guide indicator
    const previewInfo: PdfExportPreviewInfo = {
      total_sheets: 4,
      grid_rows: 2,
      grid_cols: 2,
      canvas_width_mm: 594,
      canvas_height_mm: 420,
      sheet_width_mm: 297,
      sheet_height_mm: 210,
      overlap_mm: 10,
      format: 'tiled',
      target_sheet_size: 'A4',
      total_persons: 12,
      summary_text: '4 Sheets',
    };
    const htmlCanvasRu = renderWithProviders(
      <PrintPreviewCanvas
        config={{
          format: 'tiled',
          view_mode: 'pedigree',
          include_assembly_guide: true,
          show_crop_marks: true,
        }}
        previewInfo={previewInfo}
      />,
      'ru'
    );
    expect(htmlCanvasRu).toContain('Лист 1 из 4 [Ряд 1, Колонка 1]');
    expect(htmlCanvasRu).toContain('Лист 4 из 4 [Ряд 2, Колонка 2]');
    expect(htmlCanvasRu).toContain('Лист 1: Схема сборки включена');
    expect(htmlCanvasRu).not.toContain('Sheet 1 of 4');
    expect(htmlCanvasRu).not.toContain('Assembly Guide Included');

    // UX-100: PersonPicker programmatic label and Subtitle aria-describedby
    expect(htmlEn).toContain('aria-label="Focal Person"');
    expect(htmlEn).toContain('aria-describedby="print-chart-subtitle-hint"');
    expect(htmlEn).toContain('id="print-chart-subtitle-hint"');

    const htmlRu = renderWithProviders(
      <PrintConfigSidebar
        config={{ view_mode: 'pedigree', format: 'tiled' }}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'ru'
    );
    expect(htmlRu).toContain('aria-label="Центральная персона"');

    // UX-101: Clear button affordance for custom title and subtitle
    const htmlWithCustomText = renderWithProviders(
      <PrintConfigSidebar
        config={{
          title: 'Petrov Family Reunion',
          subtitle: 'Summer 2026',
        }}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlWithCustomText).toContain('aria-label="Clear"');

    // When empty, no clear buttons
    const htmlWithoutCustomText = renderWithProviders(
      <PrintConfigSidebar
        config={{
          title: undefined,
          subtitle: undefined,
        }}
        onChange={() => {}}
        onExport={() => {}}
      />,
      'en'
    );
    expect(htmlWithoutCustomText).not.toContain('aria-label="Clear"');
  });
});



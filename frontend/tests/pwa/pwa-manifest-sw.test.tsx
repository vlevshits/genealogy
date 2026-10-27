import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { PwaInstallPrompt, PwaOfflineBanner } from '../../src/components/pwa';
import { LivingBadge } from '../../src/components/domain';

function renderWithProviders(node: React.ReactElement): string {
  return renderToStaticMarkup(
    <ThemeProvider>
      <I18nProvider>{node}</I18nProvider>
    </ThemeProvider>
  );
}

describe('Phase 1.13 PWA Manifest, Privacy-Preserving Service Worker & Mobile Banner UI', () => {
  it('provides a valid manifest.webmanifest with standalone display, theme color, icons, and shortcuts', () => {
    const manifestPath = path.resolve(
      __dirname,
      '../../public/manifest.webmanifest'
    );
    expect(fs.existsSync(manifestPath)).toBe(true);
    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf-8'));

    expect(manifest.name).toBe('Levshits Family Archive');
    expect(manifest.short_name).toBe('Levshits Tree');
    expect(manifest.start_url).toBe('/?source=pwa');
    expect(manifest.display).toBe('standalone');
    expect(manifest.theme_color).toBe('#F8F9FA');
    expect(manifest.background_color).toBe('#F8F9FA');
    expect(Array.isArray(manifest.icons)).toBe(true);
    expect(manifest.icons.length).toBeGreaterThanOrEqual(5);
    expect(Array.isArray(manifest.shortcuts)).toBe(true);
    expect(manifest.shortcuts.length).toBe(2);

    // Verify all referenced icon files exist in public/
    for (const icon of manifest.icons) {
      const iconFile = path.resolve(
        __dirname,
        '../../public',
        icon.src.replace(/^\//, '')
      );
      expect(fs.existsSync(iconFile), `Missing icon file: ${icon.src}`).toBe(
        true
      );
    }
  });

  it('enforces strict privacy boundary in sw.js (never caches /api/* or /internal-media/*)', () => {
    const swPath = path.resolve(__dirname, '../../public/sw.js');
    expect(fs.existsSync(swPath)).toBe(true);
    const swSource = fs.readFileSync(swPath, 'utf-8');

    expect(swSource).toContain("const CACHE_NAME = 'genealogy-shell-v1'");
    expect(swSource).toContain("url.pathname.startsWith('/api/')");
    expect(swSource).toContain("url.pathname.startsWith('/internal-media/')");
    expect(swSource).toContain('event.respondWith(fetch(request))');
  });

  it('renders PwaOfflineBanner when offline and hides when online', () => {
    const offlineHtml = renderWithProviders(
      <PwaOfflineBanner forceOffline={true} />
    );
    expect(offlineHtml).toContain('pwa-offline-banner');
    expect(offlineHtml).toContain(
      'You are currently offline. Live family tree data requires a network connection.'
    );

    const onlineHtml = renderWithProviders(
      <PwaOfflineBanner forceOffline={false} />
    );
    expect(onlineHtml).toBe('');
  });

  it('renders PwaInstallPrompt and iOS instructions with design-system buttons', () => {
    const html = renderWithProviders(
      <PwaInstallPrompt forceVisible={true} forceIosMode={true} />
    );
    expect(html).toContain('pwa-install-prompt');
    expect(html).toContain('Install Family Archive');
    expect(html).toContain('pwa-ios-guide');
    expect(html).toContain('Tap the Share button in Safari.');
  });

  it('never renders a LivingBadge pill', () => {
    const badgeHtml = renderWithProviders(<LivingBadge />);
    expect(badgeHtml).toBe('');
  });
});

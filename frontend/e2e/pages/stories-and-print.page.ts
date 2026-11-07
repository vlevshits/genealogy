import { Locator, Page } from '@playwright/test';
import { BasePage } from './base.page';

export class StoryIntakePage extends BasePage {
  readonly pageContainer: Locator;
  readonly stepTracker: Locator;
  readonly speakerSelectorCard: Locator;
  readonly engineSelectorBar: Locator;
  readonly geminiEngineOption: Locator;
  readonly builtinEngineOption: Locator;
  readonly narrativeTextarea: Locator;
  readonly extractButton: Locator;
  readonly stagedWorkspace: Locator;
  readonly draftCanvas: Locator;
  readonly reconciliationCard: Locator;
  readonly commitButton: Locator;

  constructor(page: Page) {
    super(page);
    this.pageContainer = page.getByTestId('ai-story-intake-page');
    this.stepTracker = page.getByTestId('story-step-tracker');
    this.speakerSelectorCard = page.getByTestId('speaker-selector-card');
    this.engineSelectorBar = page.getByTestId('story-engine-selector-bar');
    this.geminiEngineOption = this.engineSelectorBar.getByRole('radio', {
      name: /Gemini 3\.8 Flash/i,
    });
    this.builtinEngineOption = this.engineSelectorBar.getByRole('radio', {
      name: /Built-in Offline|Встроенный офлайн/i,
    });
    this.narrativeTextarea = page.getByTestId('story-narrative-textarea');
    this.extractButton = page.getByTestId('extract-draft-subtree-btn');
    this.stagedWorkspace = page.getByTestId('staged-subtree-workspace');
    this.draftCanvas = page.getByTestId('draft-subtree-canvas');
    this.reconciliationCard = page.getByTestId('reconciliation-matrix-card');
    this.commitButton = page.getByTestId('commit-story-subtree-btn');
  }

  async goto() {
    await super.goto('/stories/intake');
  }

  getReconcileRow(tempId: string): Locator {
    return this.page.getByTestId(`reconcile-row-${tempId}`);
  }
}

export class SecurityMcpPage extends BasePage {
  readonly mcpPanel: Locator;
  readonly tokenNameInput: Locator;
  readonly createTokenButton: Locator;
  readonly freshTokenBanner: Locator;
  readonly rawTokenValue: Locator;
  readonly confirmRevokeButton: Locator;
  readonly dialog: Locator;

  constructor(page: Page) {
    super(page);
    this.mcpPanel = page.getByTestId('mcp-tokens-panel');
    this.tokenNameInput = page.getByTestId('mcp-token-name-input');
    this.createTokenButton = page.getByTestId('create-mcp-token-btn');
    this.freshTokenBanner = page.getByTestId('mcp-fresh-token-banner');
    this.rawTokenValue = page.getByTestId('mcp-raw-token-value');
    this.confirmRevokeButton = page.getByTestId('confirm-revoke-mcp-token-btn');
    this.dialog = page.getByRole('dialog');
  }

  async goto() {
    await super.goto('/settings/security');
  }

  getTokenRow(id: string): Locator {
    return this.page.getByTestId(`mcp-token-row-${id}`);
  }

  getRevokeButton(id: string): Locator {
    return this.page.getByTestId(`revoke-mcp-token-btn-${id}`);
  }
}

export class PrintStudioPage extends BasePage {
  readonly pageContainer: Locator;
  readonly configSidebar: Locator;
  readonly previewCanvasRoot: Locator;
  readonly downloadButton: Locator;
  readonly mobilePreviewTab: Locator;

  constructor(page: Page) {
    super(page);
    this.pageContainer = page.getByTestId('print-export-page');
    this.configSidebar = page.getByRole('complementary', {
      name: /Print Configuration|Параметры печати/i,
    });
    this.previewCanvasRoot = page.getByTestId('print-canvas-root');
    this.downloadButton = page
      .getByRole('button', { name: /Download PDF|Скачать PDF/i })
      .first();
    this.mobilePreviewTab = page.getByRole('radio', {
      name: /Layout Preview|Макет и превью/i,
    });
  }

  async goto() {
    await super.goto('/export/print');
  }

  async ensurePreviewCanvasVisible() {
    if (await this.mobilePreviewTab.isVisible()) {
      await this.mobilePreviewTab.click();
    }
  }
}

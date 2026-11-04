import { test, expect } from './fixtures/test-base';
import { WorkbenchPage } from './pages/workbench.page';

test.describe('Dev UI Workbench & Design System Primitives', () => {
  test.beforeEach(async ({ mockUnauthenticatedState }) => {
    await mockUnauthenticatedState();
  });

  test('renders design system workbench and components showcase', async ({ page }) => {
    const workbench = new WorkbenchPage(page);
    await workbench.goto();

    await expect(workbench.pageHeader).toBeVisible();
    await expect(
      page.getByText(/Design System & Component Workbench|Дизайн-система и стенд/i).first()
    ).toBeVisible();
  });

  test('opens and closes modal dialog via keyboard and buttons', async ({ page }) => {
    const workbench = new WorkbenchPage(page);
    await workbench.goto();

    await workbench.openDialogButton.scrollIntoViewIfNeeded();
    await workbench.openDialogButton.click();

    // Modal dialog should be visible and have appropriate ARIA role
    await expect(workbench.dialog).toBeVisible();

    // Press Escape to dismiss
    await page.keyboard.press('Escape');
    await expect(workbench.dialog).not.toBeVisible();
  });
});

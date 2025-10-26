import { test, expect } from '@playwright/test';

test('search mode summary reflects user selection', async ({ page }) => {
  await page.goto('/');

  const searchInput = page.getByPlaceholder('Search threads...');
  await searchInput.click();
  await searchInput.fill('scheduler');

  const summary = page.locator('[data-testid="search-mode-summary"]');
  await expect(summary).toBeVisible();
  await expect(summary).toContainText('Hybrid');

  const modeButton = page.getByRole('button', { name: 'Change search mode' });
  await modeButton.click();
  await page.getByRole('menuitemradio', { name: /Semantic Only/i }).click();

  await expect(summary).toContainText('Semantic');
});

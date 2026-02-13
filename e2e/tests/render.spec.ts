import { test, expect } from '@playwright/test';

test.describe('render WASM demo', () => {
  test('triangle renders on canvas, no error UI, screenshot', async ({ page }) => {
    await page.goto('/wasm-demo/', { waitUntil: 'networkidle', timeout: 20000 });
    const canvas = page.locator('#canvas');
    await expect(canvas).toBeVisible({ timeout: 20000 });
    await expect(canvas).toHaveAttribute('width', /^\d+$/);
    await expect(canvas).toHaveAttribute('height', /^\d+$/);
    const error = page.locator('#error');
    await expect(error).not.toHaveClass(/visible/);
    await expect(canvas).toHaveScreenshot('render-triangle.png', {
      maxDiffPixelRatio: 0.05,
    });
  });
});

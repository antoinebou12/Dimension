import { test, expect } from '@playwright/test';

const SECTIONS = [
  { path: 'linear', heading: /mathlib — Linear algebra/i },
  { path: 'gpu', heading: /mathlib — GPU/i },
  { path: 'ml', heading: /mathlib — ML/i },
  { path: 'optimization', heading: /mathlib — Optimization/i },
  { path: 'graph', heading: /mathlib — Graph/i },
  { path: 'distance', heading: /mathlib — Distance/i },
  { path: 'noise', heading: /mathlib — Noise/i },
  { path: 'transforms', heading: /mathlib — Transforms/i },
  { path: 'camera', heading: /mathlib — Camera/i },
  { path: 'viz', heading: /mathlib — Viz/i },
] as const;

test.describe('mathlib WASM demo index', () => {
  test('index loads with heading and all section links, screenshot', async ({
    page,
  }) => {
    await page.goto('/wasm-demo/');
    await expect(
      page.getByRole('heading', { name: /mathlib WASM/i })
    ).toBeVisible();
    await expect(
      page.getByRole('link', { name: 'Linear algebra' })
    ).toBeVisible();
    await expect(
      page.getByRole('link', { name: 'GPU (WebGPU)' })
    ).toBeVisible();
    await expect(
      page.getByRole('link', { name: /ML \(K-means/ })
    ).toBeVisible();
    await expect(
      page.getByRole('link', { name: /Optimization/ })
    ).toBeVisible();
    await expect(page.getByRole('link', { name: /Graph/ })).toBeVisible();
    await expect(page.getByRole('link', { name: 'Distance' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'Noise' })).toBeVisible();
    await expect(
      page.getByRole('link', { name: /Transforms/ })
    ).toBeVisible();
    await expect(page.getByRole('link', { name: 'Camera' })).toBeVisible();
    await expect(
      page.getByRole('link', { name: /Viz \(heightmap\)/ })
    ).toBeVisible();
    await expect(page).toHaveScreenshot('mathlib-index.png', {
      fullPage: true,
      maxDiffPixelRatio: 0.02,
    });
  });
});

test.describe('mathlib WASM demo sections', () => {
  for (const { path, heading } of SECTIONS) {
    test(`${path}: loads with key content, screenshot`, async ({ page }) => {
      await page.goto(`/wasm-demo/${path}/`);
      await page.waitForLoadState('networkidle');
      await expect(page.getByRole('heading', { name: heading })).toBeVisible();
      await expect(page).toHaveScreenshot(`mathlib-${path}.png`, {
        fullPage: true,
        maxDiffPixelRatio: 0.05,
      });
    });
  }
});

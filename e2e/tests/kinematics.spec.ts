import { test, expect } from '@playwright/test';

test.describe('kinematics WASM demo', () => {
  test('canvas is visible and no error UI after load', async ({ page }) => {
    await page.goto('/wasm-demo/');
    const canvas = page.locator('#canvas');
    await expect(canvas).toBeVisible();
    await expect(canvas).toHaveAttribute('width', '800');
    await expect(canvas).toHaveAttribute('height', '600');
    const error = page.locator('#error');
    await expect(error).not.toHaveClass(/visible/);
    await expect(canvas).toHaveScreenshot('kinematics-demo.png', {
      maxDiffPixelRatio: 0.05,
    });
  });

  test('hint text is shown', async ({ page }) => {
    await page.goto('/wasm-demo/');
    const hint = page.locator('#hint');
    await expect(hint).toBeVisible();
    await expect(hint).toContainText(/orbit|IK|zoom/i);
  });

  test('page title is set', async ({ page }) => {
    await page.goto('/wasm-demo/');
    await expect(page).toHaveTitle(/kinematics-demo|3-joint arm|IK/i);
  });
});

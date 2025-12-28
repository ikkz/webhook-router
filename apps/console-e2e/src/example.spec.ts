import { test, expect } from '@playwright/test';

test('has login', async ({ page }) => {
  await page.goto('/console');

  expect(await page.locator('body').innerText()).toContain('Login');
});

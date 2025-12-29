import { test, expect } from '@playwright/test';
import { login, getAuthCredentials } from './test-helpers';

test.describe('Authentication', () => {
  test('should successfully login with valid credentials', async ({ page }) => {
    const { username, password } = getAuthCredentials();

    await page.goto('/console#/login');

    // Fill in credentials
    await page.fill('input#username', username);
    await page.fill('input#password', password);

    // Submit form
    await page.click('button[type="submit"]');

    // Should redirect to the console home page
    await page.waitForURL(/.*#\/$/, { timeout: 10000 });

    // Verify we're on the home page
    await expect(page.locator('h2:has-text("Endpoints")')).toBeVisible();
  });

  test('should fail login with invalid credentials', async ({ page }) => {
    await page.goto('/console#/login');

    // Fill in invalid credentials
    await page.fill('input#username', 'invalid');
    await page.fill('input#password', 'wrongpassword');

    // Submit form
    await page.click('button[type="submit"]');

    // Should stay on login page or show error
    // Wait a bit to ensure any navigation would have happened
    await page.waitForTimeout(1000);

    // Should still see login form
    const passwordInput = page.locator('input[type="password"]');
    await expect(passwordInput).toBeVisible();
  });

  test('should persist authentication across page refresh', async ({ page }) => {
    await login(page);

    // Verify we're logged in
    await expect(page.locator('h2:has-text("Endpoints")')).toBeVisible();

    // Refresh the page
    await page.reload();

    // Should still be logged in
    await expect(page.locator('h2:has-text("Endpoints")')).toBeVisible();
  });

  test('should redirect to login when accessing protected route without auth', async ({ page, context }) => {
    // Clear all cookies to ensure we're not authenticated
    await context.clearCookies();
    // Also clear localStorage as token is stored there
    await page.addInitScript(() => {
      // @ts-expect-error localStorage is only available in the browser context
      window.localStorage.clear();
    });

    // Try to access protected route
    // Note: with HashRouter, we go to /#/endpoints
    await page.goto('/console#/endpoints');

    // Should redirect to login or show login page
    await page.waitForTimeout(1000);
    const passwordInput = page.locator('input[type="password"]');
    await expect(passwordInput).toBeVisible();
  });
});

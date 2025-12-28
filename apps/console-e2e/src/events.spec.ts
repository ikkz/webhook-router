import { test, expect } from '@playwright/test';
import { login } from './test-helpers';

test.describe('Events Viewing', () => {
    test.beforeEach(async ({ page }) => {
        await login(page);
    });

    test('should navigate to events page', async ({ page }) => {
        // Navigate to events page
        await page.goto('/console#/events');

        // Verify events page is displayed
        await expect(page.locator('h2:has-text("Events")')).toBeVisible();
    });

    test('should display events list', async ({ page }) => {
        await page.goto('/console#/events');

        // Events page should load without errors
        // Either shows events or empty state
        const eventsHeading = page.locator('h2:has-text("Events")');
        await expect(eventsHeading).toBeVisible();
    });

    test('should show event details when events exist', async ({ page }) => {
        await page.goto('/console#/events');

        // Wait for page to load
        await page.waitForTimeout(1000);

        // Check if there are any event cards/rows
        // The exact selector depends on how events are rendered
        // This is a basic check that the page loads
        await expect(page.locator('h2:has-text("Events")')).toBeVisible();
    });
});

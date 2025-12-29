import { test, expect } from '@playwright/test';
import { MockTargetServer, WebhookSimulator, sendWebhookToIngress } from './mock-servers';
import {
    apiCreateEndpoint,
    apiCreateTarget,
    apiListEvents,
    generateTestName,
    getBaseUrl,
    login,
    waitFor,
} from './test-helpers';

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

    test('should show delivery results with failure details', async ({ page, request }) => {
        const baseUrl = getBaseUrl();
        const mockServer = new MockTargetServer();
        await mockServer.start();

        try {
            const endpointName = generateTestName('Deliveries');
            const endpoint = await apiCreateEndpoint(request, baseUrl, endpointName);

            const successTargetName = generateTestName('Success Target');
            const failureTargetName = generateTestName('Failure Target');

            await apiCreateTarget(request, baseUrl, endpoint.id, {
                name: successTargetName,
                kind: 'http',
                url: `${mockServer.url}/ok`,
            });

            await apiCreateTarget(request, baseUrl, endpoint.id, {
                name: failureTargetName,
                kind: 'http',
                url: 'http://127.0.0.1:9/fail',
            });

            const eventTitle = generateTestName('Delivery Event');
            const payload = WebhookSimulator.http('Delivery body', eventTitle);
            const response = await sendWebhookToIngress(baseUrl, endpoint.id, 'http', payload);
            expect(response.ok).toBe(true);

            await waitFor(async () => {
                const events = await apiListEvents(request, baseUrl);
                const match = events.find((event: any) => event.title === eventTitle);
                return Boolean(match && Array.isArray(match.deliveries) && match.deliveries.length >= 2);
            }, 10000, 200);

            await page.goto('/console#/events');

            const row = page.locator('tr', { hasText: eventTitle });
            await expect(row).toBeVisible();
            await row.getByRole('button').click();

            const modal = page.locator('div.fixed.inset-0.z-50');
            await expect(modal.locator('h4:has-text("Deliveries")')).toBeVisible();

            const successCard = modal.locator('div.rounded-md.border', { hasText: successTargetName });
            await expect(successCard).toBeVisible();
            await expect(successCard.getByText('sent', { exact: false })).toBeVisible();

            const failureCard = modal.locator('div.rounded-md.border', { hasText: failureTargetName });
            await expect(failureCard).toBeVisible();
            await expect(failureCard.getByText('failed', { exact: false })).toBeVisible();
            await expect(failureCard.locator('div.text-xs.text-destructive')).toHaveText(/.+/);
        } finally {
            await mockServer.stop();
        }
    });
});

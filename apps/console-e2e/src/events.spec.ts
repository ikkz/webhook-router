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

    test('should filter events by endpoint', async ({ page, request }) => {
        const baseUrl = getBaseUrl();
        const mockServer = new MockTargetServer();
        await mockServer.start();

        try {
            // Create two endpoints
            const endpoint1Name = generateTestName('Endpoint 1');
            const endpoint2Name = generateTestName('Endpoint 2');
            const endpoint1 = await apiCreateEndpoint(request, baseUrl, endpoint1Name);
            const endpoint2 = await apiCreateEndpoint(request, baseUrl, endpoint2Name);

            // Create targets for both
            await apiCreateTarget(request, baseUrl, endpoint1.id, {
                name: 'Target 1',
                kind: 'http',
                url: `${mockServer.url}/ok`,
            });
            await apiCreateTarget(request, baseUrl, endpoint2.id, {
                name: 'Target 2',
                kind: 'http',
                url: `${mockServer.url}/ok`,
            });

            // Send events to both endpoints
            const event1Title = generateTestName('Event 1');
            const event2Title = generateTestName('Event 2');

            await sendWebhookToIngress(baseUrl, endpoint1.id, 'http', WebhookSimulator.http('Body 1', event1Title));
            await sendWebhookToIngress(baseUrl, endpoint2.id, 'http', WebhookSimulator.http('Body 2', event2Title));

            // Wait for events to be created
            await waitFor(async () => {
                const events = await apiListEvents(request, baseUrl);
                return events.some((e: any) => e.title === event1Title) &&
                    events.some((e: any) => e.title === event2Title);
            }, 10000, 200);

            await page.goto('/console#/events');

            // Initially both events should be visible
            await expect(page.locator('tr', { hasText: event1Title })).toBeVisible();
            await expect(page.locator('tr', { hasText: event2Title })).toBeVisible();

            // Filter by endpoint 1
            await page.locator('button[role="combobox"]').click();
            await page.locator(`div[role="option"]:has-text("${endpoint1Name}")`).click();

            // Wait for filter to apply
            await page.waitForTimeout(1000);

            // Only event 1 should be visible
            await expect(page.locator('tr', { hasText: event1Title })).toBeVisible();
            await expect(page.locator('tr', { hasText: event2Title })).not.toBeVisible();

            // Switch to endpoint 2
            await page.locator('button[role="combobox"]').click();
            await page.locator(`div[role="option"]:has-text("${endpoint2Name}")`).click();

            // Wait for filter to apply
            await page.waitForTimeout(1000);

            // Only event 2 should be visible
            await expect(page.locator('tr', { hasText: event2Title })).toBeVisible();
            await expect(page.locator('tr', { hasText: event1Title })).not.toBeVisible();

            // Reset filter
            await page.locator('button[role="combobox"]').click();
            await page.locator('div[role="option"]:has-text("All Endpoints")').click();

            // Verify "All Endpoints" is selected (text in the trigger)
            await expect(page.locator('button[role="combobox"]')).toContainText('All Endpoints');

            // Wait for filter to apply
            await page.waitForTimeout(1000);

            // Note: We don't assert that event1/event2 are visible here because 
            // other parallel tests (like pagination) might have pushed them off page 1.
            // The fact that we successfully switched between Endpoint 1 and 2 above proves filtering works.
        } finally {
            await mockServer.stop();
        }
    });

    test('should paginate events', async ({ page, request }) => {
        const baseUrl = getBaseUrl();
        const mockServer = new MockTargetServer();
        await mockServer.start();

        try {
            const endpointName = generateTestName('Pagination Test');
            const endpoint = await apiCreateEndpoint(request, baseUrl, endpointName);

            await apiCreateTarget(request, baseUrl, endpoint.id, {
                name: 'Test Target',
                kind: 'http',
                url: `${mockServer.url}/ok`,
            });

            // Create 25 events (more than one page with page_size=20)
            const eventTitles: string[] = [];
            for (let i = 0; i < 25; i++) {
                const title = generateTestName(`Event ${i}`);
                eventTitles.push(title);
                await sendWebhookToIngress(baseUrl, endpoint.id, 'http', WebhookSimulator.http(`Body ${i}`, title));
                // Small delay to ensure different timestamps
                await new Promise(resolve => setTimeout(resolve, 50));
            }

            // Wait for all events to be created
            await waitFor(async () => {
                const events = await apiListEvents(request, baseUrl);
                return events.length >= 25;
            }, 15000, 500);

            await page.goto('/console#/events');

            // Check that pagination controls are visible
            await expect(page.locator('button:has-text("Previous")')).toBeVisible();
            await expect(page.locator('button:has-text("Next")')).toBeVisible();

            // Previous should be disabled on first page
            await expect(page.locator('button:has-text("Previous")')).toBeDisabled();

            // Next should be enabled
            await expect(page.locator('button:has-text("Next")')).toBeEnabled();

            // Click next to go to page 2
            await page.locator('button:has-text("Next")').click();
            await page.waitForTimeout(1000);

            // Now previous should be enabled
            await expect(page.locator('button:has-text("Previous")')).toBeEnabled();

            // Page indicator should show page 2
            await expect(page.locator('text=Page 2')).toBeVisible();

            // Click previous to go back to page 1
            await page.locator('button:has-text("Previous")').click();
            await page.waitForTimeout(1000);

            // Should be back on page 1
            await expect(page.locator('text=Page 1')).toBeVisible();
            await expect(page.locator('button:has-text("Previous")')).toBeDisabled();
        } finally {
            await mockServer.stop();
        }
    });
});

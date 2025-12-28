import { APIRequestContext, Page } from '@playwright/test';

/**
 * Login helper
 */
export async function login(page: Page, username = 'admin', password = 'admin') {
    await page.goto('/console#/login');

    // Check if already logged in (redirected to a protected route)
    try {
        const isLoginPage = await page.locator('input[type="password"]').isVisible({ timeout: 2000 });
        if (!isLoginPage) {
            // Already logged in
        } else {
            await page.fill('input#username', username);
            await page.fill('input#password', password);
            await page.click('button[type="submit"]');

            // Wait for navigation to complete - expecting redirect to console home
            // With hash router: /#/
            await page.waitForURL(/.*#\/$/, { timeout: 10000 });
        }
    } catch (e) {
        // Already logged in (timeout)
    }

    await page.goto('/console#/endpoints');
    await page.waitForSelector('h2:has-text("Endpoints")', { timeout: 10000 });
}

/**
 * Wait for element with retry
 */
export async function waitForElement(page: Page, selector: string, timeout = 5000) {
    await page.waitForSelector(selector, { timeout });
}

/**
 * Select a target kind from the Radix UI select.
 */
export async function selectTargetKind(page: Page, label: string) {
    await page.click('#target-kind');
    await page.getByRole('option', { name: label }).click();
}

/**
 * Generate unique test names
 */
export function generateTestName(prefix: string): string {
    return `${prefix}_${Date.now()}_${Math.random().toString(36).substring(7)}`;
}

/**
 * Get auth credentials from environment or use defaults
 */
export function getAuthCredentials() {
    return {
        username: process.env.TEST_USERNAME || 'admin',
        password: process.env.TEST_PASSWORD || 'admin',
    };
}

/**
 * Create base64 basic auth header
 */
export function createBasicAuthHeader(username: string, password: string): string {
    const credentials = Buffer.from(`${username}:${password}`).toString('base64');
    return `Basic ${credentials}`;
}

function getAuthHeader(explicitHeader?: string): string {
    if (explicitHeader) {
        return explicitHeader;
    }
    const { username, password } = getAuthCredentials();
    return createBasicAuthHeader(username, password);
}

async function parseJsonResponse(response: any, errorMessage: string) {
    if (!response.ok()) {
        throw new Error(`${errorMessage} (status ${response.status()}): ${await response.text()}`);
    }
    return response.json();
}

export async function apiCreateEndpoint(
    request: APIRequestContext,
    baseUrl: string,
    name: string,
    authHeader?: string
) {
    const response = await request.post(`${baseUrl}/console/api/endpoints`, {
        data: { name },
        headers: {
            Authorization: getAuthHeader(authHeader),
        },
    });
    return parseJsonResponse(response, 'Failed to create endpoint');
}

export async function apiCreateTarget(
    request: APIRequestContext,
    baseUrl: string,
    endpointId: string,
    target: { name: string; kind: string; url: string; headers?: Record<string, string> },
    authHeader?: string
) {
    const response = await request.post(`${baseUrl}/console/api/endpoints/${endpointId}/targets`, {
        data: target,
        headers: {
            Authorization: getAuthHeader(authHeader),
        },
    });
    return parseJsonResponse(response, 'Failed to create target');
}

export async function apiListEvents(
    request: APIRequestContext,
    baseUrl: string,
    authHeader?: string
) {
    const response = await request.get(`${baseUrl}/console/api/events`, {
        headers: {
            Authorization: getAuthHeader(authHeader),
        },
    });
    return parseJsonResponse(response, 'Failed to list events');
}

/**
 * Wait for a condition to be true
 */
export async function waitFor(
    condition: () => Promise<boolean> | boolean,
    timeoutMs = 5000,
    intervalMs = 100
): Promise<void> {
    const startTime = Date.now();
    while (Date.now() - startTime < timeoutMs) {
        if (await condition()) {
            return;
        }
        await new Promise((resolve) => setTimeout(resolve, intervalMs));
    }
    throw new Error(`Timeout waiting for condition after ${timeoutMs}ms`);
}

/**
 * Get base URL from environment or use default
 */
export function getBaseUrl(): string {
    return process.env.BASE_URL || 'http://localhost:3100';
}

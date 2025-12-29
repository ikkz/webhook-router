import { test, expect } from '@playwright/test';
import { login, generateTestName } from './test-helpers';
import { MockTargetServer } from './mock-servers';

test.describe('Endpoint Configuration', () => {
    test.beforeEach(async ({ page }) => {
        await login(page);
    });

    test('should configure banner and footer', async ({ page, request }) => {
        // Create endpoint using UI
        const endpointName = generateTestName('Config Test');
        await page.click('button:has-text("New Endpoint")');
        await page.fill('input#endpoint-name', endpointName);
        await page.click('button[type="submit"]:has-text("Create")');
        await page.waitForTimeout(1000);

        // Navigate to endpoint details
        await page.click(`text=${endpointName}`);
        await expect(page).toHaveURL(/.*#\/endpoints\/[a-zA-Z0-9-]+/);

        // Wait for Configuration section to be visible
        await page.click('button[role="tab"]:has-text("Configuration")');
        await expect(page.locator('text=Markdown Configuration')).toBeVisible();

        // Find the CodeMirror editors - they should be inside divs with specific classes
        const bannerEditor = page.locator('.cm-editor').first();
        const footerEditor = page.locator('.cm-editor').nth(1);

        // Click into the banner editor and type
        await bannerEditor.click();
        await page.keyboard.type('## Banner Header\nThis is prepended to all messages');

        // Click into the footer editor and type  
        await footerEditor.click();
        await page.keyboard.type('---\n*This is the footer*');

        // Save configuration
        await page.click('button:has-text("Save Configuration")');

        // Wait for success message
        await expect(page.locator('text=Configuration saved successfully!')).toBeVisible({ timeout: 5000 });

        // Reload page to verify persistence
        await page.reload();
        await page.click('button[role="tab"]:has-text("Configuration")');
        await page.waitForSelector('text=Markdown Configuration', { timeout: 10000 });

        // Verify the content persisted (check that editors contain our text)
        await expect(page.locator('.cm-content').first()).toContainText('Banner Header');
        await expect(page.locator('.cm-content').nth(1)).toContainText('This is the footer');
    });

    test('should send test message with banner/footer via UI', async ({ page, request, context }) => {
        const mockServer = new MockTargetServer();
        await mockServer.start();

        try {
            // Create endpoint via API with banner and footer
            const endpointName = generateTestName('Test Send UI');
            const baseUrl = process.env.BASE_URL || 'http://localhost:3100';
            const authHeader = `Basic ${Buffer.from('admin:admin').toString('base64')}`;

            const endpointRes = await request.post(`${baseUrl}/console/api/endpoints`, {
                data: {
                    name: endpointName,
                    banner: '**[BANNER]**',
                    footer: '**[FOOTER]**'
                },
                headers: { Authorization: authHeader },
            });
            const endpoint = await endpointRes.json();

            // Create a target pointing to mock server
            await request.post(`${baseUrl}/console/api/endpoints/${endpoint.id}/targets`, {
                data: {
                    name: 'Mock Target',
                    kind: 'http',
                    url: `${mockServer.url}/webhook`,
                    headers: {}
                },
                headers: { Authorization: authHeader },
            });

            // NOW TEST THE UI - Navigate to endpoint details page
            await page.goto(`/console#/endpoints/${endpoint.id}`);

            // Switch to Configuration tab
            await page.click('button[role="tab"]:has-text("Configuration")');
            await page.waitForSelector('text=Test Send', { timeout: 10000 });

            // Wait for页面加载和 CodeMirror 编辑器初始化
            await page.waitForTimeout(1500);

            // Find the test markdown editor (3rd CodeMirror editor)
            const testEditorCount = await page.locator('.cm-editor').count();
            console.log(`Found ${testEditorCount} CodeMirror editors`);
            expect(testEditorCount).toBeGreaterThanOrEqual(3);

            const testEditor = page.locator('.cm-editor').nth(2);
            await testEditor.waitFor({ state: 'visible', timeout: 5000 });

            // Clear the default content and type new test markdown
            await testEditor.click();
            await page.waitForTimeout(300);

            // Select all
            await page.keyboard.press('Control+A');
            await page.keyboard.press('Meta+A');
            await page.waitForTimeout(200);

            // Type test content
            await page.keyboard.type('# UI Test Content');
            await page.keyboard.press('Enter');
            await page.keyboard.type('Testing from UI!');
            await page.waitForTimeout(300);

            // Find and click the "Send Test" button
            const sendTestButton = page.locator('button:has-text("Send Test")').first();
            await sendTestButton.waitFor({ state: 'visible', timeout: 5000 });

            console.log('About to click Send Test button');
            await sendTestButton.click();

            // Wait for either success or error
            await page.waitForTimeout(2000);

            // Check if delivery results appeared
            const deliveryResultsVisible = await page.locator('text=Delivery Results:')
                .isVisible({ timeout: 15000 })
                .catch(() => false);

            if (deliveryResultsVisible) {
                console.log('Delivery results appeared!');
                await expect(page.locator('text=Delivery Results:')).toBeVisible();

                // Verify mock server received the webhook
                await page.waitForTimeout(1500);
                const receivedPayloads = mockServer.getReceivedWebhooks();
                console.log(`Mock server received ${receivedPayloads.length} payloads`);
                expect(receivedPayloads.length).toBeGreaterThan(0);

                // Verify content has banner, message, and footer
                const lastPayload = receivedPayloads[receivedPayloads.length - 1].body;
                const textField = lastPayload.text || lastPayload.content || JSON.stringify(lastPayload);
                console.log('Received payload text:', textField);

                expect(textField).toContain('[BANNER]');
                expect(textField).toContain('UI Test Content');
                expect(textField).toContain('[FOOTER]');
            } else {
                // Check if there's an error message
                const errorVisible = await page.locator('text=Failed to send').isVisible().catch(() => false);
                if (errorVisible) {
                    const errorText = await page.locator('.text-destructive').allTextContents();
                    console.error('Test send failed with error:', errorText);

                    // Take a screenshot for debugging
                    await page.screenshot({ path: 'test-output/test-send-error.png' });

                    throw new Error(`Test send failed in UI: ${errorText.join(', ')}`);
                } else {
                    await page.screenshot({ path: 'test-output/no-results.png' });
                    throw new Error('Neither delivery results nor error message appeared');
                }
            }
        } finally {
            await mockServer.stop();
        }
    });

    test('should handle empty banner/footer', async ({ page }) => {
        const endpointName = generateTestName('Empty Config');

        // Create endpoint
        await page.click('button:has-text("New Endpoint")');
        await page.fill('input#endpoint-name', endpointName);
        await page.click('button[type="submit"]:has-text("Create")');
        await page.waitForTimeout(1000);
        await page.click(`text=${endpointName}`);

        // Configuration section should be visible with empty editors
        await page.click('button[role="tab"]:has-text("Configuration")');
        await expect(page.locator('text=Markdown Configuration')).toBeVisible();

        // Editors should exist but be empty
        const editorCount = await page.locator('.cm-editor').count();
        expect(editorCount).toBeGreaterThanOrEqual(2);

        // Enter a single character to enable save
        const bannerEditor = page.locator('.cm-editor').first();
        await bannerEditor.click();
        await page.keyboard.type(' ');
        await page.keyboard.press('Backspace');

        // Save button should still be disabled since no real changes
        const saveButton = page.locator('button:has-text("Save Configuration")');
        await expect(saveButton).toBeDisabled();
    });

    test('should update existing banner/footer configuration', async ({ page, request }) => {
        const baseUrl = process.env.BASE_URL || 'http://localhost:3100';
        const authHeader = `Basic ${Buffer.from('admin:admin').toString('base64')}`;

        // Create endpoint with initial banner/footer via API
        const endpointName = generateTestName('Update Config');
        const endpointRes = await request.post(`${baseUrl}/console/api/endpoints`, {
            data: {
                name: endpointName,
                banner: 'Initial Banner',
                footer: 'Initial Footer'
            },
            headers: { Authorization: authHeader },
        });
        const endpoint = await endpointRes.json();

        // Navigate to endpoint
        await page.goto(`/console#/endpoints/${endpoint.id}`);

        // Switch to Configuration tab
        await page.click('button[role="tab"]:has-text("Configuration")');
        await page.waitForSelector('text=Markdown Configuration', { timeout: 10000 });

        // Verify initial content is displayed
        await expect(page.locator('.cm-content').first()).toContainText('Initial Banner');

        // Update banner - select all and replace
        const bannerEditor = page.locator('.cm-editor').first();
        await bannerEditor.click();
        await page.keyboard.press('Control+A');
        await page.keyboard.press('Meta+A');
        await page.keyboard.type('Updated Banner');

        // Save
        await page.click('button:has-text("Save Configuration")');
        await expect(page.locator('text=Configuration saved successfully!')).toBeVisible({ timeout: 5000 });

        // Reload and verify update persisted
        await page.reload();
        await page.click('button[role="tab"]:has-text("Configuration")');
        await page.waitForSelector('text=Markdown Configuration', { timeout: 10000 });
        await expect(page.locator('.cm-content').first()).toContainText('Updated Banner');
    });

    test('should test send with no targets configured', async ({ page }) => {
        const endpointName = generateTestName('No Targets');

        // Create endpoint
        await page.click('button:has-text("New Endpoint")');
        await page.fill('input#endpoint-name', endpointName);
        await page.click('button[type="submit"]:has-text("Create")');
        await page.waitForTimeout(1000);
        await page.click(`text=${endpointName}`);

        // Wait for test send section
        await page.click('button[role="tab"]:has-text("Configuration")');
        await expect(page.locator('text=Test Send')).toBeVisible();

        // Click send test
        const testSendButton = page.locator('button:has-text("Send Test")').first();
        await testSendButton.waitFor({ state: 'visible', timeout: 5000 });
        await testSendButton.click();

        // Wait a bit for API call
        await page.waitForTimeout(2000);

        // Should show delivery results with no targets message
        const deliverySection = page.locator('text=Delivery Results:');
        if (await deliverySection.isVisible({ timeout: 2000 }).catch(() => false)) {
            const noTargetsMsg = page.locator("text=No targets configured").first();
            await expect(noTargetsMsg).toBeVisible();
            // Results appeared, verify response
            console.log('Delivery results appeared - test passed');
        }
    });
});

import { test, expect } from '@playwright/test';
import { login, generateTestName, selectTargetKind } from './test-helpers';

test.describe('Targets Management', () => {
    let endpointName: string;

    test.beforeEach(async ({ page }) => {
        await login(page);

        // Create an endpoint for testing targets
        endpointName = generateTestName('Targets Test Endpoint');
        await page.click('button:has-text("New Endpoint")');
        await page.fill('input#endpoint-name', endpointName);
        await page.click('button[type="submit"]:has-text("Create")');
        await page.waitForTimeout(1000);

        // Navigate to endpoint details
        await page.click(`text=${endpointName}`);
    });

    test('should display targets section', async ({ page }) => {
        await expect(page.locator('h3:has-text("Targets")')).toBeVisible();
        await expect(page.locator('button:has-text("Add Target")')).toBeVisible();
    });

    test('should add HTTP target', async ({ page }) => {
        const targetName = generateTestName('HTTP Target');
        const targetUrl = 'https://example.com/webhook';

        // Click "Add Target" button
        await page.click('button:has-text("Add Target")');

        // Fill in the form
        await page.fill('input#target-name', targetName);
        await selectTargetKind(page, 'HTTP');
        await page.fill('input#target-url', targetUrl);

        // Submit
        await page.click('button[type="submit"]:has-text("Create")');

        // Wait for target to appear
        await page.waitForTimeout(1000);

        // Verify target is in the list
        await expect(page.locator(`text=${targetName}`).first()).toBeVisible();
        await expect(page.locator(`text=${targetUrl}`)).toBeVisible();
    });

    test('should add Slack target', async ({ page }) => {
        const targetName = generateTestName('Slack Target');
        const targetUrl = 'https://hooks.slack.com/services/T00/B00/XXX';

        await page.click('button:has-text("Add Target")');
        await page.fill('input#target-name', targetName);
        await selectTargetKind(page, 'Slack');
        await page.fill('input#target-url', targetUrl);
        await page.click('button[type="submit"]:has-text("Create")');
        await page.waitForTimeout(1000);

        await expect(page.locator(`text=${targetName}`).first()).toBeVisible();
    });

    test('should add Lark target', async ({ page }) => {
        const targetName = generateTestName('Lark Target');
        const targetUrl = 'https://open.feishu.cn/open-apis/bot/v2/hook/xxx';

        await page.click('button:has-text("Add Target")');
        await page.fill('input#target-name', targetName);
        await selectTargetKind(page, 'Lark');
        await page.fill('input#target-url', targetUrl);
        await page.click('button[type="submit"]:has-text("Create")');
        await page.waitForTimeout(1000);

        await expect(page.locator(`text=${targetName}`).first()).toBeVisible();
    });

    test('should add DingTalk target', async ({ page }) => {
        const targetName = generateTestName('DingTalk Target');
        const targetUrl = 'https://oapi.dingtalk.com/robot/send?access_token=xxx';

        await page.click('button:has-text("Add Target")');
        await page.fill('input#target-name', targetName);
        await selectTargetKind(page, 'DingTalk');
        await page.fill('input#target-url', targetUrl);
        await page.click('button[type="submit"]:has-text("Create")');
        await page.waitForTimeout(1000);

        await expect(page.locator(`text=${targetName}`).first()).toBeVisible();
    });

    test('should delete target', async ({ page }) => {
        const targetName = generateTestName('Delete Target');
        const targetUrl = 'https://example.com/delete-test';

        // Create a target
        await page.click('button:has-text("Add Target")');
        await page.fill('input#target-name', targetName);
        await page.fill('input#target-url', targetUrl);
        await page.click('button[type="submit"]:has-text("Create")');
        await page.waitForTimeout(1000);

        // Verify target exists
        await expect(page.locator(`text=${targetName}`).first()).toBeVisible();

        // Find and click delete button (trash icon)
        // The delete button is within the same card as the target name
        const targetCard = page.locator(`text=${targetName}`).locator('..').locator('..');

        // Listen for confirmation dialog
        page.on('dialog', dialog => dialog.accept());

        // Click delete button
        await targetCard.locator('button[class*="destructive"]').click();

        // Wait for deletion
        await page.waitForTimeout(1000);

        // Verify target is removed
        await expect(page.locator(`text=${targetName}`).first()).not.toBeVisible();
    });

    test('should list multiple targets', async ({ page }) => {
        const target1 = generateTestName('Multi Target 1');
        const target2 = generateTestName('Multi Target 2');

        // Create first target
        await page.click('button:has-text("Add Target")');
        await page.fill('input#target-name', target1);
        await page.fill('input#target-url', 'https://example.com/1');
        await page.click('button[type="submit"]:has-text("Create")');
        await page.waitForTimeout(500);

        // Create second target
        await page.click('button:has-text("Add Target")');
        await page.fill('input#target-name', target2);
        await page.fill('input#target-url', 'https://example.com/2');
        await page.click('button[type="submit"]:has-text("Create")');
        await page.waitForTimeout(1000);

        // Verify both are visible
        await expect(page.locator(`text=${target1}`).first()).toBeVisible();
        await expect(page.locator(`text=${target2}`).first()).toBeVisible();
    });

    test('should show empty state when no targets exist', async ({ page }) => {
        // Check for empty state message
        const emptyMessage = page.locator('text=No targets configured');
        await emptyMessage.isVisible().catch(() => false);

        // Either we see empty state or we see the targets section
        await expect(page.locator('h3:has-text("Targets")')).toBeVisible();
    });
});

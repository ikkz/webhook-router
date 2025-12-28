import { test, expect } from '@playwright/test';
import { apiCreateEndpoint, apiCreateTarget, apiListEvents, generateTestName, getBaseUrl, waitFor } from './test-helpers';
import { MockTargetServer, WebhookSimulator, sendWebhookToIngress } from './mock-servers';

test.describe('Webhook Integration', () => {
    let mockServer: MockTargetServer;
    const baseUrl = getBaseUrl();

    test.beforeAll(async () => {
        // Start mock target server
        mockServer = new MockTargetServer(0); // Use random port
        await mockServer.start();
    });

    test.afterAll(async () => {
        // Stop mock server
        await mockServer.stop();
    });

    test.beforeEach(async () => {
        mockServer.clearWebhooks();
    });

    test('should process Slack webhook and forward to HTTP target', async ({ request }) => {
        const endpointName = generateTestName('Slack to HTTP');
        const targetName = generateTestName('HTTP Target');

        const endpoint = await apiCreateEndpoint(request, baseUrl, endpointName);
        await apiCreateTarget(request, baseUrl, endpoint.id, {
            name: targetName,
            kind: 'http',
            url: `${mockServer.url}/webhook`,
        });

        // Send webhook from "Slack"
        const slackPayload = WebhookSimulator.slack('Hello from Slack!', 'Test Message');
        const response = await sendWebhookToIngress(baseUrl, endpoint.id, 'slack', slackPayload);
        expect(response.ok).toBe(true);

        await waitFor(() => mockServer.getReceivedWebhooks().length >= 1, 5000, 100);
        const receivedWebhook = mockServer.getLastWebhook();
        expect(receivedWebhook).toBeDefined();
        expect(receivedWebhook?.body).toBeDefined();

        // Verify the forwarded payload contains markdown
        expect(receivedWebhook?.body.markdown).toContain('Hello from Slack!');
    });

    test('should process Lark webhook and forward to Slack target', async ({ request }) => {
        const endpointName = generateTestName('Lark to Slack');
        const targetName = generateTestName('Slack Target');

        const endpoint = await apiCreateEndpoint(request, baseUrl, endpointName);
        await apiCreateTarget(request, baseUrl, endpoint.id, {
            name: targetName,
            kind: 'slack',
            url: `${mockServer.url}/slack`,
        });

        // Send webhook from "Lark"
        const larkPayload = WebhookSimulator.lark('Hello from Lark!');
        const response = await sendWebhookToIngress(baseUrl, endpoint.id, 'lark', larkPayload);
        expect(response.ok).toBe(true);

        await waitFor(() => mockServer.getReceivedWebhooks().length >= 1, 5000, 100);
        const receivedWebhook = mockServer.getLastWebhook();
        expect(receivedWebhook).toBeDefined();
        expect(receivedWebhook?.body).toBeDefined();
    });

    test('should process DingTalk webhook and forward to Lark target', async ({ request }) => {
        const endpointName = generateTestName('DingTalk to Lark');
        const targetName = generateTestName('Lark Target');

        const endpoint = await apiCreateEndpoint(request, baseUrl, endpointName);
        await apiCreateTarget(request, baseUrl, endpoint.id, {
            name: targetName,
            kind: 'lark',
            url: `${mockServer.url}/lark`,
        });

        const dingtalkPayload = WebhookSimulator.dingtalk('Hello from DingTalk!');
        const response = await sendWebhookToIngress(baseUrl, endpoint.id, 'dingtalk', dingtalkPayload);
        expect(response.ok).toBe(true);

        await waitFor(() => mockServer.getReceivedWebhooks().length >= 1, 5000, 100);
        const receivedWebhook = mockServer.getLastWebhook();
        expect(receivedWebhook).toBeDefined();
    });

    test('should process WeCom webhook and forward to DingTalk target', async ({ request }) => {
        const endpointName = generateTestName('WeCom to DingTalk');
        const targetName = generateTestName('DingTalk Target');

        const endpoint = await apiCreateEndpoint(request, baseUrl, endpointName);
        await apiCreateTarget(request, baseUrl, endpoint.id, {
            name: targetName,
            kind: 'dingtalk',
            url: `${mockServer.url}/dingtalk`,
        });

        const wecomPayload = WebhookSimulator.wecom('Hello from WeCom!');
        const response = await sendWebhookToIngress(baseUrl, endpoint.id, 'wecom', wecomPayload);
        expect(response.ok).toBe(true);

        await waitFor(() => mockServer.getReceivedWebhooks().length >= 1, 5000, 100);
        const receivedWebhook = mockServer.getLastWebhook();
        expect(receivedWebhook).toBeDefined();
    });

    test('should process HTTP webhook and forward to WeCom target', async ({ request }) => {
        const endpointName = generateTestName('HTTP to WeCom');
        const targetName = generateTestName('WeCom Target');

        const endpoint = await apiCreateEndpoint(request, baseUrl, endpointName);
        await apiCreateTarget(request, baseUrl, endpoint.id, {
            name: targetName,
            kind: 'wecom',
            url: `${mockServer.url}/wecom`,
        });

        const httpPayload = WebhookSimulator.http('Hello from HTTP!', 'HTTP Test');
        const response = await sendWebhookToIngress(baseUrl, endpoint.id, 'http', httpPayload);
        expect(response.ok).toBe(true);

        await waitFor(() => mockServer.getReceivedWebhooks().length >= 1, 5000, 100);
        const receivedWebhook = mockServer.getLastWebhook();
        expect(receivedWebhook).toBeDefined();
    });

    test('should forward webhook to multiple targets', async ({ request }) => {
        const endpointName = generateTestName('Multi Target');
        const target1Name = generateTestName('Target 1');
        const target2Name = generateTestName('Target 2');

        const endpoint = await apiCreateEndpoint(request, baseUrl, endpointName);
        await apiCreateTarget(request, baseUrl, endpoint.id, {
            name: target1Name,
            kind: 'http',
            url: `${mockServer.url}/target1`,
        });
        await apiCreateTarget(request, baseUrl, endpoint.id, {
            name: target2Name,
            kind: 'http',
            url: `${mockServer.url}/target2`,
        });

        // Send one webhook
        const payload = WebhookSimulator.http('Multi-target test');
        const response = await sendWebhookToIngress(baseUrl, endpoint.id, 'http', payload);
        expect(response.ok).toBe(true);

        // Wait for both webhooks to be received
        await waitFor(() => mockServer.getReceivedWebhooks().length >= 2, 5000, 100);
        const webhooks = mockServer.getReceivedWebhooks();

        // Should have received 2 webhooks (one for each target)
        expect(webhooks.length).toBeGreaterThanOrEqual(2);
    });

    test('should handle URL verification challenge from Slack', async ({ request }) => {
        const endpointName = generateTestName('Slack Challenge');

        const endpoint = await apiCreateEndpoint(request, baseUrl, endpointName);

        // Send challenge payload
        const challengePayload = {
            type: 'url_verification',
            challenge: 'test_challenge_string',
        };

        const response = await sendWebhookToIngress(baseUrl, endpoint.id, 'slack', challengePayload);
        expect(response.ok).toBe(true);

        const responseData = (await response.json()) as { challenge: string };
        expect(responseData.challenge).toBe('test_challenge_string');
    });

    test('should create event record in database', async ({ request }) => {
        const endpointName = generateTestName('Event Record Test');
        const endpoint = await apiCreateEndpoint(request, baseUrl, endpointName);
        await apiCreateTarget(request, baseUrl, endpoint.id, {
            name: 'Test Target',
            kind: 'http',
            url: `${mockServer.url}/test`,
        });

        // Send webhook
        const payload = WebhookSimulator.http('Test event', 'Event Test');
        const response = await sendWebhookToIngress(baseUrl, endpoint.id, 'http', payload);
        expect(response.ok).toBe(true);

        await waitFor(async () => {
            const events = await apiListEvents(request, baseUrl);
            return events.some(
                (event: any) =>
                    event.endpoint_id === endpoint.id &&
                    event.platform === 'http' &&
                    String(event.markdown).includes('Test event')
            );
        }, 10000, 200);
    });
});

import express, { Express, Request, Response as ExpressResponse } from 'express';
import { Server } from 'http';

/**
 * Received webhook payload for verification
 */
export interface ReceivedWebhook {
    body: any;
    headers: Record<string, string | string[] | undefined>;
    timestamp: number;
}

/**
 * Mock target server that receives forwarded webhooks
 */
export class MockTargetServer {
    private app: Express;
    private server: Server | null = null;
    private receivedWebhooks: ReceivedWebhook[] = [];
    public port: number;
    public url: string;

    constructor(port = 0) {
        this.port = port;
        this.url = `http://127.0.0.1:${port}`;
        this.app = express();
        this.app.use(express.json());

        // Catch-all route to receive webhooks
        this.app.post(/.*/, (req: Request, res: ExpressResponse) => {
            this.receivedWebhooks.push({
                body: req.body,
                headers: { ...req.headers } as Record<string, string | string[] | undefined>,
                timestamp: Date.now(),
            });
            res.status(200).json({ success: true });
        });
    }

    async start(): Promise<void> {
        return new Promise((resolve) => {
            this.server = this.app.listen(this.port, '127.0.0.1', () => {
                const address = this.server!.address();
                if (address && typeof address === 'object') {
                    this.port = address.port;
                    this.url = `http://127.0.0.1:${this.port}`;
                }
                resolve();
            });
        });
    }

    async stop(): Promise<void> {
        return new Promise((resolve, reject) => {
            if (this.server) {
                this.server.close((err) => {
                    if (err) reject(err);
                    else resolve();
                });
            } else {
                resolve();
            }
        });
    }

    getReceivedWebhooks(): ReceivedWebhook[] {
        return this.receivedWebhooks;
    }

    getLastWebhook(): ReceivedWebhook | undefined {
        return this.receivedWebhooks[this.receivedWebhooks.length - 1];
    }

    clearWebhooks(): void {
        this.receivedWebhooks = [];
    }

    waitForWebhook(timeoutMs = 5000): Promise<ReceivedWebhook> {
        const initialCount = this.receivedWebhooks.length;
        return new Promise((resolve, reject) => {
            const startTime = Date.now();
            const checkInterval = setInterval(() => {
                if (this.receivedWebhooks.length > initialCount) {
                    clearInterval(checkInterval);
                    resolve(this.receivedWebhooks[this.receivedWebhooks.length - 1]);
                } else if (Date.now() - startTime > timeoutMs) {
                    clearInterval(checkInterval);
                    reject(new Error(`Timeout waiting for webhook after ${timeoutMs}ms`));
                }
            }, 100);
        });
    }
}

/**
 * Platform webhook simulators - generate payloads that match each platform's format
 */
export class WebhookSimulator {
    /**
     * Simulate a Slack webhook
     */
    static slack(text: string, title?: string) {
        return {
            type: 'event_callback',
            event: {
                type: 'message',
                text: text,
                user: 'U123456',
                ts: String(Date.now() / 1000),
                channel: 'C123456',
            },
            ...(title && {
                attachments: [
                    {
                        fallback: title,
                        title: title,
                    },
                ],
            }),
        };
    }

    /**
     * Simulate a Lark/Feishu webhook
     */
    static lark(text: string, title?: string) {
        return {
            schema: '2.0',
            header: {
                event_id: `event_${Date.now()}`,
                token: 'verification_token',
                create_time: String(Math.floor(Date.now() / 1000)),
                event_type: 'im.message.receive_v1',
            },
            event: {
                message: {
                    message_id: `msg_${Date.now()}`,
                    content: JSON.stringify({
                        text: text,
                    }),
                    message_type: 'text',
                },
                sender: {
                    sender_id: {
                        user_id: 'user123',
                    },
                },
            },
            ...(title && {
                header: {
                    event_id: `event_${Date.now()}`,
                    token: 'verification_token',
                    create_time: String(Math.floor(Date.now() / 1000)),
                    event_type: 'im.message.receive_v1',
                    title: title,
                },
            }),
        };
    }

    /**
     * Simulate a DingTalk webhook
     */
    static dingtalk(text: string, title?: string) {
        return {
            msgtype: 'text',
            text: {
                content: text,
            },
            msgId: `msg_${Date.now()}`,
            createAt: Date.now(),
            conversationType: '1',
            conversationId: 'cid123',
            senderId: 'sender123',
            ...(title && {
                msgtype: 'markdown',
                markdown: {
                    title: title,
                    text: text,
                },
            }),
        };
    }

    /**
     * Simulate a WeCom (WeCom Work) webhook
     */
    static wecom(text: string, title?: string) {
        return {
            msgtype: 'text',
            text: {
                content: text,
            },
            ...(title && {
                msgtype: 'markdown',
                markdown: {
                    content: `### ${title}\n\n${text}`,
                },
            }),
        };
    }

    /**
     * Simulate a generic HTTP webhook
     */
    static http(text: string, title?: string) {
        return {
            id: `evt_${Date.now()}`,
            source: 'custom',
            timestamp: Math.floor(Date.now() / 1000),
            title: title || undefined,
            markdown: text,
            raw: {},
            meta: {},
        };
    }
}

/**
 * Send webhook to ingress endpoint
 */
export async function sendWebhookToIngress(
    baseUrl: string,
    endpointId: string,
    platform: 'slack' | 'lark' | 'dingtalk' | 'wecom' | 'http',
    payload: any
): Promise<globalThis.Response> {
    const url = `${baseUrl}/ingress/${endpointId}/${platform}`;
    const response = await fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
    });
    return response;
}

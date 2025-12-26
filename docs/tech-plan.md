# Webhook Router 技术方案（草案）

## 目标
- 支持多种 webhook 入站（首期：Slack、飞书），统一转换为标准 Markdown。
- 支持一对多转发到不同类型的出站 webhook。
- 提供可视化控制台用于配置源、规则、目标、重试与观察。
- 具备可扩展的 adapter 插件机制，便于新增平台。

## 非目标（首期不做）
- 复杂工作流编排（多步骤、条件分支链路级别的编排引擎）。
- 规则引擎与条件路由。
- 深度富文本/附件的完全等价转换。
- 多租户计费、配额与企业级 SSO。

## 总体架构
- 后端：Rust 服务（`apps/webhook_router`）。
- 前端：React 控制台（`apps/console`）。
- 数据存储：SQLite（首期）。
- 队列/重试：首期不引入队列。

### 逻辑组件
- Ingress API：接收各平台 webhook，不做签名校验（通过端点 ID 隔离）。
- Adapter 层：将各平台 payload 解析为统一事件模型（UEM）。
- Renderer：将 UEM 渲染为标准 Markdown。
- Router：根据入站端点的目标配置，产生投递任务。
- Dispatcher：执行投递、重试、退避、告警。
- Console API：提供管理与观测接口。

## 统一事件模型（UEM）
- `id`：事件唯一标识（平台原始 id 或生成 UUID）。
- `source`：`slack` | `feishu` | `custom`。
- `timestamp`：事件发生时间。
- `title`：可选标题。
- `markdown`：标准 Markdown 正文。
- `raw`：原始 payload（JSON，便于回溯）。
- `meta`：可选扩展字段（channel/user/thread 等）。

## Markdown 规范
- 统一使用 CommonMark 兼容子集。
- 结构建议：标题 -> 正文 -> 附加信息（作者、频道、时间）。
- 复杂附件：首期降级为链接或文本摘要。

## 入站（Ingress）设计
- Slack
  - 支持 Events API 基础事件（message / app_mention）。
- 飞书（Feishu/Lark）
  - 支持消息事件（im.message.receive_v1）。
- 入站路径示例（用户在 console 创建端点，生成 `some-uuid`）
  - `POST /ingress/some-uuid/slack`
  - `POST /ingress/some-uuid/lark`

## 出站（Egress）设计
- 支持多类型目标：Slack Incoming Webhook、飞书自定义机器人、通用 HTTP Webhook。
- 支持一对多：单个事件可投递到多个目标。
- 投递策略（无队列）
  - 首期：同步投递（请求内）或进程内轻量异步任务。
  - 重试：指数退避（如 1m/5m/15m/1h），最大重试次数可配置。
  - 幂等：基于事件 id + 目标 id 去重。

## 路由与转发（无规则引擎）
- 入站端点绑定目标列表。
- 单个事件直接投递到该端点配置的一个或多个目标。

## 控制台（Console）
- 配置入口
- 源（Incoming Webhook）列表、端点 ID、回调地址、绑定的目标列表。
  - 目标（Outgoing Webhook）列表、类型、URL、密钥。
  - 路由规则管理。
- 观测
  - 最近事件列表（过滤、搜索）。
  - 投递状态、失败原因、重试次数。

## 后端 Rust 设计
- Web 框架：Axum（建议）
- 模块划分
  - `ingress/`：各平台接入与鉴权
  - `adapters/`：payload -> UEM
  - `render/`：UEM -> Markdown
  - `router/`：根据入站端点的目标配置，产生投递任务
  - `dispatch/`：投递与重试 worker
  - `storage/`：DB 访问与 outbox
  - `api/`：Console API

### Adapter 扩展性（Trait 方案）
- 定义统一 `WebhookAdapter` trait，包含入站解析与出站渲染两个方向：
  - `ingress_to_uem(payload) -> UemEvent`
  - `uem_to_egress(event) -> OutgoingPayload`
- 各平台（Slack/飞书/通用 HTTP）实现该 trait，便于新增类型或替换实现。

## 数据模型（草案）
- `sources`：入站源配置
- `targets`：出站目标配置
- `events`：已接收事件（含 UEM + raw）
- `deliveries`：投递记录与状态（pending/sent/failed）

## 安全与合规
- 入站不做签名校验，仅通过端点 ID 进行隔离；建议使用随机、不可猜测的 UUID。
- 出站敏感字段加密存储（如 webhook secret）。
- 访问控制：启动时通过命令行指定控制台用户名与密码，控制台使用 HTTP Basic Auth。
- 日志脱敏，避免泄漏密钥。

## 可观测性
- 结构化日志（tracing）。
- 关键指标：入站 QPS、投递成功率、延迟、失败分布。
- 调试：保留 raw payload（可配置保留时间）。

## 测试与验证
- 单元测试：adapter/renderer/router 规则。
- 集成测试：模拟平台入站与出站。
- E2E：Console 操作与投递流程。

## 迭代里程碑（建议）
1. 基础入站 + UEM + Markdown 渲染（Slack/飞书）。
2. 入站端点绑定目标 + 同步投递 + 失败重试。
3. Console 管理与观测。
4. 扩展更多平台与高级能力。

## 未决问题
- 启动参数中用户名/密码的安全存储与轮换方式。
- 数据保留策略与存储成本。

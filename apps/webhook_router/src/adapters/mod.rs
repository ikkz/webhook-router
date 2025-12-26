mod dingtalk;
mod http;
mod lark;
mod slack;
mod wecom;

use serde_json::Value;

pub use dingtalk::DingTalkAdapter;
pub use http::HttpAdapter;
pub use lark::LarkAdapter;
pub use slack::SlackAdapter;
pub use wecom::WecomAdapter;

use crate::models::{OutgoingPayload, UemEvent};

#[derive(Debug)]
pub struct AdapterError {
    pub message: String,
}

pub trait WebhookAdapter: Send + Sync {
    fn ingress_to_uem(&self, payload: &Value) -> Result<UemEvent, AdapterError>;
    fn uem_to_egress(&self, event: &UemEvent) -> Result<OutgoingPayload, AdapterError>;
}

pub fn ingress_adapter(platform: &str) -> Option<Box<dyn WebhookAdapter>> {
    match platform {
        "slack" => Some(Box::new(SlackAdapter)),
        "lark" | "feishu" => Some(Box::new(LarkAdapter)),
        "dingtalk" | "ding" => Some(Box::new(DingTalkAdapter)),
        "wecom" | "wechat_work" => Some(Box::new(WecomAdapter)),
        "http" | "custom" => Some(Box::new(HttpAdapter)),
        _ => None,
    }
}

pub fn egress_adapter(kind: &str) -> Option<Box<dyn WebhookAdapter>> {
    match kind {
        "slack" => Some(Box::new(SlackAdapter)),
        "lark" | "feishu" => Some(Box::new(LarkAdapter)),
        "dingtalk" | "ding" => Some(Box::new(DingTalkAdapter)),
        "wecom" | "wechat_work" => Some(Box::new(WecomAdapter)),
        "http" | "custom" => Some(Box::new(HttpAdapter)),
        _ => None,
    }
}

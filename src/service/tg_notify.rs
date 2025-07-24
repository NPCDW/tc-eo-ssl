use serde_json::json;

pub async fn send_msg(config: &crate::config::args_conf::Args, text: String) {
    if config.tg_bot_token.is_none() || config.tg_chat_id.is_none() {
        println!("tg_bot_token, tg_chat_id 未配置，不发送 tg 通知消息");
        return;
    }
    println!("发送 tg 通知消息");
    let url = format!("https://api.telegram.org/bot{}/sendMessage", config.tg_bot_token.as_ref().unwrap());
    let body = json!({"chat_id": config.tg_chat_id.unwrap(), "text": text, "parse_mode": "Markdown", "message_thread_id": config.tg_topic_id.unwrap_or(0)}).to_string();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("tc-eo-ssl"));
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .headers(headers)
        .body(body)
        .send()
        .await;
    match response {
        Ok(_) => println!("tg 消息发送成功"),
        Err(e) => println!("tg 消息发送失败: {}", e),
    }
}
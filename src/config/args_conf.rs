use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Deserialize, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// 密钥ID, 环境变量 TENCENTCLOUD_SECRET_ID
    #[arg(long)]
    pub secret_id: Option<String>,
    /// 密钥KEY, 环境变量 TENCENTCLOUD_SECRET_KEY
    #[arg(long)]
    pub secret_key: Option<String>,
    /// 公钥文件路径, 环境变量 TENCENTCLOUD_PUBLIC_KEY_FILE_PATH
    #[arg(long)]
    pub public_key_file_path: Option<String>,
    /// 私钥文件路径, 环境变量 TENCENTCLOUD_PRIVATE_KEY_FILE_PATH
    #[arg(long)]
    pub private_key_file_path: Option<String>,
    /// 域名列表，多个域名以英文逗号分割, 环境变量 TENCENTCLOUD_INSTANCE_ID_LIST
    #[arg(long)]
    pub instance_id_list: Option<Vec<String>>,
    /// 是否使用国际站, 环境变量 TENCENTCLOUD_INTL true国际站，false国内站，默认国内站
    #[arg(long)]
    pub intl: Option<bool>,
    
    /// TG bot token, 环境变量 TELEGRAM_BOT_TOKEN
    #[arg(long)]
    pub tg_bot_token: Option<String>,
    /// TG 聊天ID, 环境变量 TELEGRAM_CHAT_ID
    #[arg(long)]
    pub tg_chat_id: Option<i64>,
    /// TG 聊天主题ID，默认0, 环境变量 TELEGRAM_TOPIC_ID
    #[arg(long)]
    pub tg_topic_id: Option<i64>,
}

pub fn parse() -> anyhow::Result<Args> {
    let mut args = Args::parse();
    if args.secret_id.is_none() {
        match std::env::var("TENCENTCLOUD_SECRET_ID") {
            Ok(s) => args.secret_id = Some(s),
            Err(e) => println!("无法获取命令行参数 --secret-id 以及环境变量 TENCENTCLOUD_SECRET_ID: {}", e),
        }
    }
    if args.secret_key.is_none() {
        match std::env::var("TENCENTCLOUD_SECRET_KEY") {
            Ok(s) => args.secret_key = Some(s),
            Err(e) => println!("无法获取命令行参数 --secret-key 以及环境变量 TENCENTCLOUD_SECRET_KEY: {}", e),
        }
    }
    if args.public_key_file_path.is_none() {
        match std::env::var("TENCENTCLOUD_PUBLIC_KEY_FILE_PATH") {
            Ok(s) => args.public_key_file_path = Some(s),
            Err(e) => println!("无法获取命令行参数 --public-key-file-path 以及环境变量 TENCENTCLOUD_PUBLIC_KEY_FILE_PATH: {}", e),
        }
    }
    if args.private_key_file_path.is_none() {
        match std::env::var("TENCENTCLOUD_PRIVATE_KEY_FILE_PATH") {
            Ok(s) => args.private_key_file_path = Some(s),
            Err(e) => println!("无法获取命令行参数 --private-key-file-path 以及环境变量 TENCENTCLOUD_PRIVATE_KEY_FILE_PATH: {}", e),
        }
    }
    if args.instance_id_list.is_none() {
        match std::env::var("TENCENTCLOUD_INSTANCE_ID_LIST") {
            Ok(s) => args.instance_id_list = Some(s.split(",").map(|item| item.trim().to_string()).collect()),
            Err(e) => println!("无法获取命令行参数 --instance-id-list 以及环境变量 TENCENTCLOUD_INSTANCE_ID_LIST: {}", e),
        }
    }
    if args.intl.is_none() {
        match std::env::var("TENCENTCLOUD_INTL") {
            Ok(s) => args.intl = Some(&s.to_lowercase() == "true"),
            Err(_) => args.intl = Some(false),
        }
    }
    if args.tg_bot_token.is_none() {
        match std::env::var("TELEGRAM_BOT_TOKEN") {
            Ok(s) => args.tg_bot_token = Some(s),
            Err(_) => (),
        }
    }
    if args.tg_chat_id.is_none() {
        match std::env::var("TELEGRAM_CHAT_ID") {
            Ok(s) => args.tg_chat_id = Some(s.parse::<i64>()?),
            Err(_) => (),
        }
    }
    if args.tg_topic_id.is_none() {
        match std::env::var("TELEGRAM_TOPIC_ID") {
            Ok(s) => args.tg_topic_id = Some(s.parse::<i64>()?),
            Err(_) => (),
        }
    }
    anyhow::Ok(args)
}
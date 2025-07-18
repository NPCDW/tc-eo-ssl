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
}
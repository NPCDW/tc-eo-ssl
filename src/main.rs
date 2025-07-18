use clap::Parser;
use serde::{Deserialize, Serialize};
use service::tc_request::{TencentCloudRequest, TencentCloudResponse};

mod config;
mod service;

// 上传证书成功响应数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadCertificateData {
    #[serde(rename = "CertificateId")]
    pub certificate_id: String,
}

// 部署证书成功响应数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployCertificateData {
    #[serde(rename = "DeployRecordId")]
    pub deploy_record_id: i64,
    #[serde(rename = "DeployStatus")]
    pub deploy_status: i32,
}

// 上传证书
pub async fn upload_certificate(
    secret_id: String,
    secret_key: String,
    certificate_public_key: String,
    certificate_private_key: String,
    host: String,
) -> anyhow::Result<String> {
    let payload = serde_json::json!({
        "CertificatePublicKey": certificate_public_key,
        "CertificatePrivateKey": certificate_private_key,
        "CertificateUse": "teo"
    })
    .to_string();

    let request = TencentCloudRequest::new(
        secret_id,
        secret_key,
        "ssl".to_string(),
        host,
        "".to_string(),
        "UploadCertificate".to_string(),
        "2019-12-05".to_string(),
        payload,
        "".to_string(),
    );

    request.send().await
}

// 部署证书
pub async fn deploy_certificate(
    secret_id: String,
    secret_key: String,
    certificate_id: String,
    instance_id_list: Vec<String>,
    host: String,
) -> anyhow::Result<String> {
    let payload = serde_json::json!({
        "CertificateId": certificate_id,
        "InstanceIdList": instance_id_list,
        "ResourceType": "teo"
    })
    .to_string();

    let request = TencentCloudRequest::new(
        secret_id.to_string(),
        secret_key.to_string(),
        "ssl".to_string(),
        host,
        "".to_string(),
        "DeployCertificateInstance".to_string(),
        "2019-12-05".to_string(),
        payload,
        "".to_string(),
    );

    request.send().await
}

async fn execute() -> anyhow::Result<()> {
    let mut args = config::args_conf::Args::parse();
    if args.secret_id.is_none() {
        match std::env::var("TENCENTCLOUD_SECRET_ID") {
            Ok(s) => args.secret_id = Some(s),
            Err(e) => println!("无法获取命令行参数 --secret-id 以及环境变量 TENCENTCLOUD_SECRET_ID: {}", e),
        }
    }
    if args.secret_key.is_none() {
        match std::env::var("TENCENTCLOUD_SECRET_KEY") {
            Ok(s) => args.secret_id = Some(s),
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
            Ok(s) => args.secret_id = Some(s),
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
    
    let certificate_public_key = std::fs::read_to_string(args.public_key_file_path.as_ref().unwrap())?;
    let certificate_private_key = std::fs::read_to_string(args.private_key_file_path.as_ref().unwrap())?;
    let secret_id = args.secret_id.as_ref().unwrap();
    let secret_key = args.secret_key.as_ref().unwrap();
    let instance_id_list = args.instance_id_list.as_ref().unwrap();
    let intl = args.intl.unwrap_or(false);
    let host = if intl { "ssl.intl.tencentcloudapi.com".to_string() } else { "ssl.tencentcloudapi.com".to_string() };

    // 1. 上传证书
    println!("正在上传证书 {:?} {:?} ...", args.public_key_file_path, args.private_key_file_path);
    let upload_param = upload_certificate(
        secret_id.to_string(),
        secret_key.to_string(),
        certificate_public_key,
        certificate_private_key,
        host.clone(),
    ).await?;
    let upload_response = serde_json::from_str::<TencentCloudResponse<UploadCertificateData>>(&upload_param)?;
    if upload_response.response.error.is_some() {
        println!("证书上传失败");
        return Err(anyhow::anyhow!(upload_response.response.error.unwrap().to_string()));
    }
    let certificate_id = upload_response.response.data.unwrap().certificate_id;
    println!("证书上传成功，CertificateId: {}", certificate_id);

    // 2. 部署证书
    println!("正在部署证书 certificate_id 到 {:?}...", args.instance_id_list);
    let deploy_param = deploy_certificate(
        secret_id.to_string(),
        secret_key.to_string(),
        certificate_id.to_string(),
        instance_id_list.to_vec(),
        host,
    ).await?;
    let deploy_response = serde_json::from_str::<TencentCloudResponse<DeployCertificateData>>(&deploy_param)?;
    if deploy_response.response.error.is_some() {
        println!("证书部署失败");
        return Err(anyhow::anyhow!(deploy_response.response.error.unwrap().to_string()));
    }
    let deploy_record_id = deploy_response.response.data.unwrap().deploy_record_id;
    println!("证书部署成功，DeployRecordId: {}", deploy_record_id);

    anyhow::Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match execute().await {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("执行失败: {}", e);
            Err(e)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upload_certificate() {
        let secret_id = std::env::var("TENCENTCLOUD_SECRET_ID").unwrap();
        let secret_key = std::env::var("TENCENTCLOUD_SECRET_KEY").unwrap();
        let public_key = "test_public_key";
        let private_key = "test_private_key";
        let host = "ssl.intl.tencentcloudapi.com";

        let result = upload_certificate(secret_id, secret_key, public_key.to_string(), private_key.to_string(), host.to_string()).await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let response = serde_json::from_str::<TencentCloudResponse<UploadCertificateData>>(&result.unwrap());
        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.response.error.is_none());
    }

    #[tokio::test]
    async fn test_deploy_certificate() {
        let secret_id = std::env::var("TENCENTCLOUD_SECRET_ID").unwrap();
        let secret_key = std::env::var("TENCENTCLOUD_SECRET_KEY").unwrap();
        let certificate_id = "test_cert_id";
        let instance_id_list = vec!["test_instance_id".to_string()];
        let host = "ssl.intl.tencentcloudapi.com";

        let result = deploy_certificate(secret_id, secret_key, certificate_id.to_string(), instance_id_list, host.to_string()).await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let response = serde_json::from_str::<TencentCloudResponse<DeployCertificateData>>(&result.unwrap());
        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.response.error.is_none());
    }
}

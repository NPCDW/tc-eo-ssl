use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::header;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
struct TencentCloudRequest {
    secret_id: String,
    secret_key: String,
    service: String,
    host: String,
    region: String,
    action: String,
    version: String,
    payload: String,
    token: String,
}

impl TencentCloudRequest {
    fn new(
        secret_id: String,
        secret_key: String,
        service: String,
        host: String,
        region: String,
        action: String,
        version: String,
        payload: String,
        token: String,
    ) -> Self {
        Self {
            secret_id,
            secret_key,
            service,
            host,
            region,
            action,
            version,
            payload,
            token,
        }
    }

    async fn send(&self) -> Result<String, Box<dyn std::error::Error>> {
        let timestamp = Utc::now().timestamp();
        let date = Utc::now().format("%Y-%m-%d").to_string();

        // Step 1: Create canonical request
        let canonical_request = self.create_canonical_request()?;
        println!("Canonical request:\n{}", canonical_request);

        // Step 2: Create string to sign
        let string_to_sign = self.create_string_to_sign(&canonical_request, &date, timestamp)?;
        println!("String to sign:\n{}", string_to_sign);

        // Step 3: Calculate signature
        let signature = self.calculate_signature(&string_to_sign, &date)?;
        println!("Signature: {}", signature);

        // Step 4: Create authorization header
        let authorization = self.create_authorization(&signature, &date)?;
        println!("Authorization: {}", authorization);

        // Step 5: Send request
        let response = self.send_request(&authorization, timestamp).await?;
        Ok(response)
    }

    fn create_canonical_request(&self) -> Result<String, Box<dyn std::error::Error>> {
        let http_request_method = "POST";
        let canonical_uri = "/";
        let canonical_querystring = "";
        let canonical_headers = format!(
            "content-type:application/json; charset=utf-8\nhost:{}\nx-tc-action:{}\n",
            self.host,
            self.action.to_lowercase()
        );
        let signed_headers = "content-type;host;x-tc-action";

        let mut hasher = Sha256::new();
        hasher.update(self.payload.as_bytes());
        let hashed_request_payload = format!("{:x}", hasher.finalize());

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            http_request_method,
            canonical_uri,
            canonical_querystring,
            canonical_headers,
            signed_headers,
            hashed_request_payload
        );

        Ok(canonical_request)
    }

    fn create_string_to_sign(
        &self,
        canonical_request: &str,
        date: &str,
        timestamp: i64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let algorithm = "TC3-HMAC-SHA256";
        let credential_scope = format!("{}/{}/tc3_request", date, self.service);

        let mut hasher = Sha256::new();
        hasher.update(canonical_request.as_bytes());
        let hashed_canonical_request = format!("{:x}", hasher.finalize());

        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm, timestamp, credential_scope, hashed_canonical_request
        );

        Ok(string_to_sign)
    }

    fn calculate_signature(
        &self,
        string_to_sign: &str,
        date: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Create secret date
        let mut mac = HmacSha256::new_from_slice(format!("TC3{}", self.secret_key).as_bytes())?;
        mac.update(date.as_bytes());
        let secret_date = hex::encode(mac.finalize().into_bytes());

        // Create secret service
        let mut mac = HmacSha256::new_from_slice(&hex::decode(secret_date)?)?;
        mac.update(self.service.as_bytes());
        let secret_service = hex::encode(mac.finalize().into_bytes());

        // Create secret signing
        let mut mac = HmacSha256::new_from_slice(&hex::decode(secret_service)?)?;
        mac.update(b"tc3_request");
        let secret_signing = hex::encode(mac.finalize().into_bytes());

        // Calculate signature
        let mut mac = HmacSha256::new_from_slice(&hex::decode(secret_signing)?)?;
        mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        Ok(signature)
    }

    fn create_authorization(
        &self,
        signature: &str,
        date: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let algorithm = "TC3-HMAC-SHA256";
        let credential_scope = format!("{}/{}/tc3_request", date, self.service);
        let signed_headers = "content-type;host;x-tc-action";

        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, self.secret_id, credential_scope, signed_headers, signature
        );

        Ok(authorization)
    }

    async fn send_request(
        &self,
        authorization: &str,
        timestamp: i64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!("https://{}", self.host);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(authorization)?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json; charset=utf-8"),
        );
        headers.insert(
            header::HOST,
            header::HeaderValue::from_str(&self.host)?,
        );
        headers.insert(
            header::HeaderName::from_static("x-tc-action"),
            header::HeaderValue::from_str(&self.action)?,
        );
        headers.insert(
            header::HeaderName::from_static("x-tc-timestamp"),
            header::HeaderValue::from_str(&timestamp.to_string())?,
        );
        headers.insert(
            header::HeaderName::from_static("x-tc-version"),
            header::HeaderValue::from_str(&self.version)?,
        );
        headers.insert(
            header::HeaderName::from_static("x-tc-region"),
            header::HeaderValue::from_str(&self.region)?,
        );
        if !self.token.is_empty() {
            headers.insert(
                header::HeaderName::from_static("x-tc-token"),
                header::HeaderValue::from_str(&self.token)?,
            );
        }

        let response = client
            .post(&url)
            .headers(headers)
            .body(self.payload.clone())
            .send()
            .await?;

        let response_text = response.text().await?;
        Ok(response_text)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TencentCloudResponse<T> {
    #[serde(rename = "Response")]
    pub response: TencentCloudResponseDeatil<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TencentCloudResponseDeatil<T> {
    #[serde(rename = "Error")]
    pub error: Option<TencentCloudResponseError>,
    #[serde(rename = "RequestId")]
    pub request_id: String,
    #[serde(flatten)]
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TencentCloudErrorResponse {
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TencentCloudResponseError {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
}

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
) -> Result<String, Box<dyn std::error::Error>> {
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
        "ssl.intl.tencentcloudapi.com".to_string(),
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
) -> Result<String, Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "CertificateId": certificate_id,
        "InstanceIdList": instance_id_list,
        "ResourceType": "teo"
    })
    .to_string();

    let request = TencentCloudRequest::new(
        secret_id,
        secret_key,
        "ssl".to_string(),
        "ssl.intl.tencentcloudapi.com".to_string(),
        "".to_string(),
        "DeployCertificateInstance".to_string(),
        "2019-12-05".to_string(),
        payload,
        "".to_string(),
    );

    request.send().await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取凭证
    let secret_id = std::env::var("TENCENTCLOUD_SECRET_ID")?;
    let secret_key = std::env::var("TENCENTCLOUD_SECRET_KEY")?;

    // 1. 上传证书
    let public_key = "-----BEGIN CERTIFICATE-----\n...您的证书内容...\n-----END CERTIFICATE-----";
    let private_key = "-----BEGIN PRIVATE KEY-----\n...您的私钥内容...\n-----END PRIVATE KEY-----";

    println!("正在上传证书...");
    let upload_result = upload_certificate(
        secret_id.clone(),
        secret_key.clone(),
        public_key.to_string(),
        private_key.to_string(),
    )
    .await?;

    let upload_response: serde_json::Value = serde_json::from_str(&upload_result)?;
    let certificate_id = upload_response["Response"]["CertificateId"]
        .as_str()
        .ok_or("无法获取CertificateId")?;
    println!("证书上传成功，CertificateId: {}", certificate_id);

    // 2. 部署证书
    let domains = vec!["example.com".to_string(), "www.example.com".to_string()];
    
    println!("正在部署证书到 {:?}...", domains);
    let deploy_result = deploy_certificate(
        secret_id,
        secret_key,
        certificate_id.to_string(),
        domains,
    )
    .await?;

    let deploy_response: serde_json::Value = serde_json::from_str(&deploy_result)?;
    let deploy_record_id = deploy_response["Response"]["DeployRecordId"]
        .as_i64()
        .ok_or("无法获取DeployRecordId")?;
    println!("证书部署成功，DeployRecordId: {}", deploy_record_id);

    Ok(())
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

        let result = upload_certificate(secret_id, secret_key, public_key.to_string(), private_key.to_string()).await;
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

        let result = deploy_certificate(secret_id, secret_key, certificate_id.to_string(), instance_id_list).await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let response = serde_json::from_str::<TencentCloudResponse<DeployCertificateData>>(&result.unwrap());
        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.response.error.is_none());
    }
}

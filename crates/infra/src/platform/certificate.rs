use std::path::Path;

use crate::observability;

use super::PlatformError;

/// 经解析的 PEM CA 证书束，可用于构建隔离的 HTTP 客户端信任链。
#[derive(Debug)]
pub struct CaCertificateBundle {
    certificates: Vec<reqwest::Certificate>,
}

impl CaCertificateBundle {
    /// 返回证书数量。
    pub fn len(&self) -> usize {
        self.certificates.len()
    }

    /// 证书束是否为空。
    pub fn is_empty(&self) -> bool {
        self.certificates.is_empty()
    }

    /// 将全部 CA 证书添加到 HTTP 客户端构建器。
    pub fn apply_to(self, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        self.certificates
            .into_iter()
            .fold(builder, |builder, certificate| builder.add_root_certificate(certificate))
    }
}

/// 从 PEM 字节解析一个或多个 CA 证书。
pub fn parse_ca_certificate_bundle(pem: &[u8]) -> Result<CaCertificateBundle, PlatformError> {
    let certificates = reqwest::Certificate::from_pem_bundle(pem).map_err(|error| {
        let error = PlatformError::InvalidCertificate { message: error.to_string() };
        observability::platform_operation_failed("parse CA certificate bundle", &error);
        error
    })?;

    if certificates.is_empty() {
        let error = PlatformError::InvalidCertificate {
            message: "certificate bundle is empty".into(),
        };
        observability::platform_operation_failed("parse CA certificate bundle", &error);
        return Err(error);
    }

    Ok(CaCertificateBundle { certificates })
}

/// 从文件加载 PEM CA 证书束。
///
/// 此函数只解析本次客户端要追加的信任根，绝不写入操作系统的全局证书存储。
pub fn load_ca_certificate_bundle(
    path: impl AsRef<Path>,
) -> Result<CaCertificateBundle, PlatformError> {
    let path = path.as_ref();
    let pem = std::fs::read(path).map_err(|source| {
        let error = PlatformError::ReadCertificate { path: path.to_path_buf(), source };
        observability::platform_operation_failed("read CA certificate bundle", &error);
        error
    })?;
    parse_ca_certificate_bundle(&pem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_pem() {
        let error = parse_ca_certificate_bundle(b"not a certificate").unwrap_err();

        assert!(matches!(error, PlatformError::InvalidCertificate { .. }));
    }

    #[test]
    fn file_errors_include_the_path() {
        let path = std::env::temp_dir().join("sealantern-ca-certificate-does-not-exist.pem");
        let error = load_ca_certificate_bundle(&path).unwrap_err();

        assert!(matches!(error, PlatformError::ReadCertificate { .. }));
        assert!(error.to_string().contains(&path.display().to_string()));
    }
}

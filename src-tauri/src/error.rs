use serde::Serialize;

/// 应用统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),

    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("数据库迁移错误: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("JSON 解析错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP 压缩错误: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("书籍未找到: {0}")]
    BookNotFound(String),

    #[error("章节内容解密失败: {0}")]
    DecryptError(String),

    #[error("API 返回错误: code={code}, message={message}")]
    ApiError { code: i64, message: String },

    #[error("下载失败: {0}")]
    DownloadFailed(String),

    #[error("参数错误: {0}")]
    InvalidParam(String),

    #[error("其他错误: {0}")]
    Other(#[from] anyhow::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

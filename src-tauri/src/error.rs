use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Windows API 调用失败: {0}")]
    Windows(String),
    #[error("{0}")]
    Generic(String),
    #[error("进程不存在或无访问权限: {0}")]
    AccessDenied(String),
}

impl From<windows::core::Error> for AppError {
    fn from(e: windows::core::Error) -> Self {
        AppError::Windows(e.to_string())
    }
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.to_string().as_ref())
    }
}

pub type AppResult<T> = Result<T, AppError>;

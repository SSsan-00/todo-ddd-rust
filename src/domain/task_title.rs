use crate::TaskError;

/// タスクを表すValue Object
///
/// ただのStringではなく、
/// 「からでないタスクタイトル」という意味を持つ型として扱う
#[derive(Debug, PartialEq, Clone)]
pub struct TaskTitle(String);

impl TaskTitle {
    /// タスクタイトルを作成する
    ///
    /// から文字の場合は`TaskError::EmptyTitle`を返す
    pub fn new(value: &str) -> Result<Self, TaskError> {
        if value.is_empty() {
            return Err(TaskError::EmptyTitle);
        }

        Ok(Self(value.to_string()))
    }

    /// 内部の文字列を読み取り専用で取得する
    ///
    /// 所有権を壊さず、参照だけを返す
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// タスク保存の抽象
pub trait TaskRepository {
    fn save(&mut self, title: TaskTitle) -> Task;
    fn get_all(&self) -> Vec<Task>;
    fn find_by_id(&self, id: TaskId) -> Option<Task>;
    fn update(&mut self, task: Task);
}

use rusqlite::{Connection, params};

/// SQLiteを使ったTaskRepository実装
pub struct SqliteTaskRepository {
    conn: Connection,
}

impl SqliteTaskRepository {
    /// SQLiteファイルを開いてRepositoryを作成する
    pub fn new(db_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                completed INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }
}

impl TaskRepository for SqliteTaskRepository {
    fn save(&mut self, title: TaskTitle) -> Task {
        self.conn
            .execute(
                "INSERT INTO tasks (title, completed) VALUES (?1, ?2)",
                params![title.value(), 0],
            )
            .unwrap();

        let id = self.conn.last_insert_rowid() as u64;

        Task::new(TaskId::new(id), title)
    }

    fn get_all(&self) -> Vec<Task> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, completed FROM tasks ORDER BY id")
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let title: String = row.get(1)?;
                let completed: i64 = row.get(2)?;

                let mut task = Task::new(TaskId::new(id as u64), TaskTitle::new(&title).unwrap());

                if completed != 0 {
                    task.complete();
                }

                Ok(task)
            })
            .unwrap();

        rows.map(|row| row.unwrap()).collect()
    }

    fn find_by_id(&self, id: TaskId) -> Option<Task> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, completed FROM tasks WHERE id = ?1")
            .unwrap();

        stmt.query_row(params![id.value() as i64], |row| {
            let id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let completed: i64 = row.get(2)?;

            let mut task = Task::new(TaskId::new(id as u64), TaskTitle::new(&title).unwrap());

            if completed != 0 {
                task.complete();
            }

            Ok(task)
        })
        .ok()
    }

    fn update(&mut self, task: Task) {
        self.conn
            .execute(
                "UPDATE tasks SET title = ?1, completed = ?2 WHERE id = ?3",
                params![
                    task.title().value(),
                    if task.is_completed() { 1 } else { 0 },
                    task.id().value() as i64,
                ],
            )
            .unwrap();
    }
}

pub struct InMemoryTaskRepository {
    tasks: Vec<Task>,
    next_id: u64,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    fn generate_id(&mut self) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;
        TaskId::new(id)
    }
}

impl Default for InMemoryTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskRepository for InMemoryTaskRepository {
    fn save(&mut self, title: TaskTitle) -> Task {
        let id = self.generate_id();
        let task = Task::new(id, title);

        self.tasks.push(task.clone());

        task
    }

    fn get_all(&self) -> Vec<Task> {
        self.tasks.clone()
    }

    fn find_by_id(&self, id: TaskId) -> Option<Task> {
        self.tasks.iter().find(|task| task.id() == id).cloned()
    }

    fn update(&mut self, task: Task) {
        if let Some(index) = self
            .tasks
            .iter()
            .position(|current| current.id() == task.id())
        {
            self.tasks[index] = task;
        }
    }
}

pub struct TaskService<R: TaskRepository> {
    repo: R,
}

impl<R: TaskRepository> TaskService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn create(&mut self, title: &str) -> Result<Task, TaskError> {
        let title = TaskTitle::new(title)?;

        let task = self.repo.save(title);

        Ok(task)
    }

    pub fn get_all(&self) -> Vec<Task> {
        self.repo.get_all()
    }

    pub fn complete_task(&mut self, id: u64) -> Result<Task, TaskError> {
        let id = TaskId::new(id);

        let mut task = match self.repo.find_by_id(id) {
            Some(task) => task,
            None => return Err(TaskError::NotFound),
        };

        task.complete();

        self.repo.update(task.clone());

        Ok(task)
    }
}

/// タスク関連のエラー
#[derive(Debug, PartialEq)]
pub enum TaskError {
    /// タイトルがからだった場合のエラー
    EmptyTitle,
    NotFound,
}

/// タスクタイトルを表すValue Object
///
/// ただのStringではなく、
/// 「空でないタスクタイトル」という意味を持つ型として扱う
#[derive(Debug, PartialEq, Clone)]
pub struct TaskTitle(String);

impl TaskTitle {
    /// タスクタイトルを作成する
    ///
    /// 空文字の場合は`TaskError::EmptyTitle`を返す
    pub fn new(value: &str) -> Result<Self, TaskError> {
        if value.is_empty() {
            return Err(TaskError::EmptyTitle);
        }

        Ok(Self(value.to_string()))
    }

    /// 内部の文字列を読み取り専用で取得する
    ///
    /// 所有権を渡さず、参照だけを返す
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// タスクIDを表すValue
///
/// ただのu64ではなく、
/// 「タスクを識別するID」という意味を持たせる
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TaskId(u64);

impl TaskId {
    /// タスクIDを作成する
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// 内部の数値を取得する
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// タスクを表すEntuty
///
/// EntityはIDによって識別される
/// 同じタイトルでもIDが異なれば別のタスクとして扱う
#[derive(Debug, PartialEq, Clone)]
pub struct Task {
    id: TaskId,
    title: TaskTitle,
    completed: bool,
}

impl Task {
    /// タスクを新しく作成する
    ///
    /// 新規作成時点では未完了として扱う
    pub fn new(id: TaskId, title: TaskTitle) -> Self {
        Self {
            id,
            title,
            completed: false,
        }
    }

    /// タスクIDを取得する
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// タスクタイトルを取得する
    pub fn title(&self) -> &TaskTitle {
        &self.title
    }

    /// 完了済みかどうかを返す
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// タスクを完了状態にする
    pub fn complete(&mut self) {
        self.completed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_title_can_be_created_when_value_is_not_empty() {
        // Arrange
        let value = "Learn Rust";

        // Act
        let title = TaskTitle::new(value);

        // Assert
        assert!(title.is_ok());

        let title = title.unwrap();
        assert_eq!(title.value(), "Learn Rust");
    }

    #[test]
    fn task_title_returns_error_when_value_is_empty() {
        // Arrange
        let value = "";

        // Act
        let title = TaskTitle::new(value);

        // Assert
        assert_eq!(title, Err(TaskError::EmptyTitle));
    }

    #[test]
    fn task_can_be_created_with_id_and_title() {
        // Arrange
        let id = TaskId::new(1);
        let title = TaskTitle::new("Learn Rust").unwrap();

        // Act
        let task = Task::new(id, title);

        // Assert
        assert_eq!(task.id().value(), 1);
        assert_eq!(task.title().value(), "Learn Rust");
        assert!(!task.is_completed());
    }

    #[test]
    fn task_can_be_completed() {
        // Arrange
        let id = TaskId::new(1);
        let title = TaskTitle::new("Learn Rust").unwrap();
        let mut task = Task::new(id, title);

        // Act
        task.complete();

        // Assert
        assert!(task.is_completed());
    }

    #[test]
    fn task_can_be_saved_and_retrieved() {
        // Arrange
        let repo = InMemoryTaskRepository::new();
        let mut service = TaskService::new(repo);

        // Act
        service.create("Learn Rust").unwrap();

        // Assert
        let tasks = service.get_all();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title().value(), "Learn Rust");
    }

    #[test]
    fn create_returns_created_task() {
        // Arrange
        let repo = InMemoryTaskRepository::new();
        let mut service = TaskService::new(repo);

        // Act
        let task = service.create("Learn Rust").unwrap();

        // Assert
        assert_eq!(task.id().value(), 1);
        assert_eq!(task.title().value(), "Learn Rust");
        assert!(!task.is_completed());
    }

    #[test]
    fn complete_task_marks_task_as_completed() {
        // Arrange
        let repo = InMemoryTaskRepository::new();
        let mut service = TaskService::new(repo);

        let task = service.create("Learn Rust").unwrap();

        // Act
        let completed_task = service.complete_task(task.id().value()).unwrap();

        // Assert
        assert!(completed_task.is_completed());

        let tasks = service.get_all();
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].is_completed());
    }

    #[test]
    fn complete_task_returns_error_when_task_does_not_exist() {
        // Arrange
        let repo = InMemoryTaskRepository::new();
        let mut service = TaskService::new(repo);

        // Act
        let result = service.complete_task(999);

        // Assert
        assert_eq!(result, Err(TaskError::NotFound));
    }
}

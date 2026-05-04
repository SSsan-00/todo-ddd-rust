use todo_ddd_rust::{SqliteTaskRepository, TaskService};

fn main() {
    // コマンドライン引数を取得する
    let args: Vec<String> = std::env::args().collect();

    // 保存先Repositoryを作成する
    // 今はメモリ上なので、アプリを終了するとデータは消える
    let repo = SqliteTaskRepository::new("todo.db").expect("failed to open database");

    // ServiceにRepositoryを注入する
    let mut service = TaskService::new(repo);

    // コマンドは指定されていない場合
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("    todo-ddd-rust add <title>");
        eprintln!("    todo-ddd-rust list");
        return;
    }

    match args[1].as_str() {
        "add" => {
            // タイトルが指定されていない場合
            if args.len() < 3 {
                eprintln!("title is required");
                return;
            }

            let title = &args[2];

            match service.create(title) {
                Ok(task) => {
                    println!("created: [{}] {}", task.id().value(), task.title().value());
                }
                Err(error) => {
                    eprintln!("error: {:?}", error)
                }
            }
        }

        "list" => match service.get_all() {
            Ok(tasks) => {
                if tasks.is_empty() {
                    println!("no tasks");
                    return;
                }

                for task in tasks {
                    let status = if task.is_completed() { "x" } else { " " };

                    println!(
                        "[{}] {}: {}",
                        status,
                        task.id().value(),
                        task.title().value()
                    );
                }
            }
            Err(error) => {
                eprintln!("error: {:?}", error);
            }
        },

        "done" => {
            if args.len() < 3 {
                eprintln!("id is required");
                return;
            }

            let id = match args[2].parse::<u64>() {
                Ok(id) => id,
                Err(_) => {
                    eprintln!("id must be a number");
                    return;
                }
            };

            match service.complete_task(id) {
                Ok(task) => {
                    println!(
                        "completed: [{}] {}",
                        task.id().value(),
                        task.title().value()
                    );
                }
                Err(error) => {
                    eprintln!("error: {:?}", error);
                }
            }
        }

        _ => {
            eprintln!("unknown command");
        }
    }
}

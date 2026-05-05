use clap::{Parser, Subcommand};
use todo_ddd_rust::{SqliteTaskRepository, TaskService};

#[derive(Parser)]
#[command(name = "todo")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { title: String },
    List,
    Done { id: u64 },
}

fn main() {
    // コマンドライン引数を取得する
    let cli = Cli::parse();

    // 保存先Repositoryを作成する
    // 今はメモリ上なので、アプリを終了するとデータは消える
    let repo = SqliteTaskRepository::new("todo.db").expect("failed to open database");

    // ServiceにRepositoryを注入する
    let mut service = TaskService::new(repo);

    match cli.command {
        Commands::Add { title } => {
            // タイトルが指定されていない場合

            match service.create(&title) {
                Ok(task) => {
                    println!("created: [{}] {}", task.id().value(), task.title().value());
                }
                Err(error) => {
                    eprintln!("error: {:?}", error)
                }
            }
        }

        Commands::List => match service.get_all() {
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

        Commands::Done { id } => match service.complete_task(id) {
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
        },
    }
}

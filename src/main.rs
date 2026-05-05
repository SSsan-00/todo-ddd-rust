use clap::{Parser, Subcommand};
use todo::{SqliteTaskRepository, TaskService};

#[derive(Parser)]
#[command(
    name = "todo",
    version,
    about = "Simple TODO CLI built with Rust + DDD + Di",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// Task title
        title: String,
    },
    /// Show tasks
    List {
        /// Show only completed tasks
        #[arg(long)]
        completed: bool,

        /// Show only active tasks
        #[arg(long)]
        active: bool,
    },
    Done {
        /// Task ID
        id: u64,
    },
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

        Commands::List { completed, active } => match service.get_all() {
            Ok(tasks) => {
                let filtered_tasks: Vec<_> = tasks
                    .into_iter()
                    .filter(|task| {
                        if completed {
                            task.is_completed()
                        } else if active {
                            !task.is_completed()
                        } else {
                            true
                        }
                    })
                    .collect();

                if filtered_tasks.is_empty() {
                    println!("no tasks");
                    return;
                }

                for task in filtered_tasks {
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

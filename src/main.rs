use rusqlite::{params, Connection, Result};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;


#[derive(Debug)]
struct UserData {
    task: String,
    deadline: String,
    reminder: String,
}

fn save_task_to_db(user_data: UserData) -> Result<()> {
    let conn = Connection::open("tasks.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            task TEXT NOT NULL,
            deadline TEXT NOT NULL,
            reminder TEXT NOT NULL
        )",
        params![],
    )?;
    conn.execute(
        "INSERT INTO tasks (task, deadline, reminder) VALUES (?1, ?2, ?3)",
        params![user_data.task, user_data.deadline, user_data.reminder],
    )?;
    Ok(())
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    ListOptions,
    ChoseOption {
        option: String,
    },
    AllTasks,
    TodaysTasks,
    AddTask,
    ReceiveTask {
        task: String,
    },
    ReceiveTaskDeadline {
        task: String,
        deadline: String,
    },
    ReceiveTaskReminder {
        task: String,
        deadline: String,
        reminder: String,
    },
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::ListOptions].endpoint(list_options))
            .branch(dptree::case![State::ChoseOption { option }].endpoint(chose_option))
            .branch(dptree::case![State::AllTasks].endpoint(all_tasks))
            .branch(dptree::case![State::TodaysTasks].endpoint(todays_tasks))
            .branch(dptree::case![State::AddTask].endpoint(add_task))
            .branch(dptree::case![State::ReceiveTask { task }].endpoint(receive_task))
            .branch(dptree::case![State::ReceiveTaskDeadline { task, deadline }].endpoint(receive_task_deadline))
            .branch(
                dptree::case![State::ReceiveTaskReminder { task, deadline, reminder }].endpoint(receive_task_reminder),
            ),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn list_options(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
) -> HandlerResult {
    bot.send_message(message.chat_id(), "Choose an option").await?;
    bot.send_message(message.chat_id(), "Display all tasks -> /alltasks").await?;
    bot.send_message(message.chat_id(), "Display today's tasks -> /todaystasks").await?;
    bot.send_message(message.chat_id(), "Add a task -> /addtask").await?;
    match message.text() {
        Some(text) => {
            dialogue.update(State::ChoseOption { option: text.into() }).await?;
        }
        None => {
            bot.send_message(message.chat_id(), "Please send a text message").await?;
        }
    }
    Ok(())
}

async fn chose_option(
    bot: Bot,
    dialogue: MyDialogue,
    option: String,
    message: Message,
) -> HandlerResult {
    match message.text() {
        Some(text) => {
            if text == "/alltasks" {
                dialogue.update(State::AllTasks).await?;
            } else if text == "/todaystasks" {
                dialogue.update(State::TodaysTasks).await?;
            } else if text == "/addtask" {
                dialogue.update(State::AddTask).await?;
            } else {
                bot.send_message(message.chat_id(), "Please send a valid option").await?;
            }
        }
        None => {
            bot.send_message(message.chat_id(), "Please send a text message").await?;
        }
    }
    Ok(())
}

async fn all_tasks(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message
) -> HandlerResult {
    bot.send_message(message.chat_id(), "All tasks").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn todays_tasks(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message
) -> HandlerResult {
    bot.send_message(message.chat_id(), "Today's tasks").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn add_task(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message
) -> HandlerResult {
    bot.send_message(message.chat_id(), "Add a task").await?;
    bot.send_message(message.chat_id(), "Enter task name").await?;
    match message.text() {
        Some(text) => {
            dialogue.update(State::ReceiveTask { task: text.to_string() }).await?;
        }
        None => {
            bot.send_message(message.chat_id(), "Please send a text message").await?;
        }
    }
    Ok(())
}

async fn receive_task(
    bot: Bot,
    dialogue: MyDialogue,
    task: String,
    message: Message,
) -> HandlerResult {
    match message.text() {
        Some(text) => {
            bot.send_message(message.chat_id(), "Task received").await?;
            dialogue.update(State::ReceiveTaskDeadline {
                task,
                deadline: text.to_string(),
            }).await?;
        }
        None => {
            bot.send_message(message.chat_id(), "Please send a text message").await?;
        }
    }
    Ok(())
}

async fn receive_task_deadline(
    bot: Bot,
    dialogue: MyDialogue,
    task: String,
    deadline: String,
    message: Message,
) -> HandlerResult {
    match message.text() {
        Some(text) => {
            bot.send_message(message.chat_id(), "Task received").await?;
            dialogue.update(State::ReceiveTaskReminder {
                task,
                deadline,
                reminder: text.to_string(),
            }).await?;
        }
        None => {
            bot.send_message(message.chat_id(), "Please send a text message").await?;
        }
    }
    Ok(())
}

async fn receive_task_reminder(
    bot: Bot,
    dialogue: MyDialogue,
    task: String,
    deadline: String,
    reminder: String,
    message: Message,
) -> HandlerResult {
    match message.text() {
        Some(text) => {
            bot.send_message(message.chat_id(), "Task received").await?;
            let user_data = UserData {
                task,
                deadline,
                reminder,
            };
            if let Err(e) = save_task_to_db(user_data) {
                bot.send_message(message.chat_id(), "Error saving task to database").await?;
                log::error!("Error saving task to database: {}", e);
            } else {
                bot.send_message(message.chat_id(), "Task saved to database").await?;
            }
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(message.chat_id(), "Please send a text message").await?;
        }
    }
    Ok(())
}

use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    AllTasks,
    TodaysTasks,
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
            .branch(dptree::case![State::AllTasks].endpoint(all_tasks))
            .branch(dptree::case![State::TodaysTasks].endpoint(todays_tasks))
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
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(message.chat_id(), "Please send a text message").await?;
        }
    }
    Ok(())
}

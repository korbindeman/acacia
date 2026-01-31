use acacia::prelude::*;

#[model("tasks")]
pub struct Task {
    #[key]
    pub id: i32,
    pub title: String,
    pub done: bool,
}

#[form(Task)]
struct NewTask {
    title: String,
}

#[component]
fn TaskItem(task: &Task) -> Fragment {
    let task_id = task.id;
    html! {
        <li id={format!("task-{}", task_id)} class="flex gap-2 items-center p-2 border-b border-gray-200">
            <input
                type="checkbox"
                checked={task.done}
                {submits(TOGGLE_TASK(task_id)).target(Target::Parent)}
            />
            <span class={tw!("flex-1", "line-through opacity-50" => task.done)}>
                {&task.title}
            </span>
            <button {removes(DELETE_TASK(task_id))} class="ml-auto cursor-pointer text-gray-500 hover:text-red-500">
                {"×"}
            </button>
        </li>
    }
}

#[page("/")]
async fn home(db: Db) -> Result<Page> {
    let tasks = db.all::<Task>().await?;

    Ok(html! {
        <main class="max-w-md mx-auto p-5 font-sans">
            <h1 class="mb-5 text-2xl font-bold">{"Tasks"}</h1>
            <ul id="tasks" class="list-none p-0 mb-5 border border-gray-300 rounded">
                {for task in &tasks { TaskItem(task) }}
            </ul>
            <form
                {submits(CREATE_TASK).into("#tasks").append()}
                hx-on::after-request="this.reset()"
                class="flex gap-2"
            >
                <input
                    name="title"
                    placeholder="New task..."
                    required
                    class="flex-1 p-2 border border-gray-300 rounded"
                />
                <button
                    type="submit"
                    class="px-4 py-2 bg-blue-500 text-white border-none rounded cursor-pointer hover:bg-blue-600"
                >
                    {"Add"}
                </button>
            </form>
        </main>
    }.into_page())
}

#[action("/tasks", method = "POST")]
async fn create_task(db: Db, form: Valid<NewTask>) -> Result<Fragment> {
    let task = db.insert::<Task, _>(form.into_inner()).await?;
    Ok(TaskItem(&task))
}

#[action("/tasks/{id}/toggle", method = "POST")]
async fn toggle_task(Path(id): Path<i32>, db: Db) -> Result<Fragment> {
    let task = db.toggle::<Task, _>(id, |t| &mut t.done).await?;
    Ok(TaskItem(&task))
}

#[action("/tasks/{id}", method = "DELETE")]
async fn delete_task(Path(id): Path<i32>, db: Db) -> Result<Response> {
    db.delete::<Task>(id).await?;
    Ok(Response::empty())
}

#[tokio::main]
async fn main() {
    Acacia::new()
        .database("sqlite://tasks.db?mode=rwc")
        .migrate(MigratePolicy::Auto)
        .serve("0.0.0.0:3000")
        .await;
}

use acacia::prelude::*;

#[derive(Model)]
#[table("tasks")]
pub struct Task {
    #[key]
    pub id: i32,
    pub title: String,
    pub done: bool,
}

#[derive(Form)]
#[for_model(Task)]
pub struct NewTask {
    pub title: String,
}

#[component]
fn TaskItem(task: &Task) -> Fragment {
    let task_id = task.id;
    html! {
        <li id={format!("task-{}", task_id)} style="display: flex; gap: 8px; align-items: center; padding: 8px; border-bottom: 1px solid #eee;">
            <input
                type="checkbox"
                checked={task.done}
                {submits(TOGGLE_TASK(task_id)).target(Target::Parent)}
            />
            <span style={if task.done { "text-decoration: line-through; opacity: 0.5;" } else { "" }}>
                {&task.title}
            </span>
            <button {removes(DELETE_TASK(task_id))} style="margin-left: auto; cursor: pointer;">
                {"x"}
            </button>
        </li>
    }
}

#[page("/")]
async fn home(db: Db) -> Result<Page> {
    let tasks = db.all::<Task>().await?;

    Ok(html! {
        <main style="max-width: 500px; margin: 0 auto; padding: 20px; font-family: system-ui, sans-serif;">
            <h1 style="margin-bottom: 20px;">{"Tasks"}</h1>
            <ul id="tasks" style="list-style: none; padding: 0; margin: 0 0 20px 0; border: 1px solid #ddd; border-radius: 4px;">
                {for task in &tasks { TaskItem(task) }}
            </ul>
            <form
                {submits(CREATE_TASK).into("#tasks").append()}
                hx-on::after-request="this.reset()"
                style="display: flex; gap: 8px;"
            >
                <input
                    name="title"
                    placeholder="New task..."
                    required
                    style="flex: 1; padding: 8px; border: 1px solid #ddd; border-radius: 4px;"
                />
                <button
                    type="submit"
                    style="padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;"
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
    let task = db.update::<Task, _>(id, |t| t.done = !t.done).await?;
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

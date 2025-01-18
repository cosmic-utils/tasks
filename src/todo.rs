use crate::{
    app::markdown::Markdown,
    core::{
        models::{List, Task},
        service::TaskService,
        TasksError,
    },
    Error,
};

pub async fn update_list(list: List, service: TaskService) -> Result<(), Error> {
    if let Some(mut service) = service.get_service() {
        service.update_list(list).await?;
    }
    Ok(())
}

pub async fn delete_list(id: String, service: TaskService) -> Result<(), Error> {
    if let Some(mut service) = service.get_service() {
        service.delete_list(id).await?;
    }
    Ok(())
}

pub async fn create_list(list: List, service: TaskService) -> Result<List, Error> {
    if let Some(mut service) = service.get_service() {
        let list = service.create_list(list).await?;
        return Ok(list);
    }
    Err(Error::Tasks(TasksError::ServiceUnavailable))
}

pub async fn create_task(task: Task, service: TaskService) -> Result<(), Error> {
    if let Some(mut service) = service.get_service() {
        service.create_task(task).await?;
    }
    Ok(())
}

pub async fn fetch_lists(service: TaskService) -> Result<Vec<List>, Error> {
    if let Some(mut service) = service.get_service() {
        let lists = service.get_lists().await?;
        return Ok(lists);
    }
    Ok(vec![])
}

pub async fn fetch_tasks(list_id: String, service: TaskService) -> Result<Vec<Task>, Error> {
    if let Some(mut service) = service.get_service() {
        let tasks = service.get_tasks_from_list(list_id).await?;
        return Ok(tasks);
    }
    Ok(vec![])
}

pub async fn update_task(task: Task, service: TaskService) -> Result<(), Error> {
    if let Some(mut service) = service.get_service() {
        service.update_task(task).await?;
    }
    Ok(())
}

pub async fn delete_task(
    list_id: String,
    task_id: String,
    service: TaskService,
) -> Result<(), Error> {
    if let Some(mut service) = service.get_service() {
        service.delete_task(list_id, task_id).await?;
    }
    Ok(())
}

pub fn export_list(list: &List, tasks: &[Task]) -> String {
    let markdown = list.markdown();
    let tasks_markdown: String = tasks.iter().map(Markdown::markdown).collect();
    format!("{markdown}\n{tasks_markdown}")
}

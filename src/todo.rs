use crate::app::markdown::Markdown;
use tasks_core::models::list::List;
use tasks_core::models::task::Task;
use tasks_core::service::TaskService;
use std::error::Error;

pub async fn update_list(list: List, service: TaskService) -> Result<(), Box<dyn Error>> {
    if let Some(mut service) = service.get_service() {
        service.update_list(list).await?;
    }
    Ok(())
}

pub async fn delete_list(id: String, service: TaskService) -> Result<(), Box<dyn Error>> {
    if let Some(mut service) = service.get_service() {
        service.delete_list(id).await?;
    }
    Ok(())
}

pub async fn create_list(list: List, service: TaskService) -> Result<List, Box<dyn Error>> {
    if let Some(mut service) = service.get_service() {
        let list = service.create_list(list).await?;
        return Ok(list);
    }
    Err("No service found".into())
}

pub async fn create_task(task: Task, service: TaskService) -> Result<(), Box<dyn Error>> {
    if let Some(mut service) = service.get_service() {
        service.create_task(task).await?;
    }
    Ok(())
}

pub async fn fetch_lists(service: TaskService) -> Result<Vec<List>, Box<dyn Error>> {
    if let Some(mut service) = service.get_service() {
        let lists = service.get_lists().await?;
        return Ok(lists);
    }
    Ok(vec![])
}

pub async fn fetch_tasks(
    list_id: String,
    service: TaskService,
) -> Result<Vec<Task>, Box<dyn Error>> {
    if let Some(mut service) = service.get_service() {
        let tasks = service.get_tasks_from_list(list_id).await?;
        return Ok(tasks);
    }
    Ok(vec![])
}

pub async fn update_task(task: Task, service: TaskService) -> Result<(), Box<dyn Error>> {
    if let Some(mut service) = service.get_service() {
        service.update_task(task).await?;
    }
    Ok(())
}

pub async fn delete_task(
    list_id: String,
    task_id: String,
    service: TaskService,
) -> Result<(), Box<dyn Error>> {
    if let Some(mut service) = service.get_service() {
        service.delete_task(list_id, task_id).await?;
    }
    Ok(())
}

pub fn export_list(list: List, tasks: Vec<Task>) -> String {
    let markdown = list.markdown();
    let tasks_markdown: String = tasks.iter().map(|task| task.markdown()).collect();
    format!("{}\n{}", markdown, tasks_markdown)
}

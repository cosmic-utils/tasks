use done_core::models::list::List;
use done_core::models::task::Task;
use done_core::service::Service;
use crate::app::markdown::Markdown;
use std::error::Error;

pub async fn update_list(list: List) -> Result<(), Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.update_list(list).await?)
}

pub async fn delete_list(id: String) -> Result<(), Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.delete_list(id).await?)
}

pub async fn create_list(list: List) -> Result<List, Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.create_list(list).await?)
}

pub async fn create_task(task: Task) -> Result<(), Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.create_task(task).await?)
}

pub async fn fetch_lists() -> Result<Vec<List>, Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.read_lists().await.unwrap_or(vec![]))
}

pub async fn fetch_tasks(list_id: String) -> Result<Vec<Task>, Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service
        .read_tasks_from_list(list_id)
        .await
        .unwrap_or(vec![]))
}

pub async fn update_task(task: Task) -> Result<Task, Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.update_task(task).await?)
}

pub async fn delete_task(list_id: String, task_id: String) -> Result<(), Box<dyn Error>> {
    let mut service = Service::Computer.get_service();
    Ok(service.delete_task(list_id, task_id).await?)
}

pub fn export_list(list: List, tasks: Vec<Task>) -> String {
    let markdown = list.markdown();
    let tasks_markdown: String = tasks.iter().map(|task| task.markdown()).collect();
    format!("{}\n{}", markdown, tasks_markdown)
}
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::storage::models::Task;

#[derive(Debug, Clone)]
pub struct TaskCache {
    pub inner: Arc<Mutex<TaskCacheInner>>,
}

#[derive(Debug)]
pub struct TaskCacheInner {
    pub tasks_by_list: HashMap<String, Vec<Task>>, // list_id -> tasks
    pub all_tasks: Vec<Task>,                      // Flattened for virtual lists
}

impl TaskCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TaskCacheInner {
                tasks_by_list: HashMap::new(),
                all_tasks: Vec::new(),
            })),
        }
    }
    
    pub fn update_task(&mut self, task: Task) {
        let mut inner = self.inner.lock().unwrap();
        
        // Update in list-specific cache
        if let Some(list_id) = &task.list_id {
            if let Some(tasks) = inner.tasks_by_list.get_mut(list_id) {
                if let Some(index) = tasks.iter().position(|t| t.id == task.id) {
                    tasks[index] = task.clone();
                } else {
                    tasks.push(task.clone());
                }
            }
        }
        
        // Update in all_tasks cache
        if let Some(index) = inner.all_tasks.iter().position(|t| t.id == task.id) {
            inner.all_tasks[index] = task;
        } else {
            inner.all_tasks.push(task);
        }
    }
    
    pub fn remove_task(&mut self, task_id: &str, list_id: &str) {
        let mut inner = self.inner.lock().unwrap();
        
        // Remove from list-specific cache
        if let Some(tasks) = inner.tasks_by_list.get_mut(list_id) {
            tasks.retain(|t| t.id != task_id);
        }
        
        // Remove from all_tasks cache
        inner.all_tasks.retain(|t| t.id != task_id);
    }
    
    pub fn update_list_tasks(&mut self, list_id: &str, tasks: Vec<Task>) {
        let mut inner = self.inner.lock().unwrap();
        
        // Update list-specific cache
        inner.tasks_by_list.insert(list_id.to_string(), tasks.clone());
        
        // Rebuild all_tasks from individual lists
        inner.all_tasks = inner.tasks_by_list
            .values()
            .flatten()
            .cloned()
            .collect();
        
        tracing::info!("🔄 Cache updated for list {}: {} tasks, total cache now has {} tasks", 
                      list_id, tasks.len(), inner.all_tasks.len());
    }
    
    pub fn get_all_tasks(&self) -> Vec<Task> {
        let inner = self.inner.lock().unwrap();
        inner.all_tasks.clone()
    }
    
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.all_tasks.is_empty()
    }
    
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        tracing::warn!("🗑️ Cache being cleared! Had {} tasks", inner.all_tasks.len());
        inner.tasks_by_list.clear();
        inner.all_tasks.clear();
    }
}

use std::fs;
use std::io::Write;
use cosmic::Application;
use ron::de::from_str;
use ron::ser::to_string;
use crate::app::Tasks;
use crate::core::models::{List, Task};
use crate::Error;

pub fn migrate_data_dir(prev_app_ids: &[&str]) {
    for prev_app_id in prev_app_ids.iter() {
        let prev = dirs::data_local_dir().unwrap().join(prev_app_id);
        let new = dirs::data_local_dir().unwrap().join(Tasks::APP_ID);
        if prev.exists() {
            match fs::rename(prev, new) {
                Ok(()) => tracing::info!("migrated data to new directory"),
                Err(err) => tracing::error!("error migrating data: {:?}", err),
            }
        }
    }
}

pub fn migrate_data() -> Result<(), Error> {
    let data_path = dirs::data_local_dir()
        .ok_or(Error::Io(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "XDG data directory not found",
            ),
        ))?.join(Tasks::APP_ID);

    let lists_dir = data_path.join("lists");
    if lists_dir.exists() {
        for entry in fs::read_dir(&lists_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "ron").unwrap_or(false) {
                let content = fs::read_to_string(&path)?;
                let mut list: List = from_str(&content)?;
                list.file_path = path.clone();
                list.icon = Some("view-list-symbolic".to_string());
                let new_content = to_string(&list)?;
                fs::write(&path, new_content)?;
            }
        }
    }

    // Migrate tasks
    let tasks_dir = data_path.join("tasks");
    if tasks_dir.exists() {
        for list_dir in fs::read_dir(&tasks_dir)? {
            let list_dir = list_dir?;
            let list_path = list_dir.path();
            if list_path.is_dir() {
                for entry in fs::read_dir(&list_path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().map(|e| e == "ron").unwrap_or(false) {
                        let content = fs::read_to_string(&path)?;
                        let mut task: Task = from_str(&content)?;
                        task.path = list_path.clone();
                        // expanded is already #[serde(default)]
                        let new_content = to_string(&task)?;
                        fs::write(&path, new_content)?;

                        if !task.sub_tasks.is_empty() {
                            let sub_dir = list_path.join(&task.id);
                            if !sub_dir.exists() {
                                fs::create_dir_all(&sub_dir)?;
                            }
                            for sub_task in &mut task.sub_tasks {
                                sub_task.path = sub_dir.clone();
                                let sub_path = sub_dir.join(&sub_task.id).with_extension("ron");
                                let sub_content = to_string(&sub_task)?;
                                let mut file = fs::File::create(&sub_path)?;
                                file.write_all(sub_content.as_bytes())?;
                            }
                            task.sub_tasks.clear();
                            let new_content = to_string(&task)?;
                            fs::write(&path, new_content)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}


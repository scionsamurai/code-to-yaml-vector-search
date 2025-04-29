// src/services/yaml/management/mod.rs
use crate::services::file_service::FileService;
use crate::services::llm_service::LlmService;


pub mod generation;
pub mod embedding;
pub mod cleanup;


pub struct YamlManagement {
    file_service: FileService,
    llm_service: LlmService,
}

impl YamlManagement {
    pub fn new() -> Self {
        Self {
            file_service: FileService {},
            llm_service: LlmService {},
        }
    }


}
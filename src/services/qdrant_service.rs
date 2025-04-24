// src/services/qdrant_service.rs
use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{
    Distance, VectorParams, PointStruct, Value, 
    CreateCollection, VectorsConfig, Vectors,
    Condition, DeletePointsBuilder, Filter, // <-- Import Condition, DeletePointsBuilder, Filter
    UpsertPoints, // Keep this import
    SearchPoints, WithPayloadSelector // Keep these imports for the search function
};
use qdrant_client::qdrant::vectors_config::Config; 
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

pub struct QdrantService {
    client: Qdrant,
    vector_size: u64,
}

impl QdrantService {
    pub async fn new(url: &str, vector_size: u64) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let config = QdrantConfig::from_url(url);
        let client = Qdrant::new(config).unwrap(); // Consider using ? for error propagation
        Ok(QdrantService { client, vector_size })
    }
    
    pub async fn create_project_collection(
        &self,
        project_name: &str
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let collection_name = format!("project_{}", project_name);
        
        let collections = self.client.list_collections().await?;
        println!("collection_name: {:?}", &collection_name); // Keep for debugging if needed
        if collections.collections.iter().any(|c| &c.name == &collection_name) {
            println!("Collection '{}' already exists.", &collection_name); // Added log
            return Ok(());
        }
        
        println!("Creating collection '{}'", &collection_name); // Added log
        self.client.create_collection(CreateCollection {
            collection_name: collection_name.clone(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: self.vector_size,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        }).await?;
        println!("Collection '{}' created successfully.", collection_name); // Added log
        
        Ok(())
    }
    
    pub async fn store_file_embedding(
        &self,
        project_name: &str,
        file_path: &str,
        file_content: &str, // Changed name for clarity from example, assuming this is payload content
        embedding: Vec<f32>,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let collection_name = format!("project_{}", project_name);
        let point_id = Uuid::new_v4().to_string();
        
        // --- Corrected Deletion Logic ---
        // Use the DeletePointsBuilder to delete points based on a filter
        println!("Attempting to delete existing points for file: {}", file_path); // Added log
        self.client
            .delete_points(
                DeletePointsBuilder::new(collection_name.clone()) // Clone collection_name as it's used later
                    .points(Filter::must([Condition::matches( // Use Filter::must and Condition::matches
                        "file_path".to_string(),
                        file_path.to_string(), // Match the file_path field exactly
                    )]))
                    .wait(true), // Wait for the deletion to complete
            )
            .await?;
        println!("Deletion request completed for file: {}", file_path); // Added log
        // --- End of Corrected Deletion Logic ---

        // Create payload with file metadata
        let mut payload = HashMap::new();
        payload.insert("file_path".to_string(), Value::from(file_path));
        // Assuming the content you want to store is file_content, not yaml_content
        payload.insert("file_content".to_string(), Value::from(file_content)); 

        // Create the point struct
        let point = PointStruct {
            id: Some(point_id.clone().into()),
            vectors: Some(Vectors::from(embedding)),
            payload,
            ..Default::default()
        };
        
        // Use the UpsertPoints struct directly (this part was correct)
        let upsert_request = UpsertPoints {
            collection_name: collection_name, // No need to clone again here
            points: vec![point],
            wait: Some(true),
            ..Default::default()
        };
        
        println!("Upserting point for file: {}", file_path); // Added log
        self.client.upsert_points(upsert_request).await?;
        println!("Upsert successful for file: {}", file_path); // Added log
        
        Ok(point_id)
    }

    // In qdrant_service.rs
    pub async fn search_similar_files(
        &self,
        project_name: &str,
        query_embedding: Vec<f32>,
        limit: u64,
    ) -> Result<Vec<(String, String, f32)>, Box<dyn std::error::Error + Send + Sync>> {
        let collection_name = format!("project_{}", project_name);
        
        let search_request = SearchPoints {
            collection_name: collection_name.clone(), // Clone if needed elsewhere, good practice
            vector: query_embedding,
            limit: limit,
            with_payload: Some(WithPayloadSelector::from(true)),
            ..Default::default()
        };
        
        println!("Searching in collection '{}'", collection_name); // Added log
        let search_result = self.client.search_points(search_request).await?;
        println!("Search completed, found {} results", search_result.result.len()); // Added log
        
        // Extract results - return (file_path, file_content, score)
        // **Important:** Changed "yaml_content" to "file_content" to match what's stored
        let results = search_result.result
            .into_iter()
            .map(|point| {
                let file_path = point.payload.get("file_path")
                    .and_then(|v| v.kind.as_ref())
                    .and_then(|k| if let qdrant_client::qdrant::value::Kind::StringValue(s) = k {
                        Some(s.clone())
                    } else {
                        None
                    })
                    .unwrap_or_default();
                
                // Corrected payload key lookup
                let file_content = point.payload.get("file_content") 
                    .and_then(|v| v.kind.as_ref())
                    .and_then(|k| if let qdrant_client::qdrant::value::Kind::StringValue(s) = k {
                        Some(s.clone())
                    } else {
                        None
                    })
                    .unwrap_or_default();
                
                (file_path, file_content, point.score)
            })
            .collect();
        
        Ok(results)
    }
}
// src/services/qdrant_service.rs
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{
    Condition, CreateCollection, DeletePointsBuilder, Distance, Filter, PointStruct, SearchPoints,
    UpsertPointsBuilder, Value, VectorParams, VectorsConfig,
    WithPayloadSelector,
};
use qdrant_client::Qdrant;
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
        Ok(QdrantService {
            client,
            vector_size,
        })
    }

    pub async fn create_project_collection(
        &self,
        project_name: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let collection_name = format!("project_{}", project_name);

        let collections = self.client.list_collections().await?;
        if collections
            .collections
            .iter()
            .any(|c| &c.name == &collection_name)
        {
            return Ok(());
        }

        self.client
            .create_collection(CreateCollection {
                collection_name: collection_name.clone(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: self.vector_size,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await?;

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

        println!(
            "Attempting to delete existing points for file: {}",
            file_path
        ); // Added log
        self.client
            .delete_points(
                DeletePointsBuilder::new(collection_name.clone()) // Clone collection_name as it's used later
                    .points(Filter::must([Condition::matches(
                        // Use Filter::must and Condition::matches
                        "file_path".to_string(),
                        file_path.to_string(), // Match the file_path field exactly
                    )]))
                    .wait(true), // Wait for the deletion to complete
            )
            .await?;
        println!("Deletion request completed for file: {}", file_path); // Added log

        // Create payload with file metadata
        let mut payload = HashMap::new();
        payload.insert("file_path".to_string(), Value::from(file_path));
        // Assuming the content you want to store is file_content, not yaml_content
        payload.insert("file_content".to_string(), Value::from(file_content));

        // Create the point struct with explicit vector assignment
        let point = PointStruct::new(
            point_id.clone(),
            embedding, // This is the key fix
            payload,
        );

        println!(
            "Sending embedding for point_id.clone(): {}",
            point_id.clone()
        );
        println!("Vector data present: {}", point.vectors.is_some()); // Check if vectors are set

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection_name, vec![point]).wait(true))
            .await?;

        println!("Upsert successful for file: {}", file_path); // Added log

        Ok(point_id)
    }

    // In qdrant_service.rs
    pub async fn search_similar_files(
        &self,
        project_name: &str,
        query_embedding: Vec<f32>,
        limit: u64,
        return_embeddings: bool,
    ) -> Result<Vec<(String, String, f32, Option<Vec<f32>>)>, Box<dyn std::error::Error + Send + Sync>> {
        let collection_name = format!("project_{}", project_name);

        let search_request = SearchPoints {
            collection_name: collection_name.clone(), // Clone if needed elsewhere, good practice
            vector: query_embedding,
            limit: limit,
            with_payload: Some(WithPayloadSelector::from(true)),
            with_vectors: Some(return_embeddings.into()),
            ..Default::default()
        };

        let search_result = self.client.search_points(search_request).await?;

        let results = search_result
            .result
            .into_iter()
            .map(|point| {
                let file_path = point
                    .payload
                    .get("file_path")
                    .and_then(|v| v.kind.as_ref())
                    .and_then(|k| {
                        if let qdrant_client::qdrant::value::Kind::StringValue(s) = k {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                // Corrected payload key lookup
                let file_content = point
                    .payload
                    .get("file_content")
                    .and_then(|v| v.kind.as_ref())
                    .and_then(|k| {
                        if let qdrant_client::qdrant::value::Kind::StringValue(s) = k {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                // Extract vectors if present
                let vectors = point.vectors.and_then(|v| match v.vectors_options {
                    Some(qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(
                        vector_output,
                    )) => Some(vector_output.data),
                    _ => None,
                });

                (file_path, file_content, point.score, vectors)
            })
            .collect();

        Ok(results)
    }

    // Add this method to QdrantService implementation
    pub async fn delete_file_vectors(
        &self,
        project_name: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let collection_name = format!("project_{}", project_name);

        println!(
            "Deleting vectors for file: {} in collection {}",
            file_path, collection_name
        );

        // Use DeletePointsBuilder to delete points based on file_path
        self.client
            .delete_points(
                DeletePointsBuilder::new(collection_name)
                    .points(Filter::must([Condition::matches(
                        "file_path".to_string(),
                        file_path.to_string(),
                    )]))
                    .wait(true),
            )
            .await?;

        println!("Successfully deleted vectors for file: {}", file_path);

        Ok(())
    }
}

// src/routes/project/cluster.rs
use crate::services::clustering_service::cluster_embeddings;
use crate::services::qdrant_service::QdrantService;
use actix_web::{post, web, Error, HttpResponse};
use serde_json::json;
use std::env;

#[post("/api/cluster/{project_name}")]
pub async fn cluster_project_embeddings(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let project_name = path.into_inner();
    println!("Received cluster request for project: {}", project_name);

    // Get Qdrant URL from environment variable
    let qdrant_server_url =
        env::var("QDRANT_SERVER_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());

    // Create a new QdrantService instance
    let qdrant_service = QdrantService::new(&qdrant_server_url, 1536)
        .await
        .map_err(|e| {
            eprintln!("Failed to create Qdrant service: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to connect to Qdrant")
        })?;

    // 1. Retrieve embeddings from Qdrant
    let search_results: Vec<(String, String, f32, Option<Vec<f32>>)> = qdrant_service
        .search_similar_files(&project_name, vec![0.0; 1536], 1000, true)
        .await
        .map_err(|e| {
            eprintln!("Error retrieving embeddings: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to retrieve embeddings from Qdrant")
        })?;
    let embeddings: Vec<Vec<f32>> = search_results
    .iter()
    .filter_map(|(_, _, _, vectors)| vectors.as_ref().cloned()) // Extract and clone the vectors
    .collect();

    let associated_files: Vec<String> = search_results
        .iter()
        .filter_map(|(path, _, _, _)| path.clone().into())
        .collect();

    println!("Number of embeddings: {}", embeddings.len());

    // 2. Perform clustering
    let n_clusters = 10; // Adjust as needed
    let max_iterations = 100; // Adjust as needed

    let clustering_result =
        cluster_embeddings(embeddings, n_clusters, max_iterations).map_err(|e| {
            eprintln!("Error performing clustering: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to perform clustering")
        })?;

    println!("Clustering result: {:?}", clustering_result);

    // generate a Vec<String, usize> mapping of file paths to cluster indices and then sort by indices
    let mut file_cluster_mapping: Vec<(String, usize)> = associated_files
        .into_iter()
        .zip(clustering_result.clone().into_iter())
        .collect();
    file_cluster_mapping.sort_by_key(|(_, cluster_index)| *cluster_index);
    
    // println!("file_cluster_mapping result: {:?}", file_cluster_mapping);
    for (file_path, cluster_index) in &file_cluster_mapping {
        println!("File: {}, Cluster Index: {}", file_path, cluster_index);
    }

    // 3. Prepare and return the response
    let response_json = json!({
        "project": project_name,
        "clusters": clustering_result,
    });

    Ok(HttpResponse::Ok().json(response_json))
}

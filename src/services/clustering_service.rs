// src/services/clustering_service.rs

use ndarray::Array2;
use dbscan::Dbscan;
use crate::services::qdrant_service::QdrantService;
use std::error::Error;

pub struct ClusteringService;

impl ClusteringService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn cluster_project_files(
        &self,
        project_name: &str,
        epsilon: f64,
        min_points: usize,
        qdrant_url: &str,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error + Send + Sync>> {
        let qdrant_service = QdrantService::new(qdrant_url, 1536).await?; // Assuming 1536 is your embedding size

        // 1. Fetch Embeddings
        let embeddings_with_filepaths = self.get_all_file_embeddings(project_name, &qdrant_service).await?;

        if embeddings_with_filepaths.is_empty() {
            return Ok(vec![]); // Or handle the empty case as needed
        }

        let (file_paths, embeddings): (Vec<String>, Vec<Vec<f32>>) = embeddings_with_filepaths.into_iter().unzip();

        // 2. Convert to ndarray
        let num_embeddings = embeddings.len();
        let embedding_size = embeddings[0].len(); // Assuming all embeddings have the same size

        let mut data = Vec::with_capacity(num_embeddings * embedding_size);
        for embedding in embeddings {
            data.extend(embedding);
        }
        let data_array = Array2::from_shape_vec((num_embeddings, embedding_size), data)?;

        // 3. Perform DBSCAN Clustering
        let dbscan: Dbscan<f64> = dbscan(epsilon, min_points);
        let clusters = dbscan.run(&data_array);

        // 4. Group file paths by cluster
        let mut clustered_files: Vec<Vec<String>> = Vec::new();
        for cluster_id in clusters.as_slice() {
            match cluster_id {
                Some(cluster_id) => {
                    // Ensure there is a vector for cluster_id
                    while clustered_files.len() <= *cluster_id {
                        clustered_files.push(Vec::new());
                    }
                    clustered_files[*cluster_id].push(file_paths[*cluster_id].clone());
                }
                None => {
                    // Handle noise points (unclustered files) - maybe put them in their own "noise" cluster or ignore them
                    println!("File {} is a noise point", file_paths[*clusters.len() - 1]);
                }
            }
        }

        Ok(clustered_files)
    }

    async fn get_all_file_embeddings(&self, project_name: &str, qdrant_service: &QdrantService) -> Result<Vec<(String, Vec<f32>)>, Box<dyn Error + Send + Sync>> {
        let collection_name = format!("project_{}", project_name);
        
        // Fetch all points from the collection.  This is inefficient for large collections.
        let all_points = qdrant_service.client.scroll(
            collection_name,
            None,    // Offset - start from the beginning
            None,    // Limit - fetch all points in one go if possible
            true,    // With payload
            true     // With vectors
        ).await?;
        
        let mut embeddings_with_filepaths = Vec::new();

        for point in all_points.result {
            let file_path = point.payload.get("file_path")
                .and_then(|v| v.kind.as_ref())
                .and_then(|k| if let qdrant_client::qdrant::value::Kind::StringValue(s) = k {
                    Some(s.clone())
                } else {
                    None
                })
                .unwrap_or_default();

            let embedding = point.vectors.map(|vectors| {
                match vectors.vectors {
                    Some(qdrant_client::qdrant::vectors::Vectors::Vector(vector)) => vector.data,
                    _ => Vec::new() //Or return an error
                }
            }).unwrap_or_default();

            if !file_path.is_empty() && !embedding.is_empty() {
                embeddings_with_filepaths.push((file_path, embedding));
            }
        }
        Ok(embeddings_with_filepaths)
    }
}
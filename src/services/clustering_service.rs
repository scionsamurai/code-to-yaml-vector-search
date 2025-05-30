// src/services/clustering_service.rs
pub fn cluster_embeddings(
    embeddings: Vec<Vec<f32>>,
    n_clusters: usize,
    max_iterations: usize,
) -> Result<Vec<usize>, Box<dyn std::error::Error + Send + Sync>> {
    if embeddings.is_empty() {
        return Ok(Vec::new());
    }

    let clustering_result: CosineClustering = kmeans_cosine(n_clusters, &embeddings, max_iterations);

    Ok(clustering_result.membership)
}

use rand::Rng;

pub struct CosineClustering {
    pub membership: Vec<usize>,
    pub centroids: Vec<Vec<f32>>,
    pub inertia: f32, // Sum of distances from points to their centroids
}

/// Performs k-means clustering using cosine distance
pub fn kmeans_cosine(
    k: usize,
    data: &[Vec<f32>],
    max_iter: usize,
) -> CosineClustering {
    if data.is_empty() || k == 0 {
        return CosineClustering {
            membership: vec![],
            centroids: vec![],
            inertia: 0.0,
        };
    }

    if k >= data.len() {
        // Each point gets its own cluster
        let membership: Vec<usize> = (0..data.len()).collect();
        let centroids: Vec<Vec<f32>> = data.iter().map(|v| normalize_vector(v)).collect();
        return CosineClustering {
            membership,
            centroids,
            inertia: 0.0,
        };
    }

    let mut membership = vec![0; data.len()];
    let mut centroids = initialize_centroids_cosine(k, data);

    for _iter in 0..max_iter {
        let mut changes = 0;

        // Assignment step: assign each point to nearest centroid
        for (i, point) in data.iter().enumerate() {
            let old_cluster = membership[i];
            let mut best_cluster = 0;
            let mut min_distance = f32::INFINITY;

            for (c, centroid) in centroids.iter().enumerate() {
                let dist = cosine_distance(point, centroid);
                if dist < min_distance {
                    min_distance = dist;
                    best_cluster = c;
                }
            }

            if best_cluster != old_cluster {
                membership[i] = best_cluster;
                changes += 1;
            }
        }

        // Update step: recompute centroids
        update_centroids_cosine(&mut centroids, data, &membership);

        // Check for convergence
        if changes == 0 {
            break;
        }
    }

    // Calculate final inertia
    let inertia = calculate_inertia(data, &centroids, &membership);

    CosineClustering {
        membership,
        centroids,
        inertia,
    }
}

/// Calculates cosine distance between two vectors (1 - cosine_similarity)
fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0; // Maximum distance for zero vectors
    }

    let cosine_sim = dot_product / (norm_a * norm_b);
    
    // Clamp to [-1, 1] to handle floating point errors
    let cosine_sim = cosine_sim.max(-1.0).min(1.0);
    
    1.0 - cosine_sim
}

/// Normalizes a vector to unit length
fn normalize_vector(vec: &[f32]) -> Vec<f32> {
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 {
        vec.to_vec()
    } else {
        vec.iter().map(|x| x / norm).collect()
    }
}

/// Initialize centroids using k-means++ adapted for cosine similarity
fn initialize_centroids_cosine(k: usize, data: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let mut rng = rand::rng();
    let mut centroids = Vec::with_capacity(k);
    let mut taken = vec![false; data.len()];

    // Choose first centroid randomly
    let first_idx = rng.random_range(0..data.len());
    taken[first_idx] = true;
    centroids.push(normalize_vector(&data[first_idx]));

    // Choose remaining centroids using k-means++ logic with cosine distance
    for _ in 1..k {
        let mut max_distance = -1.0;
        let mut best_idx = 0;

        for (i, point) in data.iter().enumerate() {
            if taken[i] {
                continue;
            }

            // Find minimum distance to existing centroids
            let min_dist = centroids
                .iter()
                .map(|centroid| cosine_distance(point, centroid))
                .fold(f32::INFINITY, f32::min);

            if min_dist > max_distance {
                max_distance = min_dist;
                best_idx = i;
            }
        }

        taken[best_idx] = true;
        centroids.push(normalize_vector(&data[best_idx]));
    }

    centroids
}

/// Updates centroids by computing normalized mean of assigned points
fn update_centroids_cosine(
    centroids: &mut [Vec<f32>],
    data: &[Vec<f32>],
    membership: &[usize],
) {
    let k = centroids.len();

    // Reset centroids
    for centroid in centroids.iter_mut() {
        centroid.fill(0.0);
    }
    let mut counts = vec![0; k];

    // Sum points in each cluster
    for (i, point) in data.iter().enumerate() {
        let cluster = membership[i];
        counts[cluster] += 1;
        for (j, &value) in point.iter().enumerate() {
            centroids[cluster][j] += value;
        }
    }

    // Compute mean and normalize each centroid
    for (centroid, &count) in centroids.iter_mut().zip(counts.iter()) {
        if count > 0 {
            // Take arithmetic mean
            for value in centroid.iter_mut() {
                *value /= count as f32;
            }
            
            // Normalize to unit length for cosine similarity
            let norm: f32 = centroid.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for value in centroid.iter_mut() {
                    *value /= norm;
                }
            }
        } else {
            // Handle empty clusters by reinitializing randomly
            // This is a simple strategy - you might want something more sophisticated
            let mut rng = rand::rng();
            for value in centroid.iter_mut() {
                *value = rng.random_range(-1.0..1.0);
            }
            *centroid = normalize_vector(centroid);
        }
    }
}

/// Calculate total inertia (sum of squared distances from points to centroids)
fn calculate_inertia(
    data: &[Vec<f32>],
    centroids: &[Vec<f32>],
    membership: &[usize],
) -> f32 {
    data.iter()
        .zip(membership.iter())
        .map(|(point, &cluster)| {
            let dist = cosine_distance(point, &centroids[cluster]);
            dist * dist
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let dist = cosine_distance(&a, &b);
        assert!((dist - 1.0).abs() < 1e-10); // Should be 1.0 (orthogonal vectors)

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![1.0, 0.0, 0.0];
        let dist2 = cosine_distance(&c, &d);
        assert!(dist2.abs() < 1e-10); // Should be 0.0 (identical vectors)
    }

    #[test]
    fn test_clustering() {
        let data = vec![
            // Cluster 1: similar vectors
            vec![1.0, 1.0, 0.0],
            vec![1.1, 0.9, 0.0],
            vec![0.9, 1.1, 0.0],
            // Cluster 2: different vectors
            vec![0.0, 0.0, 1.0],
            vec![0.0, 0.1, 0.9],
            vec![0.1, 0.0, 1.0],
        ];

        let result = kmeans_cosine(2, &data, 100);
        
        assert_eq!(result.membership.len(), data.len());
        assert_eq!(result.centroids.len(), 2);
        
        // Check that similar vectors are in the same cluster
        assert_eq!(result.membership[0], result.membership[1]);
        assert_eq!(result.membership[1], result.membership[2]);
        assert_eq!(result.membership[3], result.membership[4]);
        assert_eq!(result.membership[4], result.membership[5]);
        
        // Check that different groups are in different clusters
        assert_ne!(result.membership[0], result.membership[3]);
    }

    #[test]
    fn test_normalize_vector() {
        let vec = vec![3.0, 4.0];
        let normalized = normalize_vector(&vec);
        let norm: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }
}
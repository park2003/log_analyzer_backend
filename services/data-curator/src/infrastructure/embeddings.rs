use crate::application::EmbeddingService;
use anyhow::{Context, Result};
use async_trait::async_trait;
use ndarray::Array4;
use once_cell::sync::Lazy;
use ort::{session::Session, value::Value};
use std::sync::{Arc, Mutex};

// CLIP model dimensions
const IMAGE_SIZE: u32 = 224;
const EMBEDDING_DIM: usize = 768;

// Initialize ONNX Runtime on first use
static ORT_INIT: Lazy<()> = Lazy::new(|| {
    ort::init()
        .with_name("clip_embedding")
        .commit()
        .expect("Failed to initialize ONNX Runtime");
});

pub struct ClipEmbeddingService {
    session: Arc<Mutex<Session>>,
}

impl ClipEmbeddingService {
    pub async fn new(model_path: &str) -> Result<Self> {
        // Ensure ONNX Runtime is initialized
        Lazy::force(&ORT_INIT);

        let session = Session::builder()?.commit_from_file(model_path)?;

        Ok(Self {
            session: Arc::new(Mutex::new(session)),
        })
    }

    pub async fn new_with_download() -> Result<Self> {
        // In production, download CLIP model from a trusted source
        // For now, assume model is provided
        let model_path = std::env::var("CLIP_MODEL_PATH")
            .unwrap_or_else(|_| "models/clip-vit-base-patch32.onnx".to_string());

        Self::new(&model_path).await
    }

    fn preprocess_image(&self, image_data: &[u8]) -> Result<Array4<f32>> {
        // Load image from bytes
        let img =
            image::load_from_memory(image_data).context("Failed to load image from memory")?;

        // Resize to CLIP input size
        let resized = img.resize_exact(
            IMAGE_SIZE,
            IMAGE_SIZE,
            image::imageops::FilterType::Lanczos3,
        );

        // Convert to RGB
        let rgb_image = resized.to_rgb8();

        // Normalize using CLIP's normalization values
        // Mean: [0.48145466, 0.4578275, 0.40821073]
        // Std: [0.26862954, 0.26130258, 0.27577711]
        let mean: [f32; 3] = [0.48145466, 0.4578275, 0.40821073];
        let std: [f32; 3] = [0.26862954, 0.2613026, 0.2757771];

        // Create normalized array [1, 3, 224, 224]
        let mut array = Array4::<f32>::zeros((1, 3, IMAGE_SIZE as usize, IMAGE_SIZE as usize));

        for (x, y, pixel) in rgb_image.enumerate_pixels() {
            let channels = pixel.0;
            for (c, &value) in channels.iter().enumerate() {
                let normalized = (value as f32 / 255.0 - mean[c]) / std[c];
                array[[0, c, y as usize, x as usize]] = normalized;
            }
        }

        Ok(array)
    }
}

#[async_trait]
impl EmbeddingService for ClipEmbeddingService {
    async fn generate_embedding(&self, image_data: &[u8]) -> Result<Vec<f32>> {
        // Preprocess image
        let input_array = self.preprocess_image(image_data)?;

        // Convert to ONNX Runtime tensor
        let input_tensor = Value::from_array(input_array)?;

        // Run inference using ort::inputs! macro - lock mutex for mutable access
        let mut session = self.session.lock().unwrap();
        let outputs = session.run(ort::inputs![input_tensor])?;

        // Extract embedding from first output
        let output = &outputs[0];
        let tensor_data = output.try_extract_tensor::<f32>()?;

        // Convert to Vec<f32> - tensor_data is (&Shape, &[f32])
        let embedding = tensor_data.1.to_vec();

        // Normalize embedding (L2 normalization for cosine similarity)
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        let normalized_embedding = if norm > 0.0 {
            embedding.iter().map(|x| x / norm).collect()
        } else {
            embedding
        };

        Ok(normalized_embedding)
    }
}

// Mock implementation for testing without ONNX model
pub struct MockEmbeddingService;

impl MockEmbeddingService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EmbeddingService for MockEmbeddingService {
    async fn generate_embedding(&self, _image_data: &[u8]) -> Result<Vec<f32>> {
        // Generate random embedding for testing
        use rand::Rng;
        let mut rng = rand::rng();

        let embedding: Vec<f32> = (0..EMBEDDING_DIM)
            .map(|_| rng.random_range(-1.0..1.0))
            .collect();

        // Normalize
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        let normalized = embedding.iter().map(|x| x / norm).collect();

        Ok(normalized)
    }
}

// Additional utility functions for clustering
pub mod clustering {
    use ndarray::Array2;

    pub fn find_cluster_boundaries(embeddings: &[Vec<f32>], n_clusters: usize) -> Vec<usize> {
        if embeddings.is_empty() || n_clusters == 0 {
            return Vec::new();
        }

        // Convert to ndarray format
        let n_samples = embeddings.len();
        let n_features = embeddings[0].len();
        let mut data = Array2::<f32>::zeros((n_samples, n_features));

        for (i, embedding) in embeddings.iter().enumerate() {
            for (j, &value) in embedding.iter().enumerate() {
                data[[i, j]] = value;
            }
        }

        // Simplified clustering - just take evenly spaced samples
        // In production, use proper k-means clustering
        let mut boundary_indices = Vec::new();

        if n_samples > 0 {
            let step = n_samples.max(1) / n_clusters.max(1);
            for i in 0..n_clusters {
                let idx = (i * step).min(n_samples - 1);
                if !boundary_indices.contains(&idx) {
                    boundary_indices.push(idx);
                }
            }
        }

        boundary_indices
    }
}

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::prelude::*;

pub struct WorkloadDistributor {
    num_workers: usize,
    batch_size: usize,
    total_processed: Arc<AtomicUsize>,
}

impl WorkloadDistributor {
    pub fn new(num_workers: usize, batch_size: usize) -> Self {
        Self {
            num_workers,
            batch_size,
            total_processed: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn distribute<T, F, R>(&self, items: Vec<T>, processor: F) -> Vec<R>
    where
        T: Send,
        F: Fn(T) -> R + Send + Sync,
        R: Send,
    {
        let total_processed = self.total_processed.clone();

        items
            .into_par_iter()
            .map(|item| {
                let result = processor(item);
                total_processed.fetch_add(1, Ordering::Relaxed);
                result
            })
            .collect()
    }

    pub fn distribute_batched<T, F, R>(&self, items: Vec<T>, processor: F) -> Vec<R>
    where
        T: Send + Clone,
        F: Fn(Vec<T>) -> Vec<R> + Send + Sync,
        R: Send,
    {
        let batches: Vec<Vec<T>> = items
            .into_iter()
            .collect::<Vec<_>>()
            .chunks(self.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let total_processed = self.total_processed.clone();

        batches
            .into_par_iter()
            .flat_map(|batch| {
                let results = processor(batch);
                total_processed.fetch_add(self.batch_size, Ordering::Relaxed);
                results
            })
            .collect()
    }

    pub fn total_processed(&self) -> usize {
        self.total_processed.load(Ordering::Relaxed)
    }

    pub fn reset_counter(&self) {
        self.total_processed.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distribute() {
        let distributor = WorkloadDistributor::new(4, 10);
        let items = vec![1, 2, 3, 4, 5];
        let results = distributor.distribute(items, |x| x * 2);
        assert_eq!(results.len(), 5);
        assert!(results.contains(&2));
        assert!(results.contains(&10));
    }
}

use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use crate::utils::limited_list::LimitedList;

pub struct LimitedListWithTimeout<T>
    where
        T: Eq + Clone + Debug {
    inner: Arc<Mutex<LimitedList<T>>>,
}

impl<T: Send + 'static + Clone + Debug> LimitedListWithTimeout<T>
    where
        T: Eq + Clone + Debug {
    pub fn new(limit: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LimitedList::new(limit))),
        }
    }

    pub async fn add(&mut self, item: T) {
        {
            let mut list = self.inner.lock().await;
            list.add(item.clone());
        }

        let inner_clone = Arc::clone(&self.inner);
        tokio::spawn(async move {
            sleep(Duration::from_secs(30 * 60)).await;
            
            let mut list = inner_clone.lock().await;
            list.del(&item);
        });
    }

    pub async fn all(&self) -> Vec<T> {
        let list = self.inner.lock().await;
        list.all().into_iter().cloned().collect()
    }

    pub async fn del(&mut self, item: &T) {
        let mut list = self.inner.lock().await;
        list.del(item)
    }
    
    pub async fn is_full(&self) -> bool {
        let list = self.inner.lock().await;
        list.is_full()
    }
}
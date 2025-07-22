use crate::browser::BrowserManager;
use crate::error::AppResult;
use log::{debug, error, info, warn};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

/// Browser instance status
#[derive(Debug, Clone, PartialEq)]
pub enum BrowserStatus {
    Available,
    Busy,
    Failed,
    Recovering,
}

/// Browser instance with metadata
pub struct BrowserInstance {
    pub id: String,
    pub platform: String,
    pub status: BrowserStatus,
    pub created_at: u64,
    pub last_used: u64,
    pub task_count: u64,
    pub failure_count: u64,
    pub manager: Option<BrowserManager>,
}

impl BrowserInstance {
    pub fn new(id: String, platform: String) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Self {
            id,
            platform,
            status: BrowserStatus::Available,
            created_at: now,
            last_used: now,
            task_count: 0,
            failure_count: 0,
            manager: None,
        }
    }

    pub async fn initialize(&mut self) -> AppResult<()> {
        self.manager = Some(BrowserManager::new()?);
        info!("Browser instance {} initialized for {}", self.id, self.platform);
        Ok(())
    }

    pub fn mark_used(&mut self) {
        self.last_used = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.task_count += 1;
        self.status = BrowserStatus::Busy;
    }

    pub fn mark_available(&mut self) {
        self.status = BrowserStatus::Available;
    }

    pub fn mark_failed(&mut self) {
        self.failure_count += 1;
        self.status = BrowserStatus::Failed;
    }

    pub fn is_healthy(&self) -> bool {
        self.failure_count < 3 && self.status != BrowserStatus::Failed
    }
}

/// Load balancer configuration
#[derive(Debug, Clone)]
pub struct LoadBalancerConfig {
    pub min_instances_per_platform: usize,
    pub max_instances_per_platform: usize,
    pub instance_timeout_seconds: u64,
    pub health_check_interval_seconds: u64,
    pub auto_scaling_enabled: bool,
    pub max_tasks_per_instance: u64,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            min_instances_per_platform: 1,
            max_instances_per_platform: 3,
            instance_timeout_seconds: 300, // 5 minutes
            health_check_interval_seconds: 60,
            auto_scaling_enabled: true,
            max_tasks_per_instance: 10,
        }
    }
}

/// Browser load balancer
pub struct BrowserLoadBalancer {
    config: LoadBalancerConfig,
    instances: Arc<RwLock<Vec<BrowserInstance>>>,
    request_queue: Arc<Mutex<VecDeque<PendingRequest>>>,
    is_running: Arc<Mutex<bool>>,
}

/// Pending browser request
#[derive(Debug)]
pub struct PendingRequest {
    pub platform: String,
    pub task_type: String,
    pub priority: u8, // 0 = highest, 255 = lowest
    pub created_at: u64,
}

impl BrowserLoadBalancer {
    /// Create a new browser load balancer
    pub fn new(config: LoadBalancerConfig) -> Self {
        Self {
            config,
            instances: Arc::new(RwLock::new(Vec::new())),
            request_queue: Arc::new(Mutex::new(VecDeque::new())),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Initialize the load balancer
    pub async fn initialize(&self) -> AppResult<()> {
        // Create initial instances for each platform
        let platforms = ["facebook", "instagram", "youtube", "tiktok"];
        
        for platform in platforms {
            for i in 0..self.config.min_instances_per_platform {
                let instance_id = format!("{}_{}", platform, i);
                let mut instance = BrowserInstance::new(instance_id, platform.to_string());
                instance.initialize().await?;
                
                let mut instances = self.instances.write().await;
                instances.push(instance);
            }
        }

        info!("Browser load balancer initialized with {} instances", 
              self.config.min_instances_per_platform * platforms.len());
        Ok(())
    }

    /// Start the load balancer background tasks
    pub async fn start(&self) -> AppResult<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        info!("Starting browser load balancer");

        // Start health check task
        let instances = Arc::clone(&self.instances);
        let config = self.config.clone();
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            while *is_running.lock().await {
                if let Err(e) = Self::health_check_task(&instances, &config).await {
                    error!("Health check task failed: {}", e);
                }
                sleep(Duration::from_secs(config.health_check_interval_seconds)).await;
            }
            info!("Health check task stopped");
        });

        // Start request processing task
        let instances = Arc::clone(&self.instances);
        let request_queue = Arc::clone(&self.request_queue);
        let config = self.config.clone();
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            while *is_running.lock().await {
                if let Err(e) = Self::process_request_queue(&instances, &request_queue, &config).await {
                    error!("Request processing task failed: {}", e);
                }
                sleep(Duration::from_millis(100)).await; // Check queue frequently
            }
            info!("Request processing task stopped");
        });

        info!("Browser load balancer started");
        Ok(())
    }

    /// Stop the load balancer
    pub async fn stop(&self) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        info!("Browser load balancer stopping");
    }

    /// Get an available browser instance for a platform
    pub async fn get_browser_instance(&self, platform: &str) -> AppResult<Option<String>> {
        let mut instances = self.instances.write().await;
        
        // Find an available instance for the platform
        for instance in instances.iter_mut() {
            if instance.platform == platform && 
               instance.status == BrowserStatus::Available && 
               instance.is_healthy() {
                instance.mark_used();
                return Ok(Some(instance.id.clone()));
            }
        }

        // No available instance found
        if self.config.auto_scaling_enabled {
            // Try to create a new instance if under the limit
            let platform_instances = instances.iter()
                .filter(|i| i.platform == platform)
                .count();
                
            if platform_instances < self.config.max_instances_per_platform {
                let instance_id = format!("{}_{}", platform, platform_instances);
                let mut new_instance = BrowserInstance::new(instance_id.clone(), platform.to_string());
                
                if new_instance.initialize().await.is_ok() {
                    new_instance.mark_used();
                    instances.push(new_instance);
                    info!("Created new browser instance: {}", instance_id);
                    return Ok(Some(instance_id));
                }
            }
        }

        Ok(None)
    }

    /// Release a browser instance back to the pool
    pub async fn release_browser_instance(&self, instance_id: &str, success: bool) -> AppResult<()> {
        let mut instances = self.instances.write().await;
        
        if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
            if success {
                instance.mark_available();
                debug!("Released browser instance: {}", instance_id);
            } else {
                instance.mark_failed();
                warn!("Marked browser instance as failed: {}", instance_id);
            }
        }

        Ok(())
    }

    /// Add a request to the queue
    pub async fn queue_request(&self, platform: &str, task_type: &str, priority: u8) -> AppResult<()> {
        let request = PendingRequest {
            platform: platform.to_string(),
            task_type: task_type.to_string(),
            priority,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        let mut queue = self.request_queue.lock().await;
        
        // Insert request based on priority (lower number = higher priority)
        let insert_pos = queue.iter()
            .position(|r| r.priority > priority)
            .unwrap_or(queue.len());
            
        queue.insert(insert_pos, request);
        
        debug!("Queued {} request for {} with priority {}", task_type, platform, priority);
        Ok(())
    }

    /// Health check task
    async fn health_check_task(
        instances: &Arc<RwLock<Vec<BrowserInstance>>>,
        config: &LoadBalancerConfig,
    ) -> AppResult<()> {
        let mut instances = instances.write().await;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        for instance in instances.iter_mut() {
            // Check for timed out instances
            if instance.status == BrowserStatus::Busy && 
               now - instance.last_used > config.instance_timeout_seconds {
                warn!("Browser instance {} timed out", instance.id);
                instance.mark_failed();
            }

            // Try to recover failed instances
            if instance.status == BrowserStatus::Failed && instance.failure_count < 3 {
                instance.status = BrowserStatus::Recovering;
                
                // Try to reinitialize
                if instance.initialize().await.is_ok() {
                    instance.status = BrowserStatus::Available;
                    instance.failure_count = 0;
                    info!("Recovered browser instance: {}", instance.id);
                } else {
                    instance.mark_failed();
                }
            }

            // Remove instances that have exceeded max tasks
            if instance.task_count > config.max_tasks_per_instance {
                instance.status = BrowserStatus::Failed;
                warn!("Browser instance {} exceeded max tasks ({})", instance.id, instance.task_count);
            }
        }

        // Remove permanently failed instances
        instances.retain(|instance| instance.failure_count < 5);

        Ok(())
    }

    /// Process request queue
    async fn process_request_queue(
        instances: &Arc<RwLock<Vec<BrowserInstance>>>,
        request_queue: &Arc<Mutex<VecDeque<PendingRequest>>>,
        _config: &LoadBalancerConfig,
    ) -> AppResult<()> {
        let mut queue = request_queue.lock().await;
        
        if queue.is_empty() {
            return Ok(());
        }

        let instances_read = instances.read().await;
        
        // Process requests if we have available instances
        while let Some(request) = queue.front() {
            let has_available = instances_read.iter().any(|i| 
                i.platform == request.platform && 
                i.status == BrowserStatus::Available && 
                i.is_healthy()
            );

            if has_available {
                let request = queue.pop_front().unwrap();
                debug!("Processing queued {} request for {}", request.task_type, request.platform);
                // Request will be processed by the application
            } else {
                break; // No available instances, wait for next cycle
            }
        }

        Ok(())
    }

    /// Get load balancer statistics
    pub async fn get_statistics(&self) -> LoadBalancerStats {
        let instances = self.instances.read().await;
        let queue = self.request_queue.lock().await;

        let mut stats = LoadBalancerStats::default();
        stats.total_instances = instances.len();
        stats.queued_requests = queue.len();

        for instance in instances.iter() {
            match instance.status {
                BrowserStatus::Available => stats.available_instances += 1,
                BrowserStatus::Busy => stats.busy_instances += 1,
                BrowserStatus::Failed => stats.failed_instances += 1,
                BrowserStatus::Recovering => stats.recovering_instances += 1,
            }

            stats.total_tasks += instance.task_count;
            stats.total_failures += instance.failure_count;
        }

        stats
    }
}

/// Load balancer statistics
#[derive(Debug, Default)]
pub struct LoadBalancerStats {
    pub total_instances: usize,
    pub available_instances: usize,
    pub busy_instances: usize,
    pub failed_instances: usize,
    pub recovering_instances: usize,
    pub queued_requests: usize,
    pub total_tasks: u64,
    pub total_failures: u64,
}

/// Global browser load balancer
static mut LOAD_BALANCER: Option<BrowserLoadBalancer> = None;
static LOAD_BALANCER_INIT: std::sync::Once = std::sync::Once::new();

/// Get the global browser load balancer
pub fn get_load_balancer() -> &'static BrowserLoadBalancer {
    unsafe {
        LOAD_BALANCER_INIT.call_once(|| {
            let config = LoadBalancerConfig::default();
            LOAD_BALANCER = Some(BrowserLoadBalancer::new(config));
        });
        LOAD_BALANCER.as_ref().unwrap()
    }
}

/// Initialize browser load balancing
pub async fn init_browser_load_balancing() -> AppResult<()> {
    let balancer = get_load_balancer();
    balancer.initialize().await?;
    balancer.start().await?;
    info!("Browser load balancing initialized and started");
    Ok(())
}

/// Stop browser load balancing
pub async fn stop_browser_load_balancing() {
    let balancer = get_load_balancer();
    balancer.stop().await;
}

/// Get an available browser for a platform
pub async fn get_available_browser(platform: &str) -> AppResult<Option<String>> {
    let balancer = get_load_balancer();
    balancer.get_browser_instance(platform).await
}

/// Release a browser back to the pool
pub async fn release_browser(instance_id: &str, success: bool) -> AppResult<()> {
    let balancer = get_load_balancer();
    balancer.release_browser_instance(instance_id, success).await
}
use crate::error::{AppError, AppResult};
use log::{debug, error, info, warn};
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Network configuration for resilience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub timeout_seconds: u64,
    pub fallback_dns: Vec<String>,
    pub proxy_urls: Vec<String>,
    pub enable_tor: bool,
    pub user_agents: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            retry_attempts: 5,
            retry_delay_ms: 2000,
            timeout_seconds: 30,
            fallback_dns: vec![
                "8.8.8.8".to_string(),
                "1.1.1.1".to_string(),
                "9.9.9.9".to_string(),
            ],
            proxy_urls: vec![],
            enable_tor: false,
            user_agents: vec![
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string(),
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string(),
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string(),
            ],
        }
    }
}

/// Network connection status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Degraded,
    Disconnected,
    ProxyMode,
    TorMode,
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub retries_used: u64,
    pub proxy_switches: u64,
    pub dns_fallbacks: u64,
    pub last_success_time: Option<u64>,
    pub last_failure_time: Option<u64>,
}

/// Resilient HTTP client with automatic fallbacks
pub struct ResilientClient {
    config: NetworkConfig,
    clients: Vec<Client>,
    current_client_index: usize,
    status: Arc<RwLock<ConnectionStatus>>,
    stats: Arc<RwLock<NetworkStats>>,
}

impl ResilientClient {
    /// Create a new resilient client
    pub fn new(config: NetworkConfig) -> AppResult<Self> {
        let mut clients = Vec::new();

        // Create primary client without proxy
        let primary_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent(&config.user_agents[0])
            .build()
            .map_err(|e| AppError::GenericError(format!("Failed to create HTTP client: {}", e)))?;
        clients.push(primary_client);

        // Create clients with different proxies
        for proxy_url in &config.proxy_urls {
            match Proxy::all(proxy_url) {
                Ok(proxy) => {
                    match Client::builder()
                        .timeout(Duration::from_secs(config.timeout_seconds))
                        .proxy(proxy)
                        .user_agent(&config.user_agents[clients.len() % config.user_agents.len()])
                        .build()
                    {
                        Ok(client) => {
                            clients.push(client);
                            info!("Added proxy client: {}", proxy_url);
                        }
                        Err(e) => {
                            warn!("Failed to create proxy client for {}: {}", proxy_url, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Invalid proxy URL {}: {}", proxy_url, e);
                }
            }
        }

        // Create Tor client if enabled
        if config.enable_tor {
            match Proxy::all("socks5://127.0.0.1:9050") {
                Ok(tor_proxy) => {
                    match Client::builder()
                        .timeout(Duration::from_secs(config.timeout_seconds + 30)) // Tor is slower
                        .proxy(tor_proxy)
                        .user_agent(&config.user_agents[clients.len() % config.user_agents.len()])
                        .build()
                    {
                        Ok(client) => {
                            clients.push(client);
                            info!("Added Tor client");
                        }
                        Err(e) => {
                            warn!("Failed to create Tor client: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create Tor proxy: {}", e);
                }
            }
        }

        if clients.is_empty() {
            return Err(AppError::GenericError("No HTTP clients could be created".to_string()));
        }

        Ok(Self {
            config,
            clients,
            current_client_index: 0,
            status: Arc::new(RwLock::new(ConnectionStatus::Connected)),
            stats: Arc::new(RwLock::new(NetworkStats::default())),
        })
    }

    /// Make a resilient HTTP GET request
    pub async fn get(&mut self, url: &str) -> AppResult<reqwest::Response> {
        let mut last_error = None;
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        drop(stats);

        for attempt in 1..=self.config.retry_attempts {
            // Try with current client
            let client = &self.clients[self.current_client_index];

            match client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let mut stats = self.stats.write().await;
                        stats.successful_requests += 1;
                        stats.last_success_time = Some(
                            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                        );

                        if attempt > 1 {
                            stats.retries_used += attempt as u64 - 1;
                        }

                        // Update connection status
                        let mut status = self.status.write().await;
                        *status = if self.current_client_index == 0 {
                            ConnectionStatus::Connected
                        } else if self.current_client_index < self.clients.len() - 1 {
                            ConnectionStatus::ProxyMode
                        } else {
                            ConnectionStatus::TorMode
                        };

                        info!("Successfully made request to {} on attempt {} with client {}", 
                              url, attempt, self.current_client_index);
                        return Ok(response);
                    } else {
                        warn!("HTTP error {} for {}", response.status(), url);
                        last_error = Some(AppError::GenericError(
                            format!("HTTP error: {}", response.status())
                        ));
                    }
                }
                Err(e) => {
                    warn!("Request failed on attempt {} with client {}: {}", 
                          attempt, self.current_client_index, e);
                    last_error = Some(AppError::GenericError(format!("Request failed: {}", e)));

                    // Try next client if available
                    if self.clients.len() > 1 && attempt < self.config.retry_attempts {
                        self.switch_client().await;
                    }
                }
            }

            // Wait before retry (except on last attempt)
            if attempt < self.config.retry_attempts {
                let delay = Duration::from_millis(self.config.retry_delay_ms * attempt as u64);
                debug!("Waiting {}ms before retry", delay.as_millis());
                sleep(delay).await;
            }
        }

        // All attempts failed
        let mut stats = self.stats.write().await;
        stats.failed_requests += 1;
        stats.last_failure_time = Some(
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );

        let mut status = self.status.write().await;
        *status = ConnectionStatus::Disconnected;

        error!("All {} attempts failed for {}", self.config.retry_attempts, url);
        Err(last_error.unwrap_or_else(|| AppError::GenericError("Request failed".to_string())))
    }

    /// Get the underlying HTTP client for custom requests
    pub fn get_client(&self) -> &Client {
        &self.clients[self.current_client_index]
    }

    /// Switch to the next available client
    async fn switch_client(&mut self) {
        let old_index = self.current_client_index;
        self.current_client_index = (self.current_client_index + 1) % self.clients.len();
        
        let mut stats = self.stats.write().await;
        stats.proxy_switches += 1;
        
        info!("Switched from client {} to client {}", old_index, self.current_client_index);
    }

    /// Test connectivity to a list of URLs
    pub async fn test_connectivity(&mut self, test_urls: &[&str]) -> AppResult<HashMap<String, bool>> {
        let mut results = HashMap::new();

        for url in test_urls {
            match self.get(url).await {
                Ok(response) => {
                    results.insert(url.to_string(), response.status().is_success());
                }
                Err(_) => {
                    results.insert(url.to_string(), false);
                }
            }
        }

        Ok(results)
    }

    /// Get current connection status
    pub async fn get_status(&self) -> ConnectionStatus {
        self.status.read().await.clone()
    }

    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkStats {
        self.stats.read().await.clone()
    }

    /// Reset statistics
    pub async fn reset_stats(&mut self) {
        let mut stats = self.stats.write().await;
        *stats = NetworkStats::default();
    }

    /// Get success rate percentage
    pub async fn get_success_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        if stats.total_requests == 0 {
            0.0
        } else {
            (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0
        }
    }
}

/// DNS fallback implementation
pub struct DnsResolver {
    fallback_servers: Vec<String>,
    current_server_index: usize,
}

impl DnsResolver {
    pub fn new(fallback_servers: Vec<String>) -> Self {
        Self {
            fallback_servers,
            current_server_index: 0,
        }
    }

    /// Resolve domain with fallback DNS servers
    pub async fn resolve(&mut self, domain: &str) -> AppResult<Vec<std::net::IpAddr>> {
        // This is a simplified implementation
        // In a real scenario, you would use a DNS library like trust-dns
        
        for (attempt, dns_server) in self.fallback_servers.iter().enumerate() {
            debug!("Attempting to resolve {} using DNS server {}", domain, dns_server);
            
            // Here you would implement actual DNS resolution
            // For now, we'll use the standard resolver as a placeholder
            match tokio::net::lookup_host(format!("{}:80", domain)).await {
                Ok(addresses) => {
                    let ips: Vec<std::net::IpAddr> = addresses.map(|addr| addr.ip()).collect();
                    if !ips.is_empty() {
                        info!("Successfully resolved {} to {:?} using DNS {}", domain, ips, dns_server);
                        return Ok(ips);
                    }
                }
                Err(e) => {
                    warn!("DNS resolution failed for {} using {}: {}", domain, dns_server, e);
                    
                    if attempt < self.fallback_servers.len() - 1 {
                        self.current_server_index = (self.current_server_index + 1) % self.fallback_servers.len();
                    }
                }
            }
        }

        Err(AppError::GenericError(format!("Failed to resolve {} using all DNS servers", domain)))
    }
}

/// Network resilience manager
pub struct NetworkResilienceManager {
    client: ResilientClient,
    dns_resolver: DnsResolver,
    config: NetworkConfig,
}

impl NetworkResilienceManager {
    /// Create a new network resilience manager
    pub fn new(config: NetworkConfig) -> AppResult<Self> {
        let client = ResilientClient::new(config.clone())?;
        let dns_resolver = DnsResolver::new(config.fallback_dns.clone());

        Ok(Self {
            client,
            dns_resolver,
            config,
        })
    }

    /// Perform comprehensive connectivity test
    pub async fn connectivity_test(&mut self) -> AppResult<HashMap<String, bool>> {
        info!("Performing comprehensive connectivity test");

        let test_urls = [
            "https://www.google.com",
            "https://www.cloudflare.com",
            "https://httpbin.org/status/200",
            "https://www.tiktok.com",
            "https://www.facebook.com",
            "https://www.instagram.com",
            "https://www.youtube.com",
        ];

        let results = self.client.test_connectivity(&test_urls).await?;
        
        let success_count = results.values().filter(|&&success| success).count();
        let success_rate = (success_count as f64 / results.len() as f64) * 100.0;

        info!("Connectivity test completed: {:.1}% success rate ({}/{})", 
              success_rate, success_count, results.len());

        Ok(results)
    }

    /// Get the resilient HTTP client
    pub fn get_client(&mut self) -> &mut ResilientClient {
        &mut self.client
    }

    /// Get the DNS resolver
    pub fn get_dns_resolver(&mut self) -> &mut DnsResolver {
        &mut self.dns_resolver
    }

    /// Get network configuration
    pub fn get_config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Generate network resilience report
    pub async fn generate_report(&self) -> HashMap<String, serde_json::Value> {
        let mut report = HashMap::new();
        
        let status = self.client.get_status().await;
        let stats = self.client.get_stats().await;
        let success_rate = self.client.get_success_rate().await;

        report.insert("status".to_string(), serde_json::json!(format!("{:?}", status)));
        report.insert("total_requests".to_string(), serde_json::json!(stats.total_requests));
        report.insert("successful_requests".to_string(), serde_json::json!(stats.successful_requests));
        report.insert("failed_requests".to_string(), serde_json::json!(stats.failed_requests));
        report.insert("success_rate".to_string(), serde_json::json!(success_rate));
        report.insert("retries_used".to_string(), serde_json::json!(stats.retries_used));
        report.insert("proxy_switches".to_string(), serde_json::json!(stats.proxy_switches));
        report.insert("dns_fallbacks".to_string(), serde_json::json!(stats.dns_fallbacks));

        if let Some(last_success) = stats.last_success_time {
            report.insert("last_success_time".to_string(), serde_json::json!(last_success));
        }

        if let Some(last_failure) = stats.last_failure_time {
            report.insert("last_failure_time".to_string(), serde_json::json!(last_failure));
        }

        report.insert("config".to_string(), serde_json::to_value(&self.config).unwrap_or_default());

        report
    }
}

/// Global network resilience manager
static mut NETWORK_MANAGER: Option<NetworkResilienceManager> = None;
static NETWORK_MANAGER_INIT: std::sync::Once = std::sync::Once::new();

/// Get the global network resilience manager
pub fn get_network_manager() -> &'static mut NetworkResilienceManager {
    unsafe {
        NETWORK_MANAGER_INIT.call_once(|| {
            let config = NetworkConfig::default();
            match NetworkResilienceManager::new(config) {
                Ok(manager) => NETWORK_MANAGER = Some(manager),
                Err(e) => panic!("Failed to initialize network manager: {}", e),
            }
        });
        NETWORK_MANAGER.as_mut().unwrap()
    }
}

/// Initialize network resilience
pub async fn init_network_resilience() -> AppResult<()> {
    let manager = get_network_manager();
    
    // Perform initial connectivity test
    let results = manager.connectivity_test().await?;
    let success_count = results.values().filter(|&&success| success).count();
    
    if success_count == 0 {
        warn!("No connectivity detected during initialization");
    } else {
        info!("Network resilience initialized with {}/{} sites reachable", success_count, results.len());
    }
    
    Ok(())
}
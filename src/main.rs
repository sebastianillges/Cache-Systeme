use moka::sync::Cache;
use std::time::{Duration, Instant};
use serde_json::json;
use std::fs;
use std::io;
use serde::de::Error;
use serde_json::{Value, error::Error as SerdeError};
use rand::Rng;
use colored::*;
use plotters::prelude::*;
use plotters::style::Color;

struct DatenQuelleCache {
    cache: Cache<String, String>,
    hits: i32,
    misses: i32,
    total_time: Duration,
}


fn main() {
    let file_path = "telefonbuch.json";

   /* test_cache_random(500, 1000, file_path);
    test_cache_80_20(500, 1000, file_path);
    test_without_cache_random(1000, file_path); */
    let iteration_values = [1, 10, 100, 1000];
    let mut times_cache = Vec::new();
    let mut times_json = Vec::new();
    for &iterations in &iteration_values {
        let (time_cache, time_json) = test_SameValue(100, 1000, iterations, file_path);
        times_cache.push(time_cache);
        times_json.push(time_json);
    }
    //plot_results(&iteration_values, &times_cache, &times_json, "SameValue.png");
    
}



fn test_cache_80_20(cache_size: usize, num_keys: i32, file_path: &str) {
    let start = Instant::now();
    let mut cache = DatenQuelleCache::new(cache_size, Duration::from_secs(30));
    for _i in 0..10000 {
        let rand_num = rand::thread_rng().gen_range(1..10);
        let mut rand_area = 0;
        if rand_num < 3 {
            rand_area = rand::thread_rng().gen_range(1..(num_keys as f32 * 0.8) as i32);
        } else {
            rand_area = rand::thread_rng().gen_range((num_keys as f32 * 0.8) as i32..num_keys);
        }
        let _ = cache.abrufen(&rand_num.to_string(), file_path);
    }
    let total_time = start.elapsed();
    println!("Total time: {:?}", total_time);
}

fn test_SameValue(cache_size: usize, num_keys: i32, iterations: i32, file_path: &str) -> (Duration, Duration) {
   
    let mut cache = DatenQuelleCache::new(cache_size, Duration::from_secs(30));
    cache.abrufen("1", file_path);
    let start = Instant::now();
    for _i in 0..(iterations -1) {
        cache.abrufen("1", file_path);
    }
    let total_time = start.elapsed();

    let startJSON = Instant::now();
    for _i in 0..(iterations -1) {
        wert_aus_json_abrufen("1", file_path);
    }
    let total_timeJSON = startJSON.elapsed();

    println!("Total time: {:?}", total_time);
    println!("Total time: {:?}", total_timeJSON);
    (total_time, total_timeJSON)
}

fn test_without_cache_random(num_keys: i32, file_path: &str) {
    let start = Instant::now();
    for _i in 0..10000 {
        let rand_num = rand::thread_rng().gen_range(1..num_keys).to_string();
        let _ = wert_aus_json_abrufen(&rand_num, file_path);
    }
    let total_time = start.elapsed();
    println!("Total time: {:?}", total_time);
}

fn test_cache_random(cache_size: usize, num_keys: i32, file_path: &str) {
    let mut cache = DatenQuelleCache::new(cache_size, Duration::from_secs(30));
    for _i in 0..10000 {
        let rand_num = rand::thread_rng().gen_range(1..num_keys).to_string();
        let _ = cache.abrufen(&rand_num, file_path);
    }
    println!("Hits: {}", cache.hits);
    println!("Misses: {}", cache.misses);
    println!("Total time: {:?}", cache.total_time);
}

fn wert_aus_json_abrufen(key: &str, file_path: &str) -> Result<i32, SerdeError> {
    // Lesen der Datei
    let data = fs::read_to_string(file_path)
        .map_err(|e| SerdeError::custom(e.to_string()))?;

    // Parsen des JSON-Inhalts
    let json: Value = serde_json::from_str(&data)?;

    // Zugriff auf den Wert für den gegebenen Schlüssel
    match json.get(key) {
        Some(value) => {
            if let Some(number) = value.as_i64() {
                Ok(number as i32)
            } else {
                Err(SerdeError::custom("Wert ist keine Zahl"))
            }
        },
        None => Err(SerdeError::custom("Schlüssel nicht gefunden"))
    }
}


impl DatenQuelleCache {
    fn new(max_capacity: usize, ttl: Duration) -> DatenQuelleCache {
        DatenQuelleCache {
            cache: Cache::builder()
                .max_capacity(max_capacity as u64)//caching policy einstellen
                .time_to_live(ttl)
                .build(),
            hits: 0,
            misses: 0,
            total_time: Duration::new(0, 0),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key)
    }

    fn set(&self, key: String, value: String) {
        self.cache.insert(key, value);
    }
    fn abrufen(&mut self, key: &str, file_path: &str) -> i32 {
        let start = Instant::now();
        match self.get(key) {
            Some(value) => {
                let duration = start.elapsed();
                self.total_time += duration;
                // println!("{}", format!("Cache-Hit[{}]: {} {:?}", key, value, duration).green());
                self.hits += 1;
                value.parse().unwrap_or(0)
            },
            None => {
                match wert_aus_json_abrufen(key, file_path) {
                    Ok(value) => {
                        self.set(key.to_string(), value.to_string());
                        let duration = start.elapsed();
                        self.total_time += duration;
                        // println!("{}", format!("Cache-Miss[{}]: {} {:?}", key, value, duration).red());
                        self.misses += 1;
                        value
                    },
                    Err(e) => {
                        println!("Error: {} ", e);
                        0
                    }
                }
            }
        }
    }
}


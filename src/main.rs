use moka::sync::Cache;
use std::time::{Duration, Instant};
use std::fs;
use std::os::unix::raw::ino_t;
use std::ptr::null;
use std::slice::IterMut;
use serde::de::Error;
use serde_json::{Value, error::Error as SerdeError, json};
use rand::Rng;
use serde_json::Value::Null;


struct Cache_impl {
    cache: Cache<String, String>,
    hits: i32,
    misses: i32,
}


fn main() {
    let mut json_object = json!({"cache": Null, "noncache": Null});
    let file_path = "telefonbuch.json";
    let json_num_keys = 1000;

    let cache_tests: [fn(usize, i32, &str, i32) -> (&str, Duration); 3] = [test_random_cache, test_80_20_cache, test_same_value_cache];
    let non_cache_tests: [fn(i32, &str, i32) -> (&str, Duration); 3] = [test_random, test_80_20, test_same_value];
    let cache_size = [16, 32, 64, 128, 256, 512, 1024];
    let iterations = 1000;


    // generate json for cache tests
    for &size in &cache_size {
        println!("{}", size);
        for &test in &cache_tests {
            let data = test(size, json_num_keys, file_path, iterations);
            let test_name = data.0;
            let result = data.1.as_micros().to_string();

            if json_object["cache"][test_name].is_null() {
                json_object["cache"][test_name] = Value::Array(Vec::new());
            }

            if let Some(array) = json_object["cache"][test_name].as_array_mut() {
                let mut size_result = serde_json::Map::new();
                size_result.insert(size.to_string(), Value::String(result));
                array.push(Value::Object(size_result));
            }
        }
    }

    // generate json for NON cache tests, expected to have the same performance since every access is to the json
    for &test in &non_cache_tests {
        let data = test(json_num_keys, file_path, iterations);
        json_object["noncache"][data.0] = json!(data.1.as_micros().to_string());
    }
    println!("{}", json_object);

    let json_string = serde_json::to_string_pretty(&json_object).expect("Failed to serialize JSON");
    fs::write("output.json", json_string).expect("TODO: panic message");
}

fn test_random_cache(cache_size: usize, num_keys: i32, file_path: &str, iterations: i32) -> (&str, Duration) {
    let name = "random";
    let start = Instant::now();
    let mut cache = Cache_impl::new(cache_size, Duration::from_secs(30));
    for _i in 1..iterations {
        let rand_num = rand::thread_rng().gen_range(1..num_keys).to_string();
        let _ = cache.abrufen(&rand_num, file_path);
    }
    let total_time = start.elapsed();
    //println!("Total time: {:?}", total_time);
    (name, total_time)
}

fn test_80_20_cache(cache_size: usize, num_keys: i32, file_path: &str, iterations: i32) -> (&str, Duration) {
    let name = "80-20";
    let start = Instant::now();
    let mut cache = Cache_impl::new(cache_size, Duration::from_secs(30));
    for _i in 1..iterations {
        let rand_num = rand::thread_rng().gen_range(1..10);
        let mut rand_area = 0;
        if rand_num < 3 {
            rand_area = rand::thread_rng().gen_range(1..(num_keys as f32 * 0.8) as i32);
        } else {
            rand_area = rand::thread_rng().gen_range((num_keys as f32 * 0.8) as i32..num_keys);
        }
        let _ = cache.abrufen(&rand_area.to_string(), file_path);
    }
    let total_time = start.elapsed();
    //println!("Total time: {:?}", total_time);
    (name, total_time)
}

fn test_same_value_cache(cache_size: usize, num_keys: i32, file_path: &str, iterations: i32) -> (&str, Duration) {
    let mut cache = Cache_impl::new(cache_size, Duration::from_secs(30));
    let name = "same_value";
    let start = Instant::now();
    for _i in 1..iterations {
        cache.abrufen("1", file_path);
    }
    let total_time = start.elapsed();
    //println!("Total time: {:?}", total_time);
    (name, total_time)
}

fn test_random(num_keys: i32, file_path: &str, iterations: i32) -> (&str, Duration) {
    let name = "random";
    let start = Instant::now();
    for _i in 1..iterations {
        let rand_num = rand::thread_rng().gen_range(1..num_keys).to_string();
        let _ = get_from_json(&rand_num, file_path);
    }
    let total_time = start.elapsed();
    //println!("Total time: {:?}", total_time);
    (name, total_time)
}

fn test_80_20(num_keys: i32, file_path: &str, iterations: i32) -> (&str, Duration) {
    let name = "80-20";
    let start = Instant::now();
    for _i in 1..iterations {
        let rand_num = rand::thread_rng().gen_range(1..10);
        let mut rand_area = 0;
        if rand_num < 3 {
            rand_area = rand::thread_rng().gen_range(1..(num_keys as f32 * 0.8) as i32);
        } else {
            rand_area = rand::thread_rng().gen_range((num_keys as f32 * 0.8) as i32..num_keys);
        }
        let _ = get_from_json(&rand_area.to_string(), file_path);
    }
    let total_time = start.elapsed();
    //println!("Total time: {:?}", total_time);
    (name, total_time)
}


fn test_same_value(num_keys: i32, file_path: &str, iterations: i32) -> (&str, Duration) {
    let name = "same_value";
    let start = Instant::now();
    for _i in 1..iterations {
        let _ = get_from_json("1", file_path);
    }
    let total_time = start.elapsed();
    //println!("Total time: {:?}", total_time);
    (name, total_time)
}


fn get_from_json(key: &str, file_path: &str) -> Result<i32, SerdeError> {
    let data = fs::read_to_string(file_path).map_err(|e| SerdeError::custom(e.to_string()))?;

    let json: Value = serde_json::from_str(&data)?;

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


impl Cache_impl {
    fn new(max_capacity: usize, ttl: Duration) -> Cache_impl {
        Cache_impl {
            cache: Cache::builder()
                .max_capacity(max_capacity as u64)//caching policy einstellen
                .time_to_live(ttl)
                .build(),
            hits: 0,
            misses: 0,
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
                // println!("{}", format!("Cache-Hit[{}]: {} {:?}", key, value, duration).green());
                self.hits += 1;
                value.parse().unwrap_or(0)
            },
            None => {
                match get_from_json(key, file_path) {
                    Ok(value) => {
                        self.set(key.to_string(), value.to_string());
                        let duration = start.elapsed();
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


use moka::sync::Cache;
use std::time::{Duration, Instant};
use std::fs;
use std::os::unix::raw::ino_t;
use std::ptr::null;
use std::slice::IterMut;
use moka::Policy;
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
    let mut results_json = json!({"cachesize": Null, "numberofaccesses": Null, "hitmiss": Null});
    let data_path = "telefonbuch.json";

    let num_json_keys = 512; // number of existing data, has to be 2^x, current max 1024
    let num_accesses = 4 * num_json_keys;
    let num_measurements = 3;

    // generate options.json so that plot.py knows the parameters of the tests
    let options_json = json!({
        "options": {
            "num_json_keys": num_json_keys,
            "num_accesses": num_accesses,
            "num_measurements": num_measurements
        }
    });
    let json_string = serde_json::to_string_pretty(&options_json).expect("Failed to serialize JSON");
    fs::write("options.json", json_string).expect("TODO: panic message");


    // generate json for cache tests (x: cache size, y: time)
    let tests: [fn(usize, i32, &str, i32) -> (&str, Duration); 2] = [test_random_cache, test_80_20_cache];
    let cache_sizes = generate_doubling_array(16, num_json_keys);
    for &size in &cache_sizes {
        println!("{}", size);

        for &test in &tests {
            for _ in 0..num_measurements {
                let data = test(size, num_json_keys, data_path, num_accesses);
                let test_name = data.0;
                let result = (data.1 / num_accesses as u32).as_micros().to_string();

                if results_json["cachesize"][test_name][size.to_string()].is_null() {
                    results_json["cachesize"][test_name][size.to_string()] = Value::Array(Vec::new());
                }

                if let Some(array) = results_json["cachesize"][test_name][size.to_string()].as_array_mut() {
                    array.push(Value::String(result));
                }
            }
        }
    }

    // generate json for cache vs no cache (x: number of accesses, y = time)
    let tests: [fn(usize, i32, &str, i32) -> (&str, Duration); 2] = [test_same_value_cache, test_same_value];
    let accesses_array = generate_doubling_array(64, num_accesses * 4);
    for &test in &tests {
        for &access in &accesses_array {
            println!("{}", access);
            for i in 0..num_measurements {
                println!("{}", i);
                let data = test((num_json_keys / 2) as usize, 0, data_path, access as i32);
                let test_name = data.0;
                let result = data.1.as_micros().to_string();
                println!("{}", result);

                if results_json["numberofaccesses"][test_name][access.to_string()].is_null() {
                    results_json["numberofaccesses"][test_name][access.to_string()] = Value::Array(Vec::new());
                }

                if let Some(array) = results_json["numberofaccesses"][test_name][access.to_string()].as_array_mut() {
                    array.push(Value::String(result));
                }
            }
        }
    }

    // Hit-Miss-Rate (x: cache size, y = Hit-Miss-Rate)
    let tests: [fn(usize, i32, &str, i32) -> (&str, f32); 2] = [test_random_cache_hit_miss, test_80_20_cache_hit_miss];
    let cache_sizes = generate_doubling_array(16, num_json_keys);
    for &size in &cache_sizes {
        println!("{}", size);
        for &test in &tests {
            let data = test(size, num_json_keys, data_path, num_accesses);
            let test_name = data.0;
            let result = data.1;

            if results_json["hitmiss"][test_name][size.to_string()].is_null() {
                results_json["hitmiss"][test_name][size.to_string()] = Value::from(result);
            }
        }
    }


    println!("{}", results_json);


    let json_string = serde_json::to_string_pretty(&results_json).expect("Failed to serialize JSON");
    fs::write("output.json", json_string).expect("TODO: panic message");
}

fn test_random_cache(cache_size: usize, num_keys: i32, file_path: &str, iterations: i32) -> (&str, Duration) {
    let name = "random_cache";
    let start = Instant::now();
    let mut cache = Cache_impl::new(cache_size, Duration::from_secs(30));
    for _ in 1..iterations {
        let rand_num = rand::thread_rng().gen_range(1..num_keys).to_string();
        let _ = cache.search(&rand_num, file_path);
    }
    let total_time = start.elapsed();
    (name, total_time)
}

fn test_80_20_cache(cache_size: usize, num_keys: i32, file_path: &str, iterations: i32) -> (&str, Duration) {
    let name = "80-20_cache";
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
        let _ = cache.search(&rand_area.to_string(), file_path);
    }
    let total_time = start.elapsed();
    (name, total_time)
}

fn test_random_cache_hit_miss(cache_size: usize, num_keys: i32, file_path: &str, iterations: i32) -> (&str, f32) {
    let name = "test_random_cache_hit_miss";
    let mut cache = Cache_impl::new(cache_size, Duration::from_secs(30));
    for _ in 1..iterations {
        let rand_num = rand::thread_rng().gen_range(1..num_keys).to_string();
        let _ = cache.search(&rand_num, file_path);
    }
    println!("{}:{}", cache.hits, cache.misses);
    (name, (cache.hits / cache.misses) as f32)
}

fn test_80_20_cache_hit_miss(cache_size: usize, num_keys: i32, file_path: &str, iterations: i32) -> (&str, f32) {
    let name = "80-20-test_80_20_cache_hit_miss";
    let mut cache = Cache_impl::new(cache_size, Duration::from_secs(30));
    for _i in 1..iterations {
        let rand_num = rand::thread_rng().gen_range(1..10);
        let mut rand_area = 0;
        if rand_num < 3 {
            rand_area = rand::thread_rng().gen_range(1..(num_keys as f32 * 0.8) as i32);
        } else {
            rand_area = rand::thread_rng().gen_range((num_keys as f32 * 0.8) as i32..num_keys);
        }
        let _ = cache.search(&rand_area.to_string(), file_path);
    }
    println!("{}:{}", cache.hits, cache.misses);
    (name, (cache.hits / cache.misses) as f32)
}

fn test_same_value_cache(cache_size: usize, num_keys: i32, file_path: &str, iterations: i32) -> (&str, Duration) {
    let mut cache = Cache_impl::new(cache_size, Duration::from_secs(30));
    let name = "same_value_cache";
    cache.search("1", file_path);
    let start = Instant::now();
    for _i in 1..iterations {
        cache.search("1", file_path);
    }
    let total_time = start.elapsed();
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
    (name, total_time)
}


fn test_same_value(cache_size: usize, num_keys: i32, file_path: &str, iterations: i32) -> (&str, Duration) {
    let name = "same_value";
    let start = Instant::now();
    for _i in 1..iterations {
        let _ = get_from_json("1", file_path);
    }
    let total_time = start.elapsed();
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
                .max_capacity(max_capacity as u64)
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
    fn search(&mut self, key: &str, file_path: &str) -> i32 {
        match self.get(key) {
            Some(value) => {
                self.hits += 1;
                value.parse().unwrap_or(0)
            },
            None => {
                match get_from_json(key, file_path) {
                    Ok(value) => {
                        self.set(key.to_string(), value.to_string());
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

fn generate_doubling_array(min_value: usize, max_value: i32) -> Vec<usize> {
    let mut array = Vec::new();
    let mut current = min_value;
    let max_value = max_value as usize;

    while current <= max_value {
        array.push(current);
        current *= 2;
    }
    array
}
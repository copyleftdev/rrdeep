use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;
use std::time::Instant;

const N: usize = 64;
const W: usize = 7;
const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64(x: u8) -> char {
    B64[(x % 64) as usize] as char
}

#[derive(Clone)]
struct Rolling {
    w: [u8; W],
    h1: u64,
    h2: u64,
    n: usize,
    i: usize,
}

impl Rolling {
    fn new() -> Self {
        Rolling {
            w: [0; W],
            h1: 0,
            h2: 0,
            n: 0,
            i: 0,
        }
    }
    fn roll(&mut self, b: u8) {
        if self.n < W {
            self.n += 1;
        } else {
            let old = self.w[self.i];
            self.h1 = self.h1.wrapping_sub(old as u64);
            self.h2 = self.h2
                .wrapping_sub(self.n as u64 * old as u64)
                .wrapping_add(self.h1);
        }
        self.w[self.i] = b;
        self.i = (self.i + 1) % W;
        self.h1 = self.h1.wrapping_add(b as u64);
        self.h2 = self.h2.wrapping_add(self.h1);
    }
    fn digest(&self) -> u64 {
        self.h2
            .wrapping_add(self.h1.wrapping_mul(self.n as u64))
    }
}

#[derive(Clone)]
struct Simple {
    h: u64,
}

impl Simple {
    fn new() -> Self {
        Simple { h: 0 }
    }
    fn update(&mut self, b: u8) {
        self.h = self.h.wrapping_mul(16777619);
        self.h = self.h.wrapping_add(b as u64);
    }
    fn c(&self) -> char {
        b64((self.h & 0xff) as u8)
    }
}

fn derive_bs(size: usize) -> u64 {
    let mut b = 1u64;
    while (b as usize) * N < size {
        b <<= 1;
    }
    b
}


pub struct PerfMetrics {
    pub total_bytes: usize,
    pub duration_s: f64,
    pub speed_mbps: f64,
}


fn producer_thread(path: PathBuf, tx: SyncSender<Vec<u8>>) {
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: could not open file {}: {}", path.display(), e);
            let _ = tx.send(Vec::new());
            return;
        }
    };

    let mut reader = BufReader::new(file);
    loop {
        let mut buf = vec![0u8; 64 * 1024];
        match reader.read(&mut buf) {
            Ok(0) => {
                
                let _ = tx.send(Vec::new());
                break;
            }
            Ok(n) => {
                buf.truncate(n);
                if tx.send(buf).is_err() {
                    
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", path.display(), e);
                let _ = tx.send(Vec::new());
                break;
            }
        }
    }
}


pub fn compute_rrdeep_from_path_concurrent(
    path: &PathBuf
) -> std::io::Result<(String, PerfMetrics)> {
    let meta = std::fs::metadata(path)?;
    let size = meta.len() as usize;
    if size == 0 {
        return Ok(("3:3:1".to_string(), PerfMetrics {
            total_bytes: 0,
            duration_s: 0.0,
            speed_mbps: 0.0,
        }));
    }

    let b1 = derive_bs(size);
    let b2 = if b1 > 1 { b1 / 2 } else { 1 };

    let (tx, rx) = sync_channel::<Vec<u8>>(4);
    let path_clone = path.clone();

    // Start timing now
    let start = Instant::now();
    let mut total_bytes = 0usize;

    thread::spawn(move || {
        producer_thread(path_clone, tx);
    });

    let mut r1 = Rolling::new();
    let mut s1 = Simple::new();
    let mut sig1_chars = Vec::new();

    let mut r2 = Rolling::new();
    let mut s2 = Simple::new();
    let mut sig2_chars = Vec::new();

    for chunk in rx {
        if chunk.is_empty() {
            break;
        }
        total_bytes += chunk.len();
        for &b in &chunk {
            r1.roll(b);
            s1.update(b);
            if (r1.digest() % b1) == (b1 - 1) {
                sig1_chars.push(s1.c());
                s1 = Simple::new();
            }
            r2.roll(b);
            s2.update(b);
            if (r2.digest() % b2) == (b2 - 1) {
                sig2_chars.push(s2.c());
                s2 = Simple::new();
            }
        }
    }

    sig1_chars.push(s1.c());
    if sig1_chars.len() > N {
        sig1_chars.truncate(N);
    }
    let s1_str: String = sig1_chars.into_iter().collect();

    sig2_chars.push(s2.c());
    if sig2_chars.len() > N {
        sig2_chars.truncate(N);
    }
    let s2_str: String = sig2_chars.into_iter().collect();

    
    let elapsed = start.elapsed().as_secs_f64();
    let speed_mbps = if elapsed > 0.0 {
        (total_bytes as f64) / (1024.0 * 1024.0 * elapsed)
    } else {
        0.0
    };
    let perf = PerfMetrics {
        total_bytes,
        duration_s: elapsed,
        speed_mbps,
    };

    Ok((format!("{}:{}:{}", s1_str, s2_str, b1), perf))
}

pub fn compare_rrdeep(a: &str, b: &str) -> i32 {
    let (a1, a2, ab) = match parse(a) {
        Some(x) => x,
        None => return 0,
    };
    let (b1, b2, bb) = match parse(b) {
        Some(x) => x,
        None => return 0,
    };
    if ab == 0 || bb == 0 {
        return 0;
    }
    if ab > bb * 2 || bb > ab * 2 {
        return 0;
    }

    let d1 = edit_dist(&a1, &b1);
    let max1 = a1.len().max(b1.len()).max(1);
    let s1 = (100 * (max1 - d1)) / max1;

    let d2 = edit_dist(&a2, &b2);
    let max2 = a2.len().max(b2.len()).max(1);
    let s2 = (100 * (max2 - d2)) / max2;

    let mut score = if ab == bb {
        (s1 + s2) / 2
    } else {
        let mut tmp = if s1 > s2 { s1 } else { s2 };
        if tmp > 0 {
            tmp -= 1;
        }
        tmp
    };

    let prefix_len = common_prefix(&a1, &b1);
    if prefix_len >= 2 {
        score += 1;
    }
    if prefix_len >= 4 {
        score += 1;
    }
    if score > 100 {
        score = 100;
    }
    score as i32
}

fn parse(s: &str) -> Option<(String, String, u64)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let x1 = parts[0].to_string();
    let x2 = parts[1].to_string();
    let x3 = parts[2].parse::<u64>().ok()?;
    Some((x1, x2, x3))
}

fn edit_dist(a: &str, b: &str) -> usize {
    let aa = a.as_bytes();
    let bb = b.as_bytes();
    let la = aa.len();
    let lb = bb.len();
    let mut dp = vec![0; (la + 1) * (lb + 1)];
    for i in 0..=la {
        dp[i * (lb + 1)] = i;
    }
    for j in 0..=lb {
        dp[j] = j;
    }
    for i in 1..=la {
        for j in 1..=lb {
            let cost = if aa[i - 1] == bb[j - 1] { 0 } else { 1 };
            let up = dp[(i - 1) * (lb + 1) + j] + 1;
            let left = dp[i * (lb + 1) + j - 1] + 1;
            let diag = dp[(i - 1) * (lb + 1) + (j - 1)] + cost;
            dp[i * (lb + 1) + j] = up.min(left.min(diag));
        }
    }
    dp[la * (lb + 1) + lb]
}

fn common_prefix(a: &str, b: &str) -> usize {
    let mut n = 0;
    for (x, y) in a.chars().zip(b.chars()) {
        if x == y {
            n += 1;
        } else {
            break;
        }
    }
    n
}

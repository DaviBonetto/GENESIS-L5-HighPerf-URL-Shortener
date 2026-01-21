# GENESIS // High-Performance URL Shortener

![Status](https://img.shields.io/badge/Status-Development-blue?style=flat-square)
![Layer](https://img.shields.io/badge/Layer-L5_Product-purple?style=flat-square)
![Stack](https://img.shields.io/badge/Core-Rust-orange?style=flat-square)

> **Part of the Titan Protocol Initiative.**
> Engineered for O(1) lookups using probabilistic data structures (Bloom Filters) and zero-copy networking.

## 🛠️ Tech Stack
- **Engine:** Rust (Actix-web + Tokio)
- **L1 Cache:** Bloom Filter (Probabilistic)
- **Storage:** Redis (Hot) + Postgres (Cold)

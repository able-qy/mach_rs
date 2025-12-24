# MachRS ⚡️

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

**MachRS** 是一个基于 Rust 编写的高性能、内存级数字货币撮合引擎（Matching Engine）。

它旨在提供**极致的性能**与**金融级的安全性**，实现了标准的价格优先/时间优先（Price/Time Priority）撮合算法，并包含完整的账户资金冻结与结算闭环逻辑。

## ✨ 核心特性

* **高性能撮合**：基于 `BTreeMap` 和 `VecDeque` 的订单簿设计，支持高并发撮合。
* **资金安全**：内置双重记账逻辑（Available/Frozen），杜绝超额消费。
* **完整闭环**：支持 挂单 -> 冻结 -> 撮合 -> 结算 -> 解冻 的完整生命周期。
* **模块化架构**：核心引擎（Engine）、账户系统（Account）与数据定义（Types）解耦，易于扩展。
* **工程化测试**：包含 Criterion 基准测试（Benchmark）与覆盖边界情况的集成测试。

## 🏗 架构设计

系统主要由三个核心模块组成：

```text
+----------------+          +-----------------+          +----------------+
|                |  下单    |                 |  撮合    |                |
| AccountManager | -------> |    Main Loop    | -------> |    OrderBook   |
| (资金管理)      | <------- | (业务编排/结算)  | <------- | (撮合核心)      |
|                |  结算    |                 |  成交事件 |                |
+----------------+          +-----------------+          +----------------+
      ^                            |                           ^
      |                            |                           |
      +-------[ 依赖 ]-------- Types (Order/Asset) --------[ 依赖 ]--+

```

* **src/account.rs**: 管理用户资产，处理充值、冻结、解冻、转账。
* **src/engine.rs**: 维护买卖盘（OrderBook），执行撮合算法，生成成交事件（TradeEvent）。
* **src/types.rs**: 定义通用的金融数据结构（Order, Trade, Asset）。

## 🚀 快速开始

### 前置要求

确保你安装了 Rust 工具链 (Cargo)。

### 1. 运行演示

模拟完整的下单与结算流程：

```bash
cargo run

```

### 2. 运行测试

包含部分成交、完全成交、撤单资金回退等多种边界测试：

```bash
cargo test

```

### 3. 性能基准测试

使用 Criterion 测试纯撮合引擎的吞吐量（TPS）：

```bash
cargo bench

```

## 📖 代码示例

```rust
use mach_rs::{AccountManager, OrderBook, Asset, Order, OrderSide};

fn main() {
    let mut account = AccountManager::new();
    let mut book = OrderBook::new();
    let btc = Asset::from("BTC");
    let usdt = Asset::from("USDT");

    // 1. 充值
    account.deposit(1, btc, 10).unwrap();       // Maker
    account.deposit(2, usdt, 20000).unwrap();   // Taker

    // 2. 挂单 (Maker)
    account.try_freeze(1, btc, 1).unwrap();
    book.match_order(Order { 
        id: 101, user_id: 1, price: 20000, quantity: 1, side: OrderSide::Ask 
    });

    // 3. 吃单 (Taker)
    account.try_freeze(2, usdt, 20000).unwrap();
    let trades = book.match_order(Order { 
        id: 102, user_id: 2, price: 20000, quantity: 1, side: OrderSide::Bid 
    });

    // 4. 结算
    for trade in trades {
        // 处理 Maker 和 Taker 的资金划转...
        println!("成交: {:?}", trade);
    }
}

```

## 🛠 开发路线图 (Roadmap)

* [x] **Core**: 基础限价单撮合 (Limit Order Matching)
* [x] **Account**: 资产冻结与解冻机制
* [x] **Test**: 集成测试与基准测试环境
* [x] **Feature**: 撤单功能 (Cancel Order) & 索引构建
* [ ] **Safety**: 引入 `rust_decimal` 替代 u64 解决精度问题
* [ ] **IO**: 引入 `serde` 实现数据序列化与持久化
* [ ] **Error**: 使用 `thiserror` 规范化错误处理
* [ ] **Arch**: 升级为基于 Channel 的异步 Actor 模型

## 📄 许可证

MIT License
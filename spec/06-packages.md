# Packages Specification

## Overview

This document describes the main Rust packages (crates) used in Ferrous CI/CD. It includes the purpose, usage, and version information for each package.

## Core Dependencies

### Async Runtime

#### tokio

**Version**: 1.48  
**Purpose**: Async runtime and I/O processing

Tokio is Rust's async runtime that provides the following features:

- **Async I/O**: Network and file system operations
- **Task Scheduling**: Concurrent task execution
- **Timers**: Async timers and delayed execution
- **Channels**: Inter-task communication

**Usage Locations**:
- HTTP server execution
- Async database connection processing
- Async build execution management
- Event processing

**Configuration**:
```toml
tokio = { version = "1.48", features = ["full"] }
```

The `full` feature enables all Tokio functionality.

#### async-trait

**Version**: 0.1  
**Purpose**: Async method definitions in traits

Since Rust traits cannot directly define `async fn`, this macro is used.

**Usage Locations**:
- Repository interfaces
- Event publishers/handlers
- Domain services

**Example**:
```rust
#[async_trait]
trait Repository {
    async fn save(&self, entity: &Entity) -> Result<()>;
}
```

### Error Handling

#### anyhow

**Version**: 1.0  
**Purpose**: Simplifying error handling

Uses `anyhow::Result<T>` to return errors without explicitly specifying error types.

**Usage Locations**:
- Configuration file loading
- Application initialization
- Utility functions

#### thiserror

**Version**: 2.0  
**Purpose**: Custom error type definitions

Provides macros that automatically implement the `Error` trait.

**Usage Locations**:
- Domain error definitions
- Application error definitions
- Error hierarchy structures

**Example**:
```rust
#[derive(Debug, thiserror::Error)]
enum DomainError {
    #[error("Validation failed: {0}")]
    Validation(String),
}
```

### Serialization

#### serde

**Version**: 1.0  
**Purpose**: Data serialization/deserialization

**Usage Locations**:
- Entity JSON/YAML conversion
- API requests/responses
- Configuration file loading
- Data exchange with database

**Configuration**:
```toml
serde = { version = "1.0", features = ["derive"] }
```

#### serde_json

**Version**: 1.0  
**Purpose**: JSON format serialization

**Usage Locations**:
- APIレスポンスの生成
- 設定ファイルの読み込み
- データの永続化

#### serde_yaml

**Version**: 0.9  
**Purpose**: YAML format serialization

**Usage Locations**:
- 設定ファイル（`config.yaml`）の読み込み
- パイプライン設定ファイル（`.ferrous-ci.yaml`）の読み込み

### 識別子生成

#### uuid

**Version**: 1.6  
**Purpose**: UUID (Universally Unique Identifier) generation

**Usage Locations**:
- エンティティIDの生成（PipelineId、BuildId等）
- トークンの生成
- 一意な識別子が必要な場面

**Configuration**:
```toml
uuid = { version = "1.6", features = ["v4", "serde"] }
```

- `v4`: UUID v4（ランダム）の生成
- `serde`: Serdeによるシリアライゼーションサポート

### 日時処理

#### chrono

**Version**: 0.4  
**Purpose**: Date/time and timezone processing

**Usage Locations**:
- エンティティのタイムスタンプ（created_at、updated_at）
- ビルドの実行時間計算
- スケジュールトリガーの処理

**Configuration**:
```toml
chrono = { version = "0.4", features = ["serde"] }
```

## Webフレームワーク

### axum

**バージョン**: 0.8  
**用途**: HTTPサーバーとREST APIの実装

AxumはTokio上で動作するモダンなWebフレームワークです。

**特徴**:
- 型安全なルーティング
- ミドルウェアサポート
- リクエスト/レスポンスの自動変換
- WebSocketサポート

**使用箇所**:
- REST APIエンドポイントの実装
- リクエストハンドラー
- ミドルウェア（認証、CORS等）

**Related Packages**:
- `tower`: ミドルウェアスタック
- `tower-http`: HTTP固有のミドルウェア
- `hyper`: 低レベルHTTP実装

### tower

**Version**: 0.5  
**Purpose**: Middleware and service abstraction

**Usage Locations**:
- リクエストの前処理・後処理
- エラーハンドリング
- タイムアウト処理

### tower-http

**Version**: 0.6  
**Purpose**: HTTP-specific middleware

**Features**:
- CORS設定
- リクエストトレーシング
- 圧縮
- Security headers

**Configuration**:
```toml
tower-http = { version = "0.6", features = ["cors", "trace"] }
```

### hyper

**Version**: 1.0  
**Purpose**: Low-level HTTP implementation

Axumの基盤となるHTTPライブラリです。直接使用することは少ないですが、Axumが内部的に使用します。

## データベース

### sqlx

**Version**: 0.8  
**Purpose**: Async SQL database access

**Features**:
- コンパイル時SQL検証
- 型安全なクエリ
- 接続プーリング
- マイグレーションサポート

**使用箇所**:
- リポジトリ実装
- データベースマイグレーション
- クエリ実行

**Configuration**:
```toml
sqlx = { 
    version = "0.8", 
    features = [
        "runtime-tokio-rustls",
        "postgres",
        "sqlite",
        "migrate",
        "uuid",
        "chrono"
    ] 
}
```

**Feature Descriptions**:
- `runtime-tokio-rustls`: TokioランタイムとTLSサポート
- `postgres`: PostgreSQLサポート
- `sqlite`: SQLiteサポート
- `migrate`: マイグレーション機能
- `uuid`: UUID型のサポート
- `chrono`: 日時型のサポート

### diesel

**Version**: 2.3  
**Purpose**: Type-safe SQL query builder (optional)

**Usage Locations**:
- 複雑なクエリの構築
- 型安全なクエリが必要な場合

**Configuration**:
```toml
diesel = { 
    version = "2.3", 
    features = ["postgres", "sqlite", "chrono", "uuid"] 
}
```

### diesel_migrations

**Version**: 2.1  
**Purpose**: Diesel migration management

## ログとトレーシング

### tracing

**Version**: 0.1  
**Purpose**: Structured logging and tracing

**Features**:
- 構造化ログ
- 分散トレーシング
- パフォーマンス計測
- スパン（Span）によるコンテキスト追跡

**使用箇所**:
- アプリケーションログ
- リクエストトレーシング
- パフォーマンス計測

### tracing-subscriber

**Version**: 0.3  
**Purpose**: Tracing data collection and output

**Features**:
- 環境変数によるログレベル制御
- JSON形式のログ出力
- ファイルへのログ出力

**Configuration**:
```toml
tracing-subscriber = { 
    version = "0.3", 
    features = ["env-filter", "json"] 
}
```

## 設定管理

### config

**Version**: 0.15  
**Purpose**: Configuration file and environment variable loading

**Features**:
- YAML、JSON、TOML形式のサポート
- 環境変数による上書き
- 設定の階層構造
- デフォルト値の設定

**使用箇所**:
- `config.yaml`の読み込み
- 環境変数からの設定読み込み
- 設定の検証

### dotenv

**Version**: 0.15  
**Purpose**: Environment variable loading from `.env` files

**Usage Locations**:
- 開発環境での設定管理
- ローカル開発時の環境変数管理

## Git操作

### git2

**Version**: 0.20  
**Purpose**: Git repository operations

**Features**:
- リポジトリのクローン
- コミット情報の取得
- ブランチ操作
- ファイル差分の取得

**使用箇所**:
- プロジェクトリポジトリのクローン
- ビルド時のコード取得
- コミット情報の取得

## Authentication & Security

### jsonwebtoken

**Version**: 10.2  
**Purpose**: JWT (JSON Web Token) generation and verification

**Usage Locations**:
- ユーザー認証
- API認証
- トークンの生成と検証

### bcrypt

**Version**: 0.17  
**Purpose**: Password hashing

**Usage Locations**:
- ユーザーパスワードのハッシュ化
- パスワード検証

## メッセージング（将来実装）

### redis

**Version**: 0.32  
**Purpose**: Redis client (planned)

**Planned Usage**:
- キャッシュ
- セッション管理
- メッセージキュー

**Configuration**:
```toml
redis = { 
    version = "0.32", 
    features = ["tokio-comp", "connection-manager"] 
}
```

### lapin

**Version**: 3.7  
**Purpose**: RabbitMQ client (planned)

**Planned Usage**:
- イベントキュー
- タスクキュー
- メッセージブローカー

## CLI

### clap

**Version**: 4.5  
**Purpose**: Command-line argument parsing

**Features**:
- 型安全な引数解析
- 自動ヘルプ生成
- 環境変数からの設定読み込み
- サブコマンドサポート

**使用箇所**:
- CLIコマンドの実装
- コマンド引数の解析

**Configuration**:
```toml
clap = { version = "4.5", features = ["derive", "env"] }
```

### colored

**Version**: 3.0  
**Purpose**: Terminal output coloring

**Usage Locations**:
- CLI出力の色付け
- エラーメッセージの強調
- 成功メッセージの表示

## 監視・メトリクス

### prometheus

**Version**: 0.14  
**Purpose**: Prometheus metrics collection (planned)

**Planned Usage**:
- ビルド数のメトリクス
- エージェントの状態メトリクス
- パフォーマンスメトリクス

**Configuration**:
```toml
prometheus = { version = "0.14", features = ["process"] }
```

### opentelemetry

**Version**: 0.31  
**Purpose**: Distributed tracing (planned)

**Planned Usage**:
- リクエストトレーシング
- ビルド実行のトレーシング
- パフォーマンス分析

### opentelemetry-jaeger

**Version**: 0.22  
**Purpose**: Sending tracing data to Jaeger (planned)

## テスト

### mockall

**Version**: 0.13  
**Purpose**: Mock object generation

**Usage Locations**:
- ユニットテストでのモック作成
- リポジトリのモック
- サービスのモック

### wiremock

**Version**: 0.6  
**Purpose**: HTTP mock server

**Usage Locations**:
- 統合テストでの外部APIモック
- Webhookテスト

## Development Dependencies

### criterion

**Version**: 0.7  
**Purpose**: Benchmark testing

**Usage Locations**:
- パフォーマンステスト
- ベンチマーク実行

### proptest

**Version**: 1.9  
**Purpose**: Property-based testing

**Usage Locations**:
- ドメインロジックのプロパティテスト
- エッジケースの検出

### quickcheck

**Version**: 1.0  
**Purpose**: QuickCheck testing

**Usage Locations**:
- ランダム入力によるテスト
- プロパティテスト

### rstest

**バージョン**: 0.26  
**用途**: パラメータ化テスト

**使用箇所**:
- 複数のテストケースを効率的に記述
- テストフィクスチャの管理

### test-case

**バージョン**: 3.1  
**用途**: テストケースのマクロ

**使用箇所**:
- テストケースの簡潔な記述

### pretty_assertions

**バージョン**: 1.4  
**用途**: 読みやすいアサーション出力

**使用箇所**:
- テスト失敗時の出力改善
- 差分の見やすい表示

### insta

**バージョン**: 1.34  
**用途**: スナップショットテスト

**使用箇所**:
- JSON出力のスナップショットテスト
- APIレスポンスの検証

### tempfile

**バージョン**: 3.8  
**用途**: 一時ファイルの作成

**使用箇所**:
- テストでの一時ファイル作成
- 一時ディレクトリの作成

### serial_test

**バージョン**: 3.0  
**用途**: テストの順次実行

**使用箇所**:
- データベーステストでの競合回避
- リソース共有が必要なテスト

## システム情報

### num_cpus

**バージョン**: 1.16  
**用途**: CPUコア数の取得

**使用箇所**:
- デフォルトワーカー数の決定
- リソース管理

## Package Selection Rationale

### 非同期処理

- **Tokio**: Rustの非同期処理のデファクトスタンダード
- **async-trait**: トレイトでの非同期メソッド定義に必要

### Webフレームワーク

- **Axum**: モダンで型安全、Tokioとの統合が良好
- **Tower**: ミドルウェアの標準的な抽象化

### データベース

- **SQLx**: コンパイル時SQL検証、型安全性
- **Diesel**: オプションとして、複雑なクエリが必要な場合に使用

### ログ・トレーシング

- **Tracing**: 構造化ログとトレーシングの標準
- **Tracing-subscriber**: 柔軟なログ出力設定

### 認証

- **jsonwebtoken**: JWTの標準的な実装
- **bcrypt**: パスワードハッシュ化の標準

## Security Considerations

### Dependency Management

- **cargo audit**: Security vulnerability detection
- **cargo outdated**: Dependency update checking

### Recommended Practices

1. Regular dependency updates
2. Security audits
3. Principle of least privilege (only enable necessary features)

## Package Update Policy

- **Major Versions**: Evaluate carefully before updating
- **Minor Versions**: Check compatibility before updating
- **Patch Versions**: Consider automatic updates

## Future Additions

The following packages are being considered for addition:

- **reqwest**: HTTPクライアント（外部API呼び出し）
- **tera**: テンプレートエンジン（通知メール等）
- **r2d2**: 接続プール管理
- **actix-web**: Axumの代替として検討（現在はAxumを使用）

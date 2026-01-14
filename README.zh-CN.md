# auto_field_macros

## 项目概述

`auto_field_macros` 是一个基于 Rust 语言开发的过程宏库，专为 `auto_field_trait` 库设计，提供了自动字段处理的宏支持。该库通过宏定义，简化了开发者在使用 `auto_field_trait` 库时的代码编写，自动生成所需的 trait 实现和字段处理逻辑。

### 功能特性

- **自动生成 ActiveModelBehavior 实现**：自动处理字段填充逻辑
- **自动生成 QueryExtensions 实现**：提供便捷的查询方法
- **自动生成 CustomizationExt 实现**：支持软删除和批量操作
- **灵活的配置选项**：通过属性配置选择需要的功能
- **支持多种字段类型**：
  - 雪花ID生成
  - 时间戳管理
  - 审计日志
  - 租户支持
  - 版本控制
  - 软删除

### 技术架构

- **语言**：Rust
- **核心依赖**：
  - `proc-macro2`：过程宏支持
  - `quote`：Rust 代码生成
  - `syn`：Rust 语法分析

## 安装与配置

### 安装

在 `Cargo.toml` 文件中添加依赖：

```toml
dependencies =
    auto_field_trait = { version = "0.1.3", git = "https://github.com/tttq/auto_field_trait.git", features = ["postgres", "with-web"] }
    sea-orm = "0.12"

proc-macro-dependencies =
    auto_field_macros = { version = "0.1.3", git = "https://github.com/tttq/auto_field_macros.git" }
```

### 配置

`auto_field_macros` 库不需要额外的配置文件，只需要在使用时通过属性进行配置即可。

## 使用指南

### 基本使用

1. **导入依赖**：

```rust
use auto_field_macros::AutoField;
use auto_field_trait::QueryExtensions;
use auto_field_trait::CustomizationExt;
```

2. **定义实体并使用宏**：

```rust
use sea_orm::entity::prelude::*;
use auto_field_macros::AutoField;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, AutoField)]
#[sea_orm(table_name = "users")]
#[auto_field(snowflake_id, timestamps, audit, tenant, version, soft_delete)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    pub email: String,
    pub create_time: Option<DateTime<Utc>>,
    pub update_time: Option<DateTime<Utc>>,
    pub create_by: Option<String>,
    pub create_id: Option<String>,
    pub update_by: Option<String>,
    pub update_id: Option<String>,
    pub tenant_id: Option<String>,
    pub tenant_name: Option<String>,
    pub version: Option<i32>,
    pub delete_flag: Option<i32>,
}
```

3. **宏配置选项**：

`auto_field` 宏支持以下配置选项：

- `snowflake_id`：启用雪花ID自动生成
- `timestamps`：启用时间戳自动填充
- `audit`：启用审计字段自动填充
- `tenant`：启用租户字段自动填充
- `version`：启用版本号自动管理
- `soft_delete`：启用软删除功能

可以通过以下方式配置：

```rust
// 方式1：全量启用
#[auto_field(snowflake_id, timestamps, audit, tenant, version, soft_delete)]

// 方式2：部分启用
#[auto_field(timestamps, audit)]

// 方式3：使用key-value形式
#[auto_field(snowflake_id = true, timestamps = true)]
```

4. **使用自动生成的功能**：

```rust
// 使用QueryExtensions
let users = User::find_not_deleted().all(db).await?;
let users = User::find_by_tenant_id("tenant_123").all(db).await?;

// 使用CustomizationExt
User::soft_delete(db, "user_789").await?;
User::soft_delete_many(db, &["user_101", "user_102"]).await?;

// 使用batch_update
let update_many = User::batch_update()
    .col_expr(User::Column::Name, Expr::value("new_name"))
    .filter(User::Column::Id.eq("user_123"))
    .exec(db)
    .await?;

// 使用batch_insert_many
let users = vec![
    UserActiveModel {
        name: ActiveValue::Set("user_1".to_string()),
        email: ActiveValue::Set("user_1@example.com".to_string()),
        ..Default::default()
    },
    UserActiveModel {
        name: ActiveValue::Set("user_2".to_string()),
        email: ActiveValue::Set("user_2@example.com".to_string()),
        ..Default::default()
    },
];

let insert_result = User::batch_insert_many(users)
    .exec(db)
    .await?;
```

### 高级用法

1. **条件配置**：

可以根据需要选择启用的功能，例如只启用时间戳和软删除：

```rust
#[auto_field(timestamps, soft_delete)]
```

2. **验证配置**：

宏会自动验证配置的有效性，例如如果启用了 `audit` 功能，则必须同时启用 `timestamps` 功能，否则会编译错误。

3. **自定义字段名**：

目前 `auto_field_macros` 库使用固定的字段名，例如：
- `create_time`：创建时间
- `update_time`：更新时间
- `create_by`：创建人
- `create_id`：创建人ID
- `update_by`：更新人
- `update_id`：更新人ID
- `tenant_id`：租户ID
- `tenant_name`：租户名称
- `version`：版本号
- `delete_flag`：删除标记

如果需要自定义字段名，可以通过修改 `auto_field_trait` 库的源代码实现。

## 注意事项

### 环境要求

- **Rust 版本**：1.65.0 及以上
- **SeaORM 版本**：0.12.x
- **auto_field_trait 版本**：与 `auto_field_macros` 版本匹配

### 限制条件

1. 目前仅支持 SeaORM 框架
2. 只支持固定的字段名，不支持自定义字段名
3. 必须与 `auto_field_trait` 库配合使用
4. 某些功能有依赖关系，例如 `audit` 功能依赖 `timestamps` 功能

### 常见问题

1. **问题**：编译错误，提示缺少依赖
   **解决方案**：确保已正确安装所有依赖，包括 `auto_field_trait` 和 `sea-orm`

2. **问题**：编译错误，提示配置无效
   **解决方案**：检查宏配置是否正确，例如 `audit` 功能必须同时启用 `timestamps` 功能

3. **问题**：自动字段没有被正确填充
   **解决方案**：确保已正确注册 `HookedSeaOrmPlugin` 插件和上下文获取器

## 项目目录结构

```
auto_field_macros/
├── src/
│   └── lib.rs                # 库入口文件，包含宏定义
├── Cargo.toml                # 依赖配置
└── README.md                 # 项目文档
```

### 文件用途说明

| 文件/文件夹 | 用途 |
| --- | --- |
| `src/lib.rs` | 库的入口文件，包含 `AutoField` 宏的定义和实现 |
| `Cargo.toml` | 项目依赖和构建配置 |
| `README.md` | 项目文档，包含使用说明和 API 参考 |

## 宏实现细节

### AutoField 宏配置结构

```rust
#[derive(Debug, Clone, Default)]
struct AutoFieldConfig {
    pub snowflake_id: bool,
    pub timestamps: bool,
    pub audit: bool,
    pub tenant: bool,
    pub version: bool,
    pub soft_delete: bool,
}
```

### 宏处理流程

1. **解析属性配置**：解析 `#[auto_field(...)]` 属性，生成配置结构
2. **验证配置**：验证配置的有效性，例如 `audit` 功能必须同时启用 `timestamps` 功能
3. **生成 ActiveModelBehavior 实现**：自动处理字段填充逻辑
4. **生成 QueryExtensions 实现**：提供便捷的查询方法
5. **生成 CustomizationExt 实现**：支持软删除和批量操作
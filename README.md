# auto_field_macros

## Project Overview

`auto_field_macros` is a procedural macro library developed in Rust, designed specifically for the `auto_field_trait` library, providing macro support for automatic field processing. This library simplifies the code writing for developers when using the `auto_field_trait` library by automatically generating the required trait implementations and field processing logic through macro definitions.

### Features

- **Automatic ActiveModelBehavior Implementation**: Automatically handles field filling logic
- **Automatic QueryExtensions Implementation**: Provides convenient query methods
- **Automatic CustomizationExt Implementation**: Supports soft delete and batch operations
- **Flexible Configuration Options**: Select required features through attribute configuration
- **Supports Multiple Field Types**:
  - Snowflake ID generation
  - Timestamp management
  - Audit logging
  - Tenant support
  - Version control
  - Soft delete

### Technical Architecture

- **Language**: Rust
- **Core Dependencies**:
  - `proc-macro2`：Procedural macro support
  - `quote`：Rust code generation
  - `syn`：Rust syntax analysis

## Installation and Configuration

### Installation

Add dependencies to your `Cargo.toml` file:

```toml
dependencies =
    auto_field_trait = { version = "0.1.3", git = "https://github.com/tttq/auto_field_trait.git", features = ["postgres", "with-web"] }
    sea-orm = "0.12"

proc-macro-dependencies =
    auto_field_macros = { version = "0.1.3", git = "https://github.com/tttq/auto_field_macros.git" }
```

### Configuration

The `auto_field_macros` library does not require additional configuration files, only need to be configured through attributes when using it.

## Usage Guide

### Basic Usage

1. **Import Dependencies**:

```rust
use auto_field_macros::AutoField;
use auto_field_trait::QueryExtensions;
use auto_field_trait::CustomizationExt;
```

2. **Define Entity and Use Macro**:

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

3. **Macro Configuration Options**:

The `auto_field` macro supports the following configuration options:

- `snowflake_id`：Enable snowflake ID automatic generation
- `timestamps`：Enable timestamp automatic filling
- `audit`：Enable audit field automatic filling
- `tenant`：Enable tenant field automatic filling
- `version`：Enable version number automatic management
- `soft_delete`：Enable soft delete functionality

You can configure it in the following ways:

```rust
// Way 1: Enable all features
#[auto_field(snowflake_id, timestamps, audit, tenant, version, soft_delete)]

// Way 2: Enable partial features
#[auto_field(timestamps, audit)]

// Way 3: Use key-value form
#[auto_field(snowflake_id = true, timestamps = true)]
```

4. **Use Automatically Generated Functions**:

```rust
// Use QueryExtensions
let users = User::find_not_deleted().all(db).await?;
let users = User::find_by_tenant_id("tenant_123").all(db).await?;

// Use CustomizationExt
User::soft_delete(db, "user_789").await?;
User::soft_delete_many(db, &["user_101", "user_102"]).await?;

// Use batch_update
let update_many = User::batch_update()
    .col_expr(User::Column::Name, Expr::value("new_name"))
    .filter(User::Column::Id.eq("user_123"))
    .exec(db)
    .await?;

// Use batch_insert_many
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

### Advanced Usage

1. **Conditional Configuration**:

You can select the features you need according to your requirements, for example, only enable timestamps and soft delete:

```rust
#[auto_field(timestamps, soft_delete)]
```

2. **Validation Configuration**:

The macro will automatically validate the validity of the configuration, for example, if you enable the `audit` feature, you must also enable the `timestamps` feature, otherwise a compilation error will occur.

3. **Custom Field Names**:

Currently, the `auto_field_macros` library uses fixed field names, such as:
- `create_time`：Creation time
- `update_time`：Update time
- `create_by`：Creator
- `create_id`：Creator ID
- `update_by`：Updater
- `update_id`：Updater ID
- `tenant_id`：Tenant ID
- `tenant_name`：Tenant name
- `version`：Version number
- `delete_flag`：Delete flag

If you need to customize field names, you can modify the source code of the `auto_field_trait` library.

## Notes

### Environment Requirements

- **Rust Version**: 1.65.0 or higher
- **SeaORM Version**: 0.12.x
- **auto_field_trait Version**: Matching the `auto_field_macros` version

### Limitations

1. Currently only supports SeaORM framework
2. Only supports fixed field names, does not support custom field names
3. Must be used with the `auto_field_trait` library
4. Some features have dependencies, for example, the `audit` feature depends on the `timestamps` feature

### Common Issues

1. **Issue**: Compilation error, missing dependencies
   **Solution**: Ensure that all dependencies are correctly installed, including `auto_field_trait` and `sea-orm`

2. **Issue**: Compilation error, invalid configuration
   **Solution**: Check if the macro configuration is correct, for example, the `audit` feature must also enable the `timestamps` feature

3. **Issue**: Auto fields are not being filled correctly
   **Solution**: Ensure that the `HookedSeaOrmPlugin` plugin and context getter are correctly registered

## Project Directory Structure

```
auto_field_macros/
├── src/
│   └── lib.rs                # Library entry file, containing macro definitions
├── Cargo.toml                # Dependency configuration
└── README.md                 # Project documentation
```

### File Usage Description

| File/Folder | Purpose |
| --- | --- |
| `src/lib.rs` | Library entry point, containing the definition and implementation of the `AutoField` macro |
| `Cargo.toml` | Project dependencies and build configuration |
| `README.md` | Project documentation, including usage instructions and API reference |

## Macro Implementation Details

### AutoField Macro Configuration Structure

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

### Macro Processing Flow

1. **Parse Attribute Configuration**: Parse the `#[auto_field(...)]` attribute and generate a configuration structure
2. **Validate Configuration**: Validate the validity of the configuration, for example, the `audit` feature must also enable the `timestamps` feature
3. **Generate ActiveModelBehavior Implementation**: Automatically handle field filling logic
4. **Generate QueryExtensions Implementation**: Provide convenient query methods
5. **Generate CustomizationExt Implementation**: Support soft delete and batch operations
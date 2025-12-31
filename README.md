# AutoField 宏系统

SeaORM 自动字段填充宏，通过 ActiveModelBehavior 生命周期钩子自动填充通用数据库字段。

## 功能特性

- 🔄 **自动字段填充**: 在插入和更新操作时自动填充字段
- 🆔 **雪花ID生成**: 自动生成唯一的雪花ID作为主键
- ⏰ **时间戳管理**: 自动管理创建时间和更新时间
- 👤 **审计跟踪**: 自动记录创建人和修改人信息
- 🏢 **多租户支持**: 自动填充租户信息
- 📊 **版本控制**: 自动管理记录版本号
- 🗑️ **软删除**: 支持逻辑删除功能
- 🔧 **可配置**: 灵活的配置选项，按需启用功能

## 使用方法

### 1. 添加宏到实体

```rust
use auto_field_macros::AutoField;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, AutoField)]
#[sea_orm(table_name = "sys_user")]
#[auto_field(
    snowflake_id = true,        // 启用雪花ID自动生成
    timestamps = true,          // 启用时间戳字段自动填充
    audit = true,              // 启用审计字段自动填充
    tenant = true,             // 启用租户字段自动填充
    version = true,            // 启用版本号自动管理
    soft_delete = true,        // 启用软删除功能
    state = true,              // 启用状态字段自动填充
    default_state = "1",       // 默认状态值
    default_state_name = "启用" // 默认状态名称
)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub create_by: Option<String>,
    pub update_by: Option<String>,
    pub tenant_id: Option<String>,
    pub tenant_name: Option<String>,
    pub version: Option<i32>,
    pub delete_flag: Option<i32>,
    pub state: Option<String>,
    pub state_name: Option<String>,
    
    // 业务字段
    pub user_name: Option<String>,
    pub email: Option<String>,
}
```

### 2. 配置选项

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `snowflake_id` | bool | false | 是否启用雪花ID自动生成 |
| `timestamps` | bool | false | 是否启用时间戳字段自动填充 |
| `audit` | bool | false | 是否启用审计字段自动填充 |
| `tenant` | bool | false | 是否启用租户字段自动填充 |
| `version` | bool | false | 是否启用版本号自动管理 |
| `soft_delete` | bool | false | 是否启用软删除功能 |
| `state` | bool | false | 是否启用状态字段自动填充 |
| `default_state` | String | "1" | 默认状态值 |
| `default_state_name` | String | "启用" | 默认状态名称 |

### 3. 简化配置

```rust
// 使用默认配置（所有功能启用）
#[derive(AutoField)]
#[auto_field]
pub struct Model { ... }

// 只启用特定功能
#[derive(AutoField)]
#[auto_field(timestamps, audit)]
pub struct Model { ... }
```

## 生成的功能

### ActiveModelBehavior 实现

宏会自动生成 `ActiveModelBehavior` 实现，包含：

- `before_insert`: 插入前的字段填充
- `before_update`: 更新前的字段填充

### QueryExtensions 实现

宏会自动生成查询扩展方法：

```rust
// 查询未删除的记录
let users = Entity::find_not_deleted().all(&db).await?;

// 按租户查询
let tenant_users = Entity::find_by_tenant_id("tenant_001").all(&db).await?;

// 按创建人查询
let user_records = Entity::find_by_creator_id("user_123").all(&db).await?;
```

### SoftDeleteExt 实现

宏会自动生成软删除方法：

```rust
// 软删除单个记录
Entity::soft_delete(&db, "user_id").await?;

// 软删除多个记录
Entity::soft_delete_many(&db, &["id1", "id2"]).await?;
```

## 字段映射

| 配置 | 影响的字段 | 插入时行为 | 更新时行为 |
|------|------------|------------|------------|
| `snowflake_id` | `id` | 生成雪花ID | 不变 |
| `timestamps` | `create_time`, `update_time` | 设置当前时间 | 更新 `update_time` |
| `audit` | `create_by`, `update_by` | 设置当前用户 | 更新 `update_by` |
| `tenant` | `tenant_id`, `tenant_name` | 设置当前租户 | 不变 |
| `version` | `version` | 设置为 1 | 递增 |
| `soft_delete` | `delete_flag` | 设置为 0 | 软删除时设置为 1 |
| `state` | `state`, `state_name` | 设置默认值 | 不变 |

## 注意事项

1. **字段保护**: 只有当字段为 `ActiveValue::NotSet` 时才会自动填充
2. **上下文依赖**: 审计和租户字段需要配置上下文提供者
3. **字段类型**: 确保实体字段类型与预期类型匹配
4. **依赖组件**: 雪花ID生成需要注册 `SnowflakeIdGenerator` 组件

## 错误处理

宏会进行以下验证：

- 配置有效性检查
- 字段依赖关系验证
- 类型兼容性检查

编译时错误会提供清晰的错误信息帮助调试。
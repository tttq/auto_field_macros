use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, Lit, Meta};


/// AutoField 宏配置结构
#[derive(Debug, Clone, Default)]
struct AutoFieldConfig {
    pub snowflake_id: bool,
    pub timestamps: bool,
    pub audit: bool,
    pub tenant: bool,
    pub version: bool,
    pub soft_delete: bool,
    pub state: bool,
    pub default_state: Option<String>,
    pub default_state_name: Option<String>,
}

impl AutoFieldConfig {
    /// 从属性中解析配置
    pub fn from_attributes(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut config = Self::default();
        
        for attr in attrs {
            if attr.path().is_ident("auto_field") {
                match &attr.meta {
                    Meta::List(meta_list) => {
                        // 解析 #[auto_field(key = value, ...)] 格式
                        let nested = meta_list.parse_args_with(
                            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
                        )?;
                        
                        for meta in nested {
                            match meta {
                                Meta::NameValue(name_value) => {
                                    let key = name_value.path.get_ident()
                                        .ok_or_else(|| syn::Error::new_spanned(&name_value.path, "Expected identifier"))?
                                        .to_string();
                                    
                                    match key.as_str() {
                                        "snowflake_id" => {
                                            config.snowflake_id = parse_bool_value(&name_value.value)?;
                                        }
                                        "timestamps" => {
                                            config.timestamps = parse_bool_value(&name_value.value)?;
                                        }
                                        "audit" => {
                                            config.audit = parse_bool_value(&name_value.value)?;
                                        }
                                        "tenant" => {
                                            config.tenant = parse_bool_value(&name_value.value)?;
                                        }
                                        "version" => {
                                            config.version = parse_bool_value(&name_value.value)?;
                                        }
                                        "soft_delete" => {
                                            config.soft_delete = parse_bool_value(&name_value.value)?;
                                        }
                                        "state" => {
                                            config.state = parse_bool_value(&name_value.value)?;
                                        }
                                        "default_state" => {
                                            config.default_state = Some(parse_string_value(&name_value.value)?);
                                        }
                                        "default_state_name" => {
                                            config.default_state_name = Some(parse_string_value(&name_value.value)?);
                                        }
                                        _ => {
                                            return Err(syn::Error::new_spanned(
                                                &name_value.path,
                                                format!("Unknown auto_field configuration key: {}", key)
                                            ));
                                        }
                                    }
                                }
                                Meta::Path(path) => {
                                    // 处理 #[auto_field(snowflake_id)] 格式 (默认为 true)
                                    let key = path.get_ident()
                                        .ok_or_else(|| syn::Error::new_spanned(&path, "Expected identifier"))?
                                        .to_string();
                                    
                                    match key.as_str() {
                                        "snowflake_id" => config.snowflake_id = true,
                                        "timestamps" => config.timestamps = true,
                                        "audit" => config.audit = true,
                                        "tenant" => config.tenant = true,
                                        "version" => config.version = true,
                                        "soft_delete" => config.soft_delete = true,
                                        "state" => config.state = true,
                                        _ => {
                                            return Err(syn::Error::new_spanned(
                                                &path,
                                                format!("Unknown auto_field configuration key: {}", key)
                                            ));
                                        }
                                    }
                                }
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        &meta,
                                        "Expected key = value or key format in auto_field attribute"
                                    ));
                                }
                            }
                        }
                    }
                    Meta::Path(_) => {
                        // #[auto_field] 没有参数，使用默认配置
                        config = Self {
                            snowflake_id: true,
                            timestamps: true,
                            audit: true,
                            tenant: false,
                            version: true,
                            soft_delete: true,
                            state: true,
                            default_state: Some("1".to_string()),
                            default_state_name: Some("启用".to_string()),
                        };
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "Invalid auto_field attribute format"
                        ));
                    }
                }
            }
        }
        
        Ok(config)
    }
    
    /// 验证配置的有效性
    pub fn validate(&self) -> syn::Result<()> {
        // 如果启用了状态字段但没有提供默认值，给出警告
        if self.state && self.default_state.is_none() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "state is enabled but default_state is not provided"
            ));
        }
        
        // 如果启用了审计字段，时间戳字段也应该启用
        if self.audit && !self.timestamps {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "audit fields require timestamps to be enabled"
            ));
        }
        
        Ok(())
    }
}

/// 解析布尔值
fn parse_bool_value(expr: &Expr) -> syn::Result<bool> {
    match expr {
        Expr::Lit(expr_lit) => {
            match &expr_lit.lit {
                Lit::Bool(lit_bool) => Ok(lit_bool.value),
                _ => Err(syn::Error::new_spanned(expr, "Expected boolean value")),
            }
        }
        _ => Err(syn::Error::new_spanned(expr, "Expected boolean literal")),
    }
}

/// 解析字符串值
fn parse_string_value(expr: &Expr) -> syn::Result<String> {
    match expr {
        Expr::Lit(expr_lit) => {
            match &expr_lit.lit {
                Lit::Str(lit_str) => Ok(lit_str.value()),
                _ => Err(syn::Error::new_spanned(expr, "Expected string value")),
            }
        }
        _ => Err(syn::Error::new_spanned(expr, "Expected string literal")),
    }
}

/// 分析实体字段结构
#[derive(Debug)]
struct EntityFields {
    pub has_id: bool,
    pub has_create_time: bool,
    pub has_update_time: bool,
    pub has_create_by: bool,
    pub has_create_id: bool,
    pub has_update_by: bool,
    pub has_update_id: bool,
    pub has_tenant_id: bool,
    pub has_tenant_name: bool,
    pub has_version: bool,
    pub has_delete_flag: bool,
    pub has_state: bool,
    pub has_state_name: bool,
}

impl EntityFields {
    /// 从结构体字段中分析
    pub fn from_fields(fields: &Fields) -> Self {
        let mut entity_fields = Self {
            has_id: false,
            has_create_time: false,
            has_update_time: false,
            has_create_by: false,
            has_create_id: false,
            has_update_by: false,
            has_update_id: false,
            has_tenant_id: false,
            has_tenant_name: false,
            has_version: false,
            has_delete_flag: false,
            has_state: false,
            has_state_name: false,
        };
        
        if let Fields::Named(fields_named) = fields {
            for field in &fields_named.named {
                if let Some(ident) = &field.ident {
                    match ident.to_string().as_str() {
                        "id" => entity_fields.has_id = true,
                        "create_time" => entity_fields.has_create_time = true,
                        "update_time" => entity_fields.has_update_time = true,
                        "create_by" => entity_fields.has_create_by = true,
                        "create_id" => entity_fields.has_create_id = true,
                        "update_by" => entity_fields.has_update_by = true,
                        "update_id" => entity_fields.has_update_id = true,
                        "tenant_id" => entity_fields.has_tenant_id = true,
                        "tenant_name" => entity_fields.has_tenant_name = true,
                        "version" => entity_fields.has_version = true,
                        "delete_flag" => entity_fields.has_delete_flag = true,
                        "state" => entity_fields.has_state = true,
                        "state_name" => entity_fields.has_state_name = true,
                        _ => {}
                    }
                }
            }
        }
        
        entity_fields
    }
}

/// AutoField 派生宏
#[proc_macro_derive(AutoField, attributes(auto_field))]
pub fn derive_auto_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    match generate_auto_field_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// 生成 AutoField 实现
fn generate_auto_field_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    // 解析配置
    let config = AutoFieldConfig::from_attributes(&input.attrs)?;
    config.validate()?;
    
    // 分析实体字段
    let entity_fields = match &input.data {
        Data::Struct(data_struct) => EntityFields::from_fields(&data_struct.fields),
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "AutoField can only be derived for structs"
            ));
        }
    };
    
    let struct_name = &input.ident;
    // SeaORM 生成的 ActiveModel 类型名称是 ActiveModel
    let active_model_name = syn::Ident::new("ActiveModel", struct_name.span());
    
    // 生成 ActiveModelBehavior 实现
    let behavior_impl = generate_active_model_behavior(&config, &entity_fields, &active_model_name)?;
    
    // 生成 QueryExtensions 实现
    let query_extensions_impl = generate_query_extensions(&config, &entity_fields, struct_name)?;
    
    // 生成 SoftDeleteExt 实现
    let soft_delete_impl = generate_soft_delete_ext(&config, &entity_fields, struct_name, &active_model_name)?;
    
    Ok(quote! {
        #behavior_impl
        #query_extensions_impl
        #soft_delete_impl
    })
}

/// 生成 ActiveModelBehavior 实现
fn generate_active_model_behavior(
    config: &AutoFieldConfig,
    entity_fields: &EntityFields,
    active_model_name: &syn::Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut before_insert_body = Vec::new();
    let mut before_update_body = Vec::new();
    
    // 添加字段值保护逻辑的辅助宏
    before_insert_body.push(quote! {
        // 字段值保护逻辑：检查 ActiveValue 状态，仅在字段为空时填充
        // 保护已有值不被覆盖
        // 正确处理 Option<T> 类型，区分 None 和 Some(value) 状态
        macro_rules! should_fill_field {
            // 处理 Option<T> 类型字段
            ($field:expr) => {
                match &$field {
                    // ActiveValue::NotSet - 字段未设置，需要填充
                    sea_orm::ActiveValue::NotSet => true,
                    // ActiveValue::Set(None) - 字段设置为 None，需要填充
                    sea_orm::ActiveValue::Set(None) => true,
                    // ActiveValue::Set(Some(value)) - 字段已有值，不填充，保护已有值
                    sea_orm::ActiveValue::Set(Some(_)) => false,
                    // ActiveValue::Unchanged(None) - 字段为 None 且未改变，需要填充
                    sea_orm::ActiveValue::Unchanged(None) => true,
                    // ActiveValue::Unchanged(Some(value)) - 字段有值且未改变，不填充
                    sea_orm::ActiveValue::Unchanged(Some(_)) => false,
                }
            };
            // 处理非 Option 类型字段 (如 String, i32 等)
            ($field:expr, $non_option:ty) => {
                match &$field {
                    // 只有 NotSet 状态才填充非 Option 类型字段
                    sea_orm::ActiveValue::NotSet => true,
                    // 其他状态都不填充，保护已有值
                    _ => false,
                }
            };
        }
    });
    
    // 生成插入时的字段填充逻辑
    if config.snowflake_id && entity_fields.has_id {
        before_insert_body.push(quote! {
            // 雪花ID生成和填充 - 仅在字段为空时填充，保护已有值
            if should_fill_field!(self.id, String) {
                use spring::plugin::ComponentRegistry;

                
                // 通过 App::global().get_component 获取雪花ID生成器，带错误处理
                match spring::App::global().get_component::<snowflake::SnowflakeIdGenerator>() {
                    Some(mut generator) => {
                        // 调用 generate().to_string() 生成ID，带错误处理
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            generator.generate().to_string()
                        })) {
                            Ok(id) => {
                                self.id = sea_orm::ActiveValue::Set(id);
                                log::debug!("雪花ID生成成功");
                            }
                            Err(_) => {
                                log::error!("雪花ID生成过程中发生 panic，跳过ID生成");
                                // 跳过ID生成，让数据库或其他逻辑处理
                            }
                        }
                    }
                    None => {
                        // 处理组件未找到场景：记录错误并跳过ID生成
                        log::error!("雪花ID生成器组件未找到，跳过ID生成");
                        // 不设置ID，让数据库或其他逻辑处理
                    }
                }
            }
        });
    }
    
    if config.timestamps && entity_fields.has_create_time {
        before_insert_body.push(quote! {
            // 创建时间填充 - 仅在字段为空时填充，保护已有值
            if should_fill_field!(self.create_time) {
                self.create_time = sea_orm::ActiveValue::Set(Some(chrono::Utc::now().naive_utc()));
            }
        });
    }
    
    if config.timestamps && entity_fields.has_update_time {
        before_insert_body.push(quote! {
            // 更新时间填充 - 仅在字段为空时填充，保护已有值
            if should_fill_field!(self.update_time) {
                self.update_time = sea_orm::ActiveValue::Set(Some(chrono::Utc::now().naive_utc()));
            }
        });
    }
    
    if config.audit && entity_fields.has_create_by {
        before_insert_body.push(quote! {
            // 创建人填充 - 仅在字段为空时填充，保护已有值
            // 空值配置跳过：如果上下文信息为空或 null，跳过该字段的填充
            if should_fill_field!(self.create_by) {
                // 使用预先获取的上下文
                if let Some(user_name) = &context.user_name {
                    if !user_name.is_empty() {
                        self.create_by = sea_orm::ActiveValue::Set(Some(user_name.clone()));
                    }
                }
            }
        });
    }
    
    // 添加 create_id 字段填充 (审计字段)
    if config.audit && entity_fields.has_create_id {
        before_insert_body.push(quote! {
            // 创建人ID填充 - 仅在字段为空时填充，保护已有值
            // 空值配置跳过：如果上下文信息为空或 null，跳过该字段的填充
            if should_fill_field!(self.create_id) {
                // 使用预先获取的上下文
                if let Some(user_id) = &context.user_id {
                    if !user_id.is_empty() {
                        self.create_id = sea_orm::ActiveValue::Set(Some(user_id.clone()));
                    }
                }
            }
        });
    }
    
    if config.tenant && entity_fields.has_tenant_id {
        before_insert_body.push(quote! {
            // 租户ID填充 - 仅在字段为空时填充，保护已有值
            // 空值配置跳过：如果上下文信息为空或 null，跳过该字段的填充
            if should_fill_field!(self.tenant_id) {
                // 使用预先获取的上下文
                if let Some(tenant_id) = &context.tenant_id {
                    if !tenant_id.is_empty() {
                        self.tenant_id = sea_orm::ActiveValue::Set(Some(tenant_id.clone()));
                    }
                }
            }
        });
    }
    
    if config.tenant && entity_fields.has_tenant_name {
        before_insert_body.push(quote! {
            // 租户名称填充 - 仅在字段为空时填充，保护已有值
            // 空值配置跳过：如果上下文信息为空或 null，跳过该字段的填充
            if should_fill_field!(self.tenant_name) {
                // 使用预先获取的上下文
                if let Some(tenant_name) = &context.tenant_name {
                    if !tenant_name.is_empty() {
                        self.tenant_name = sea_orm::ActiveValue::Set(Some(tenant_name.clone()));
                    }
                }
            }
        });
    }
    
    if config.version && entity_fields.has_version {
        before_insert_body.push(quote! {
            // 版本号填充 - 仅在字段为空时填充，初始值为1，保护已有值
            if should_fill_field!(self.version) {
                self.version = sea_orm::ActiveValue::Set(Some(1));
            }
        });
    }
    
    if config.soft_delete && entity_fields.has_delete_flag {
        before_insert_body.push(quote! {
            // 删除标记填充 - 仅在字段为空时填充，初始值为0(未删除)，保护已有值
            if should_fill_field!(self.delete_flag) {
                self.delete_flag = sea_orm::ActiveValue::Set(Some(0));
            }
        });
    }
    
    if config.state && entity_fields.has_state {
        let default_state = config.default_state.as_deref().unwrap_or("1");
        before_insert_body.push(quote! {
            // 状态字段填充 - 仅在字段为空时填充，保护已有值
            if should_fill_field!(self.state) {
                self.state = sea_orm::ActiveValue::Set(Some(#default_state.to_string()));
            }
        });
    }
    
    if config.state && entity_fields.has_state_name {
        let default_state_name = config.default_state_name.as_deref().unwrap_or("启用");
        before_insert_body.push(quote! {
            // 状态名称填充 - 仅在字段为空时填充，保护已有值
            if should_fill_field!(self.state_name) {
                self.state_name = sea_orm::ActiveValue::Set(Some(#default_state_name.to_string()));
            }
        });
    }
    
    // 生成更新时的字段填充逻辑
    if config.timestamps && entity_fields.has_update_time {
        before_update_body.push(quote! {
            // 更新时间戳填充 - 每次更新都设置当前时间
            self.update_time = sea_orm::ActiveValue::Set(Some(chrono::Utc::now().naive_utc()));
        });
    }
    
    if config.audit && entity_fields.has_update_by {
        before_update_body.push(quote! {
            // 修改人信息填充 - 从上下文获取用户名
            // 空值配置跳过：如果上下文信息为空或 null，跳过该字段的填充
            if let Some(user_name) = &context.user_name {
                if !user_name.is_empty() {
                    self.update_by = sea_orm::ActiveValue::Set(Some(user_name.clone()));
                }
            }
        });
    }
    
    // 添加 update_id 字段填充 (审计字段)
    if config.audit && entity_fields.has_update_id {
        before_update_body.push(quote! {
            // 修改人ID填充 - 从上下文获取用户ID
            // 空值配置跳过：如果上下文信息为空或 null，跳过该字段的填充
            if let Some(user_id) = &context.user_id {
                if !user_id.is_empty() {
                    self.update_id = sea_orm::ActiveValue::Set(Some(user_id.clone()));
                }
            }
        });
    }
    
    if config.version && entity_fields.has_version {
        before_update_body.push(quote! {
            // 版本号自增逻辑 - 处理不同的 ActiveValue 状态和 Option<i32> 类型
            match &self.version {
                sea_orm::ActiveValue::Set(Some(current_version)) => {
                    // 如果版本号已设置且有值，则递增
                    self.version = sea_orm::ActiveValue::Set(Some(current_version + 1));
                }
                sea_orm::ActiveValue::Set(None) => {
                    // 如果版本号设置为 None，设置为1
                    self.version = sea_orm::ActiveValue::Set(Some(1));
                }
                sea_orm::ActiveValue::Unchanged(Some(current_version)) => {
                    // 如果版本号未改变且有值，也需要递增
                    self.version = sea_orm::ActiveValue::Set(Some(current_version + 1));
                }
                sea_orm::ActiveValue::Unchanged(None) => {
                    // 如果版本号未改变且为 None，设置为1
                    self.version = sea_orm::ActiveValue::Set(Some(1));
                }
                sea_orm::ActiveValue::NotSet => {
                    // 如果版本号未设置，设置为1
                    self.version = sea_orm::ActiveValue::Set(Some(1));
                }
            }
        });
    }
    
    Ok(quote! {
        use async_trait::async_trait;

        #[async_trait::async_trait]
        impl sea_orm::ActiveModelBehavior for #active_model_name {
            async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, sea_orm::DbErr>
            where
                C: sea_orm::ConnectionTrait,
            {
                // 只获取一次上下文，避免重复调用
                let context = ::auto_field_trait::auto_field_trait::AutoFieldContext::current_safe();
                
                if insert {
                    // 插入时的字段填充逻辑
                    #(#before_insert_body)*
                } else {
                    // 更新时的字段填充逻辑
                    #(#before_update_body)*
                }
                Ok(self)
            }
        }
    })
}

/// 生成 QueryExtensions 实现
fn generate_query_extensions(
    config: &AutoFieldConfig,
    entity_fields: &EntityFields,
    struct_name: &syn::Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    // SeaORM 生成的 Entity 类型名称是 Entity
    let entity_name = syn::Ident::new("Entity", struct_name.span());
    
    let mut methods = Vec::new();
    
    // find_not_deleted 方法
    if config.soft_delete && entity_fields.has_delete_flag {
        methods.push(quote! {
            fn find_not_deleted() -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find().filter(Self::Column::DeleteFlag.ne(1))
            }
        });
    } else {
        methods.push(quote! {
            fn find_not_deleted() -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find()
            }
        });
    }
    
    // 租户相关查询方法
    if config.tenant && entity_fields.has_tenant_id {
        methods.push(quote! {
            fn find_by_tenant_id(tenant_id: &str) -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find().filter(Self::Column::TenantId.eq(tenant_id))
            }
        });
    } else {
        methods.push(quote! {
            fn find_by_tenant_id(_tenant_id: &str) -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find()
            }
        });
    }
    
    if config.tenant && entity_fields.has_tenant_name {
        methods.push(quote! {
            fn find_by_tenant_name(tenant_name: &str) -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find().filter(Self::Column::TenantName.eq(tenant_name))
            }
        });
    } else {
        methods.push(quote! {
            fn find_by_tenant_name(_tenant_name: &str) -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find()
            }
        });
    }
    
    // 创建人相关查询方法
    if config.audit && entity_fields.has_create_by {
        methods.push(quote! {
            fn find_by_creator_id(user_id: &str) -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find().filter(Self::Column::CreateBy.eq(user_id))
            }
        });
    } else {
        methods.push(quote! {
            fn find_by_creator_id(_user_id: &str) -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find()
            }
        });
    }
    
    // 注意：create_by_name 字段在当前实体中不存在，所以这里使用 create_by 字段
    methods.push(quote! {
        fn find_by_creator_name(_user_name: &str) -> sea_orm::Select<Self> {
            use sea_orm::EntityTrait;
            // 注意：需要根据实际的用户名字段进行查询，这里暂时返回空查询
            Self::find().filter(sea_orm::Condition::all())
        }
    });
    
    Ok(quote! {
        impl ::auto_field_trait::auto_field_trait::QueryExtensions for #entity_name {
            #(#methods)*
        }
    })
}

/// 生成 SoftDeleteExt 实现
fn generate_soft_delete_ext(
    config: &AutoFieldConfig,
    entity_fields: &EntityFields,
    struct_name: &syn::Ident,
    active_model_name: &syn::Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    // SeaORM 生成的 Entity 类型名称是 Entity
    let entity_name = syn::Ident::new("Entity", struct_name.span());
    
    if !config.soft_delete || !entity_fields.has_delete_flag {
        // 如果没有启用软删除，返回空实现
        return Ok(quote! {
            #[async_trait::async_trait]
            impl ::auto_field_trait::auto_field_trait::SoftDeleteExt for #entity_name {
                async fn soft_delete<C>(_db: &C, _id: &str) -> Result<(), sea_orm::DbErr>
                where
                    C: sea_orm::ConnectionTrait,
                {
                    Err(sea_orm::DbErr::Custom("Soft delete not enabled for this entity".to_string()))
                }
                
                async fn soft_delete_many<C>(_db: &C, _ids: &[String]) -> Result<(), sea_orm::DbErr>
                where
                    C: sea_orm::ConnectionTrait,
                {
                    Err(sea_orm::DbErr::Custom("Soft delete not enabled for this entity".to_string()))
                }
            }
        });
    }
    
    Ok(quote! {
        #[async_trait::async_trait]
        impl ::auto_field_trait::auto_field_trait::SoftDeleteExt for #entity_name {
            async fn soft_delete<C>(db: &C, id: &str) -> Result<(), sea_orm::DbErr>
            where
                C: sea_orm::ConnectionTrait,
            {
                use sea_orm::EntityTrait;
                let model = Self::find_by_id(id).one(db).await?;
                if let Some(model) = model {
                    let mut active_model: #active_model_name = model.into();
                    
                    // 软删除字段填充：设置删除标记为1，触发 before_update 钩子
                    active_model.delete_flag = sea_orm::ActiveValue::Set(Some(1));
                    
                    // 通过 update 操作触发自动字段填充逻辑 (before_update 钩子)
                    // 这会自动填充 update_time, update_by, update_id, version++ 等字段
                    active_model.update(db).await?;
                }
                Ok(())
            }
            
            async fn soft_delete_many<C>(db: &C, ids: &[String]) -> Result<(), sea_orm::DbErr>
            where
                C: sea_orm::ConnectionTrait,
            {
                for id in ids {
                    Self::soft_delete(db, id).await?;
                }
                Ok(())
            }
        }
    })
}
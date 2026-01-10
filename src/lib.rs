use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Expr, Lit, Meta};


/// AutoField 宏配置结构
#[derive(Debug, Clone, Default)]
struct AutoFieldConfig {
    pub snowflake_id: bool,
    pub timestamps: bool,
    pub audit: bool,
    pub tenant: bool,
    pub version: bool,
    pub soft_delete: bool,
    pub skip_default_filters: bool,
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
                            syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
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
                                        "skip_default_filters" => {
                                            config.skip_default_filters = parse_bool_value(&name_value.value)?;
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
                                        "skip_default_filters" => config.skip_default_filters = false,
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
                            snowflake_id: false,
                            timestamps: false,
                            audit: false,
                            tenant: false,
                            version: false,
                            soft_delete: false,
                            skip_default_filters: false,
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
    
    // 不再分析实体字段，完全使用配置文件中的参数
    // let entity_fields = match &input.data {
    //     Data::Struct(data_struct) => EntityFields::from_fields(&data_struct.fields),
    //     _ => {
    //         return Err(syn::Error::new_spanned(
    //             input,
    //             "AutoField can only be derived for structs"
    //         ));
    //     }
    // };
    
    let struct_name = &input.ident;
    // SeaORM 生成的 ActiveModel 类型名称是 ActiveModel
    let active_model_name = syn::Ident::new("ActiveModel", struct_name.span());
    
    // 生成 ActiveModelBehavior 实现
    let behavior_impl = generate_active_model_behavior(&config, &active_model_name)?;
    
    // 生成 QueryExtensions 实现
    let query_extensions_impl = generate_query_extensions(&config, struct_name)?;
    
    // 生成 SoftDeleteExt 实现
    let soft_delete_impl = generate_soft_delete_ext(&config, struct_name, &active_model_name)?;
    
    Ok(quote! {
        #behavior_impl
        #query_extensions_impl
        #soft_delete_impl
    })
}

/// 生成 ActiveModelBehavior 实现
fn generate_active_model_behavior(
    config: &AutoFieldConfig,
    active_model_name: &syn::Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut before_insert_body = Vec::new();
    let mut before_update_body = Vec::new();
    
    // 添加字段值保护逻辑的辅助宏
    before_insert_body.push(quote! {
        macro_rules! should_fill_field {
            // 处理 Option<T> 类型字段
            ($field:expr) => {
                match &$field {
                    sea_orm::ActiveValue::NotSet => true,
                    sea_orm::ActiveValue::Set(None) => true,
                    sea_orm::ActiveValue::Set(Some(_)) => false,
                    sea_orm::ActiveValue::Unchanged(None) => true,
                    sea_orm::ActiveValue::Unchanged(Some(_)) => false,
                }
            };
            // 处理非 Option 类型字段
            ($field:expr, $non_option:ty) => {
                matches!(&$field, sea_orm::ActiveValue::NotSet)
            };
        }
    });
    
    // 生成插入时的字段填充逻辑
    if config.snowflake_id {
        before_insert_body.push(quote! {
            if should_fill_field!(self.id, String) {
                use spring::plugin::ComponentRegistry;

                if let Some(mut generator) = spring::App::global().get_component::<snowflake::SnowflakeIdGenerator>() {
                    if let Ok(id) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| generator.generate().to_string())) {
                        self.id = sea_orm::ActiveValue::Set(id);
                    }
                }
            }
        });
    }
    
    if config.timestamps {
        before_insert_body.push(quote! {
            if should_fill_field!(self.create_time) {
                self.create_time = sea_orm::ActiveValue::Set(Some(chrono::Utc::now().naive_utc()));
            }
            if should_fill_field!(self.update_time) {
                self.update_time = sea_orm::ActiveValue::Set(Some(chrono::Utc::now().naive_utc()));
            }
        });
    }
    
    if config.audit {
        before_insert_body.push(quote! {
            if should_fill_field!(self.create_by) {
                if let Some(user_name) = &context.user_name {
                    if !user_name.is_empty() {
                        self.create_by = sea_orm::ActiveValue::Set(Some(user_name.clone()));
                    }
                }
                if should_fill_field!(self.create_id) {
                    if let Some(user_id) = &context.user_id {
                        if !user_id.is_empty() {
                            self.create_id = sea_orm::ActiveValue::Set(Some(user_id.clone()));
                        }
                    }
                }
            }
        });
    }
    
    if config.tenant {
        before_insert_body.push(quote! {
            if should_fill_field!(self.tenant_id) {
                if let Some(tenant_id) = &context.tenant_id {
                    if !tenant_id.is_empty() {
                        self.tenant_id = sea_orm::ActiveValue::Set(Some(tenant_id.clone()));
                    }
                }
                if should_fill_field!(self.tenant_name) {
                    if let Some(tenant_name) = &context.tenant_name {
                        if !tenant_name.is_empty() {
                            self.tenant_name = sea_orm::ActiveValue::Set(Some(tenant_name.clone()));
                        }
                    }
                }
            }
        });
    }
    
    if config.version {
        before_insert_body.push(quote! {
            if should_fill_field!(self.version) {
                self.version = sea_orm::ActiveValue::Set(Some(1));
            }
        });
    }
    
    if config.soft_delete {
        before_insert_body.push(quote! {
            if should_fill_field!(self.delete_flag) {
                self.delete_flag = sea_orm::ActiveValue::Set(Some(0));
            }
        });
    }
    
    // 生成更新时的字段填充逻辑
    if config.timestamps {
        before_update_body.push(quote! {
            self.update_time = sea_orm::ActiveValue::Set(Some(chrono::Utc::now().naive_utc()));
        });
    }
    
    if config.audit {
        before_update_body.push(quote! {
            if let Some(user_name) = &context.user_name {
                if !user_name.is_empty() {
                    self.update_by = sea_orm::ActiveValue::Set(Some(user_name.clone()));
                }
            }
            if let Some(user_id) = &context.user_id {
                if !user_id.is_empty() {
                    self.update_id = sea_orm::ActiveValue::Set(Some(user_id.clone()));
                }
            }
        });
    }
    
    if config.version {
        before_update_body.push(quote! {
            match &self.version {
                sea_orm::ActiveValue::Set(Some(current_version)) => {
                    self.version = sea_orm::ActiveValue::Set(Some(current_version + 1));
                }
                sea_orm::ActiveValue::Set(None) => {
                    self.version = sea_orm::ActiveValue::Set(Some(1));
                }
                sea_orm::ActiveValue::Unchanged(Some(current_version)) => {
                    self.version = sea_orm::ActiveValue::Set(Some(current_version + 1));
                }
                sea_orm::ActiveValue::Unchanged(None) => {
                    self.version = sea_orm::ActiveValue::Set(Some(1));
                }
                sea_orm::ActiveValue::NotSet => {
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
                let context = ::auto_field_trait::auto_field_trait::AutoFieldContext::current_safe();
                
                if insert {
                    #(#before_insert_body)*
                } else {
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
    struct_name: &syn::Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    // SeaORM 生成的 Entity 类型名称是 Entity
    let entity_name = syn::Ident::new("Entity", struct_name.span());
    
    let mut methods = Vec::new();
    
    // find_not_deleted 方法 - 总是添加 delete_flag = 0 条件
    if config.soft_delete {
        methods.push(quote! {
            fn find_not_deleted() -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find().filter(Self::Column::DeleteFlag.eq(0))
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
    if config.tenant {
        if config.soft_delete {
            methods.push(quote! {
                fn find_by_tenant_id(tenant_id: &str) -> sea_orm::Select<Self> {
                    use sea_orm::EntityTrait;
                    Self::find()
                        .filter(Self::Column::TenantId.eq(tenant_id))
                        .filter(Self::Column::DeleteFlag.eq(0))
                }
            });
        } else {
            methods.push(quote! {
                fn find_by_tenant_id(tenant_id: &str) -> sea_orm::Select<Self> {
                    use sea_orm::EntityTrait;
                    Self::find().filter(Self::Column::TenantId.eq(tenant_id))
                }
            });
        }
    } else {
        methods.push(quote! {
            fn find_by_tenant_id(_tenant_id: &str) -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find()
            }
        });
    }
    
    // 创建人相关查询方法
    if config.audit {
        // find_by_creator_id 方法
        if config.soft_delete {
            methods.push(quote! {
                fn find_by_creator_id(user_id: &str) -> sea_orm::Select<Self> {
                    use sea_orm::EntityTrait;
                    Self::find()
                        .filter(Self::Column::CreateId.eq(user_id))
                        .filter(Self::Column::DeleteFlag.eq(0))
                }
            });
        } else {
            methods.push(quote! {
                fn find_by_creator_id(user_id: &str) -> sea_orm::Select<Self> {
                    use sea_orm::EntityTrait;
                    Self::find().filter(Self::Column::CreateId.eq(user_id))
                }
            });
        }
        
        // find_by_creator_name 方法
        if config.soft_delete {
            methods.push(quote! {
                fn find_by_creator_name(user_name: &str) -> sea_orm::Select<Self> {
                    use sea_orm::EntityTrait;
                    Self::find()
                        .filter(Self::Column::CreateBy.eq(user_name))
                        .filter(Self::Column::DeleteFlag.eq(0))
                }
            });
        } else {
            methods.push(quote! {
                fn find_by_creator_name(user_name: &str) -> sea_orm::Select<Self> {
                    use sea_orm::EntityTrait;
                    Self::find().filter(Self::Column::CreateBy.eq(user_name))
                }
            });
        }
    } else {
        methods.push(quote! {
            fn find_by_creator_id(_user_id: &str) -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find()
            }
        });
        methods.push(quote! {
            fn find_by_creator_name(_user_name: &str) -> sea_orm::Select<Self> {
                use sea_orm::EntityTrait;
                Self::find()
            }
        });
    }
    
    Ok(quote! {
        impl ::auto_field_trait::auto_field_trait::QueryExtensions for #entity_name {
            #(#methods)*
        }
    })
}

/// 生成 SoftDeleteExt 实现
fn generate_soft_delete_ext(
    config: &AutoFieldConfig,
    struct_name: &syn::Ident,
    active_model_name: &syn::Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    // SeaORM 生成的 Entity 类型名称是 Entity
    let entity_name = syn::Ident::new("Entity", struct_name.span());
    
    if !config.soft_delete {
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
                if let Some(model) = Self::find_by_id(id).one(db).await? {
                    let mut active_model: #active_model_name = model.into();
                    
                    // 设置删除标记为1，触发 before_update 钩子
                    active_model.delete_flag = sea_orm::ActiveValue::Set(Some(1));
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
# Platform Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided for
embedded deployments.

## SchoolRepository

```rust
#[async_trait]
pub trait SchoolRepository: Send + Sync {
    async fn get(&self, id: SchoolId) -> Result<Option<School>>;
    async fn get_by_domain(&self, domain: &Domain) -> Result<Option<School>>;
    async fn get_by_code(&self, code: &SchoolCode) -> Result<Option<School>>;
    async fn list(&self) -> Result<Vec<School>>;
    async fn list_approved(&self) -> Result<Vec<School>>;
    async fn list_pending(&self) -> Result<Vec<School>>;
    async fn insert(&self, s: &School) -> Result<()>;
    async fn update(&self, s: &School) -> Result<()>;
}
```

## UserRepository

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get(&self, id: UserId) -> Result<Option<User>>;
    async fn get_by_email(&self, school: SchoolId, email: &EmailAddress) -> Result<Option<User>>;
    async fn get_by_username(&self, school: SchoolId, username: &Username) -> Result<Option<User>>;
    async fn get_by_phone(&self, school: SchoolId, phone: &PhoneNumber) -> Result<Option<User>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<User>>;
    async fn list_by_role(&self, school: SchoolId, role_id: RoleId) -> Result<Vec<User>>;
    async fn list_by_usertype(&self, school: SchoolId, usertype: UserType) -> Result<Vec<User>>;
    async fn insert(&self, u: &User) -> Result<()>;
    async fn update(&self, u: &User) -> Result<()>;
    async fn query(&self, q: UserQuery) -> Result<Vec<User>>;
    async fn count(&self, q: UserQuery) -> Result<u64>;
    async fn page(&self, q: UserQuery, offset: u32, limit: u32) -> Result<Page<User>>;
}
```

## OtpCodeRepository

```rust
#[async_trait]
pub trait OtpCodeRepository: Send + Sync {
    async fn get(&self, id: OtpCodeId) -> Result<Option<OtpCode>>;
    async fn latest_for_user(&self, user: UserId, channel: OtpChannel) -> Result<Option<OtpCode>>;
    async fn insert(&self, o: &OtpCode) -> Result<()>;
    async fn mark_consumed(&self, id: OtpCodeId) -> Result<()>;
    async fn mark_expired(&self, id: OtpCodeId) -> Result<()>;
    async fn purge_expired(&self, before: Timestamp) -> Result<u64>;
}
```

## CourseRepository

```rust
#[async_trait]
pub trait CourseRepository: Send + Sync {
    async fn get(&self, id: CourseId) -> Result<Option<Course>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Course>>;
    async fn list_by_category(&self, school: SchoolId, category: CourseCategoryId) -> Result<Vec<Course>>;
    async fn list_published(&self, school: SchoolId) -> Result<Vec<Course>>;
    async fn insert(&self, c: &Course) -> Result<()>;
    async fn update(&self, c: &Course) -> Result<()>;
    async fn delete(&self, id: CourseId) -> Result<()>;
}
```

## CourseCategoryRepository

```rust
#[async_trait]
pub trait CourseCategoryRepository: Send + Sync {
    async fn get(&self, id: CourseCategoryId) -> Result<Option<CourseCategory>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<CourseCategory>>;
    async fn insert(&self, c: &CourseCategory) -> Result<()>;
    async fn update(&self, c: &CourseCategory) -> Result<()>;
    async fn delete(&self, id: CourseCategoryId) -> Result<()>;
    async fn referencing_courses(&self, id: CourseCategoryId) -> Result<u64>;
}
```

## CoursePageRepository

```rust
#[async_trait]
pub trait CoursePageRepository: Send + Sync {
    async fn get(&self, id: CoursePageId) -> Result<Option<CoursePage>>;
    async fn get_by_course(&self, course: CourseId) -> Result<Option<CoursePage>>;
    async fn insert(&self, p: &CoursePage) -> Result<()>;
    async fn update(&self, p: &CoursePage) -> Result<()>;
    async fn delete(&self, id: CoursePageId) -> Result<()>;
}
```

## CustomFieldRepository

```rust
#[async_trait]
pub trait CustomFieldRepository: Send + Sync {
    async fn get(&self, id: CustomFieldId) -> Result<Option<CustomField>>;
    async fn list_for_form(&self, school: SchoolId, form_name: &FormName) -> Result<Vec<CustomField>>;
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<CustomField>>;
    async fn insert(&self, f: &CustomField) -> Result<()>;
    async fn update(&self, f: &CustomField) -> Result<()>;
    async fn delete(&self, id: CustomFieldId) -> Result<()>;
    async fn referencing_values(&self, id: CustomFieldId) -> Result<u64>;
}
```

## CustomFieldValueRepository

```rust
#[async_trait]
pub trait CustomFieldValueRepository: Send + Sync {
    async fn get(&self, id: CustomFieldValueId) -> Result<Option<CustomFieldValue>>;
    async fn find(&self, field: CustomFieldId, entity_id: Uuid, entity_type: EntityType) -> Result<Option<CustomFieldValue>>;
    async fn list_for_entity(&self, entity_id: Uuid, entity_type: EntityType) -> Result<Vec<CustomFieldValue>>;
    async fn insert(&self, v: &CustomFieldValue) -> Result<()>;
    async fn update(&self, v: &CustomFieldValue) -> Result<()>;
    async fn delete(&self, id: CustomFieldValueId) -> Result<()>;
}
```

## ChartOfAccountRepository

```rust
#[async_trait]
pub trait ChartOfAccountRepository: Send + Sync {
    async fn get(&self, id: ChartOfAccountId) -> Result<Option<ChartOfAccount>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ChartOfAccount>>;
    async fn list_expense(&self, school: SchoolId) -> Result<Vec<ChartOfAccount>>;
    async fn list_income(&self, school: SchoolId) -> Result<Vec<ChartOfAccount>>;
    async fn insert(&self, c: &ChartOfAccount) -> Result<()>;
    async fn update(&self, c: &ChartOfAccount) -> Result<()>;
    async fn delete(&self, id: ChartOfAccountId) -> Result<()>;
}
```

## BaseGroupRepository / BaseSetupRepository

Each follows the same pattern: `get`, `list`, `insert`, `update`,
`delete`, plus `list_for_school`.

## ModuleRepository

```rust
#[async_trait]
pub trait ModuleRepository: Send + Sync {
    async fn get(&self, id: ModuleId) -> Result<Option<Module>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Module>>;
    async fn list_enabled(&self, school: SchoolId) -> Result<Vec<Module>>;
    async fn list_ordered(&self, school: SchoolId) -> Result<Vec<Module>>;
    async fn insert(&self, m: &Module) -> Result<()>;
    async fn update(&self, m: &Module) -> Result<()>;
    async fn delete(&self, id: ModuleId) -> Result<()>;
    async fn referencing_links(&self, id: ModuleId) -> Result<u64>;
}
```

## ModuleLinkRepository

```rust
#[async_trait]
pub trait ModuleLinkRepository: Send + Sync {
    async fn get(&self, id: ModuleLinkId) -> Result<Option<ModuleLink>>;
    async fn list_for_module(&self, module: ModuleId) -> Result<Vec<ModuleLink>>;
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<ModuleLink>>;
    async fn insert(&self, l: &ModuleLink) -> Result<()>;
    async fn update(&self, l: &ModuleLink) -> Result<()>;
    async fn delete(&self, id: ModuleLinkId) -> Result<()>;
    async fn referencing_role_permissions(&self, id: ModuleLinkId) -> Result<u64>;
}
```

## AddOnRepository

```rust
#[async_trait]
pub trait AddOnRepository: Send + Sync {
    async fn get(&self, id: AddOnId) -> Result<Option<AddOn>>;
    async fn list(&self) -> Result<Vec<AddOn>>;
    async fn list_installed(&self, school: SchoolId) -> Result<Vec<AddOn>>;
    async fn insert(&self, a: &AddOn) -> Result<()>;
    async fn record_installation(&self, school: SchoolId, add_on: AddOnId) -> Result<()>;
    async fn record_uninstallation(&self, school: SchoolId, add_on: AddOnId) -> Result<()>;
}
```

## ModuleManagerRepository

```rust
#[async_trait]
pub trait ModuleManagerRepository: Send + Sync {
    async fn get(&self, id: ModuleManagerId) -> Result<Option<ModuleManager>>;
    async fn get_default(&self) -> Result<Option<ModuleManager>>;
    async fn list(&self) -> Result<Vec<ModuleManager>>;
    async fn insert(&self, m: &ModuleManager) -> Result<()>;
    async fn update(&self, m: &ModuleManager) -> Result<()>;
}
```

## ModuleStudentParentInfoRepository

```rust
#[async_trait]
pub trait ModuleStudentParentInfoRepository: Send + Sync {
    async fn get(&self, id: ModuleStudentParentInfoId) -> Result<Option<ModuleStudentParentInfo>>;
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<ModuleStudentParentInfo>>;
    async fn insert(&self, m: &ModuleStudentParentInfo) -> Result<()>;
    async fn update(&self, m: &ModuleStudentParentInfo) -> Result<()>;
    async fn delete(&self, id: ModuleStudentParentInfoId) -> Result<()>;
}
```

## TimeZoneRepository / CountryRepository / ContinentRepository

Each follows the same pattern: `get`, `list`, `insert`, `update`.
The repositories are global (no `school_id` filter).

## CurrencyRepository

```rust
#[async_trait]
pub trait CurrencyRepository: Send + Sync {
    async fn get(&self, id: CurrencyId) -> Result<Option<Currency>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Currency>>;
    async fn find_by_code(&self, school: SchoolId, code: &CurrencyCode) -> Result<Option<Currency>>;
    async fn insert(&self, c: &Currency) -> Result<()>;
    async fn update(&self, c: &Currency) -> Result<()>;
    async fn delete(&self, id: CurrencyId) -> Result<()>;
}
```

## LanguageRepository

```rust
#[async_trait]
pub trait LanguageRepository: Send + Sync {
    async fn get(&self, id: LanguageId) -> Result<Option<Language>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Language>>;
    async fn find_by_code(&self, school: SchoolId, code: &LanguageCode) -> Result<Option<Language>>;
    async fn insert(&self, l: &Language) -> Result<()>;
    async fn update(&self, l: &Language) -> Result<()>;
    async fn delete(&self, id: LanguageId) -> Result<()>;
}
```

## SocialMediaIconRepository

```rust
#[async_trait]
pub trait SocialMediaIconRepository: Send + Sync {
    async fn get(&self, id: SocialMediaIconId) -> Result<Option<SocialMediaIcon>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<SocialMediaIcon>>;
    async fn insert(&self, s: &SocialMediaIcon) -> Result<()>;
    async fn update(&self, s: &SocialMediaIcon) -> Result<()>;
    async fn delete(&self, id: SocialMediaIconId) -> Result<()>;
}
```

## HeaderMenuManagerRepository

```rust
#[async_trait]
pub trait HeaderMenuManagerRepository: Send + Sync {
    async fn get(&self, id: HeaderMenuManagerId) -> Result<Option<HeaderMenuManager>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<HeaderMenuManager>>;
    async fn list_for_theme(&self, school: SchoolId, theme: &str) -> Result<Vec<HeaderMenuManager>>;
    async fn insert(&self, m: &HeaderMenuManager) -> Result<()>;
    async fn update(&self, m: &HeaderMenuManager) -> Result<()>;
    async fn delete(&self, id: HeaderMenuManagerId) -> Result<()>;
}
```

## PhotoGalleryRepository / VideoGalleryRepository / VisitorRepository / ToDoRepository / InstructionRepository / ExpertTeacherRepository / FrontendPermissionRepository / AmountTransferRepository / PluginRepository / CommentRepository / CommentTagRepository / VideoUploadRepository

Each follows the same pattern: `get`, `list` (with optional
`school_id` or parent filter), `insert`, `update`, `delete`.

## PersonalAccessTokenRepository

```rust
#[async_trait]
pub trait PersonalAccessTokenRepository: Send + Sync {
    async fn get(&self, id: PersonalAccessTokenId) -> Result<Option<PersonalAccessToken>>;
    async fn find_by_hash(&self, hash: &TokenHash) -> Result<Option<PersonalAccessToken>>;
    async fn list_for_user(&self, user: UserId) -> Result<Vec<PersonalAccessToken>>;
    async fn insert(&self, t: &PersonalAccessToken) -> Result<()>;
    async fn update_last_used(&self, id: PersonalAccessTokenId, at: Timestamp) -> Result<()>;
    async fn delete(&self, id: PersonalAccessTokenId) -> Result<()>;
    async fn purge_expired(&self, before: Timestamp) -> Result<u64>;
}
```

## Indexes (recommended)

```sql
CREATE UNIQUE INDEX ux_schools_school_code ON schools (school_code);
CREATE UNIQUE INDEX ux_schools_domain ON schools (domain);
CREATE INDEX ix_schools_active_status ON schools (active_status);
CREATE INDEX ix_schools_region ON schools (region);
CREATE UNIQUE INDEX ux_users_school_id_email ON users (school_id, lower(email));
CREATE UNIQUE INDEX ux_users_school_id_username ON users (school_id, lower(username));
CREATE INDEX ix_users_school_id_phone ON users (school_id, phone_number);
CREATE INDEX ix_users_school_id_role_id ON users (school_id, role_id);
CREATE INDEX ix_users_school_id_usertype ON users (school_id, usertype);
CREATE INDEX ix_users_school_id_active_status ON users (school_id, active_status);
CREATE INDEX ix_user_otp_codes_user_id_expired ON user_otp_codes (user_id, expired_time);
CREATE UNIQUE INDEX ux_courses_school_id_title ON courses (school_id, title);
CREATE INDEX ix_courses_school_id_category ON courses (school_id, category_id);
CREATE UNIQUE INDEX ux_course_categories_school_id_name ON course_categories (school_id, lower(category_name));
CREATE UNIQUE INDEX ux_custom_fields_school_id_form_label ON custom_fields (school_id, form_name, label);
CREATE INDEX ix_custom_field_values_field_id ON custom_field_values (custom_field_id);
CREATE INDEX ix_custom_field_values_entity ON custom_field_values (entity_type, entity_id);
CREATE UNIQUE INDEX ux_custom_field_values_unique ON custom_field_values (custom_field_id, entity_type, entity_id);
CREATE UNIQUE INDEX ux_chart_of_accounts_school_id_head ON chart_of_accounts (school_id, lower(head));
CREATE UNIQUE INDEX ux_base_groups_school_id_name ON base_groups (school_id, name);
CREATE UNIQUE INDEX ux_base_setups_school_id_group_name ON base_setups (school_id, base_group_id, base_setup_name);
CREATE UNIQUE INDEX ux_modules_school_id_name ON modules (school_id, name);
CREATE INDEX ix_modules_school_id_order ON modules (school_id, "order");
CREATE INDEX ix_module_links_school_id_module ON module_links (school_id, module_id);
CREATE UNIQUE INDEX ux_currencies_school_id_code ON currencies (school_id, code);
CREATE UNIQUE INDEX ux_languages_school_id_code ON languages (school_id, code);
CREATE UNIQUE INDEX ux_countries_code ON countries (code);
CREATE UNIQUE INDEX ux_continents_code ON continents (code);
CREATE UNIQUE INDEX ux_personal_access_tokens_token ON personal_access_tokens (token);
CREATE INDEX ix_personal_access_tokens_tokenable ON personal_access_tokens (tokenable_type, tokenable_id);
CREATE INDEX ix_plugins_school_id ON plugins (school_id);
CREATE UNIQUE INDEX ux_plugins_school_id_short_code ON plugins (school_id, short_code);
CREATE INDEX ix_comments_school_id ON comments (school_id);
CREATE INDEX ix_visitors_school_id_date ON visitors (school_id, date);
CREATE INDEX ix_to_dos_school_id_status ON to_dos (school_id, complete_status);
CREATE INDEX ix_amount_transfers_school_id_date ON amount_transfers (school_id, transfer_date);
CREATE INDEX ix_video_uploads_school_id_class_section ON video_uploads (school_id, class_id, section_id);
```

The `school_id` predicate is mandatory for tenant isolation.

# Platform Domain — Services

Domain services encapsulate business logic that does not fit cleanly
in a single aggregate. They are stateless, sync, and pure (no I/O).

## SchoolService

```rust
pub struct SchoolService;

impl SchoolService {
    pub fn is_active(school: &School, now: NaiveDate) -> bool { ... }
    pub fn can_login(school: &School) -> bool { ... }
    pub fn is_within_plan_dates(school: &School, now: NaiveDate) -> bool { ... }
    pub fn build_tenant_context(school: &School, user: &User, correlation: CorrelationId) -> TenantContext { ... }
    pub fn school_capacity(school: &School, package: &SchoolPackage) -> Result<u32, CapacityError> { ... }
}
```

## UserService

```rust
pub struct UserService;

impl UserService {
    pub fn is_active(user: &User) -> bool { ... }
    pub fn is_administrator(user: &User) -> bool { ... }
    pub fn default_role_for(usertype: UserType) -> RoleName { ... }
    pub fn can_self_update(actor: &User, target: &User) -> bool { ... }
    pub fn effective_capabilities(user: &User, role: &Role, assignments: &[AssignPermission]) -> BTreeSet<Capability> { ... }
    pub fn unique_email_in_school(school: SchoolId, email: &EmailAddress, existing: &[User]) -> bool { ... }
    pub fn unique_phone_in_school(school: SchoolId, phone: &PhoneNumber, existing: &[User]) -> bool { ... }
    pub fn unique_username_in_school(school: SchoolId, username: &Username, existing: &[User]) -> bool { ... }
}
```

## OtpService

```rust
pub struct OtpService;

impl OtpService {
    pub fn is_expired(otp: &OtpCode, now: Timestamp) -> bool { ... }
    pub fn is_consumed(otp: &OtpCode) -> bool { ... }
    pub fn remaining_seconds(otp: &OtpCode, now: Timestamp) -> u32 { ... }
    pub fn verify(otp: &OtpCode, code: &OtpCode) -> Result<(), OtpError> { ... }
    pub fn should_resend(latest: &OtpCode, now: Timestamp) -> bool { ... }
}
```

## CourseService

```rust
pub struct CourseService;

impl CourseService {
    pub fn is_publishable(course: &Course, category: &CourseCategory) -> Result<(), ValidationError> { ... }
    pub fn can_delete(course: &Course, enrollments: &[CourseEnrollment]) -> Result<(), ConflictError> { ... }
}
```

## CustomFieldService

```rust
pub struct CustomFieldService;

impl CustomFieldService {
    pub fn validate_value(field: &CustomField, value: &str) -> Result<(), ValidationError> { ... }
    pub fn validate_length(field: &CustomField, value: &str) -> Result<(), ValidationError> { ... }
    pub fn validate_value_range(field: &CustomField, value: f64) -> Result<(), ValidationError> { ... }
    pub fn validate_select(field: &CustomField, value: &str) -> Result<(), ValidationError> { ... }
    pub fn can_delete(field: &CustomField, value_count: u64) -> Result<(), ConflictError> { ... }
}
```

## ChartOfAccountService

```rust
pub struct ChartOfAccountService;

impl ChartOfAccountService {
    pub fn is_expense(head: &ChartOfAccount) -> bool { ... }
    pub fn is_income(head: &ChartOfAccount) -> bool { ... }
    pub fn can_delete(head: &ChartOfAccount, posting_count: u64) -> Result<(), ConflictError> { ... }
}
```

## BaseSetupService

```rust
pub struct BaseSetupService;

impl BaseSetupService {
    pub fn unique_name_in_group(group: BaseGroupId, name: &str, existing: &[BaseSetup]) -> bool { ... }
    pub fn can_delete_group(group: &BaseGroup, setup_count: u64) -> Result<(), ConflictError> { ... }
}
```

## ModuleService

```rust
pub struct ModuleService;

impl ModuleService {
    pub fn is_enabled_for_school(school: SchoolId, module: &Module, school_modules: &[SchoolModule]) -> bool { ... }
    pub fn can_delete_module(module: &Module, link_count: u64) -> Result<(), ConflictError> { ... }
    pub fn can_delete_link(link: &ModuleLink, role_permission_count: u64) -> Result<(), ConflictError> { ... }
    pub fn reorder(modules: &mut [Module], new_positions: &BTreeMap<ModuleId, i32>) -> Result<(), ValidationError> { ... }
}
```

## AddOnService

```rust
pub struct AddOnService;

impl AddOnService {
    pub fn is_installed_for_school(school: SchoolId, add_on: &AddOn, installations: &[AddOnInstallation]) -> bool { ... }
    pub fn can_install(school: &School, add_on: &AddOn, plan: &SchoolPackage) -> Result<(), ConflictError> { ... }
}
```

## LocaleService

```rust
pub struct LocaleService;

impl LocaleService {
    pub fn format_currency(amount: Decimal, currency: &Currency) -> String { ... }
    pub fn parse_currency(input: &str, currency: &Currency) -> Result<Decimal, ParseError> { ... }
    pub fn format_date(date: NaiveDate, format: &str) -> String { ... }
    pub fn is_rtl(language: &Language) -> bool { ... }
}
```

## HeaderMenuService

```rust
pub struct HeaderMenuService;

impl HeaderMenuService {
    pub fn flatten_tree(items: &[HeaderMenuManager]) -> Vec<(HeaderMenuManagerId, Depth)> { ... }
    pub fn reorder(items: &mut [HeaderMenuManager], new_positions: &BTreeMap<HeaderMenuManagerId, i32>) -> Result<(), ValidationError> { ... }
}
```

## VisitorService

```rust
pub struct VisitorService;

impl VisitorService {
    pub fn is_on_premises(visitor: &Visitor) -> bool { ... }
    pub fn duration_minutes(visitor: &Visitor) -> Option<u32> { ... }
}
```

## ToDoService

```rust
pub struct ToDoService;

impl ToDoService {
    pub fn is_overdue(todo: &ToDo, now: NaiveDate) -> bool { ... }
    pub fn is_complete(todo: &ToDo) -> bool { ... }
}
```

## AmountTransferService

```rust
pub struct AmountTransferService;

impl AmountTransferService {
    pub fn validate_transfer(cmd: &CreateAmountTransferCommand) -> Result<(), ValidationError> { ... }
    pub fn is_reversal(transfer: &AmountTransfer, prior: &[AmountTransfer]) -> bool { ... }
}
```

## PluginService

```rust
pub struct PluginService;

impl PluginService {
    pub fn should_render_on_page(plugin: &Plugin, page: &str) -> bool { ... }
    pub fn validate_short_code(code: &str) -> Result<(), ValidationError> { ... }
}
```

## CommentService

```rust
pub struct CommentService;

impl CommentService {
    pub fn is_flagged(comment: &Comment) -> bool { ... }
    pub fn extract_mentions(comment: &Comment) -> Vec<UserId> { ... }
}
```

## PersonalAccessTokenService

```rust
pub struct PersonalAccessTokenService;

impl PersonalAccessTokenService {
    pub fn hash(plaintext: &str) -> TokenHash { ... }
    pub fn verify(plaintext: &str, hash: &TokenHash) -> bool { ... }
    pub fn is_expired(token: &PersonalAccessToken, now: Timestamp) -> bool { ... }
    pub fn has_ability(token: &PersonalAccessToken, capability: Capability) -> bool { ... }
    pub fn purge_expired(tokens: &mut Vec<PersonalAccessToken>, now: Timestamp) -> Vec<PersonalAccessToken> { ... }
}
```

## Policy: UniqueSchoolFields

```rust
pub struct UniqueSchoolFields;

impl Policy<CreateSchoolCommand> for UniqueSchoolFields {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &CreateSchoolCommand) -> Outcome { ... }
}
```

## Policy: UserUniquenessInSchool

```rust
pub struct UserUniquenessInSchool;

impl Policy<RegisterUserCommand> for UserUniquenessInSchool {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &RegisterUserCommand) -> Outcome { ... }
}
```

## Specification: ActiveUsers

```rust
pub struct ActiveUsers;

impl Specification<User> for ActiveUsers {
    fn is_satisfied_by(&self, u: &User) -> bool { ... }
}
```

Composed with `And`, `Or`, `Not` for queries.

## Specification: PublishedCourses

```rust
pub struct PublishedCourses;

impl Specification<Course> for PublishedCourses {
    fn is_satisfied_by(&self, c: &Course) -> bool { ... }
}
```

## Cross-Domain Coordinator

The platform domain publishes events for every state change. RBAC,
settings, and academic domains subscribe to bootstrap their data.
There is no service-to-service call from platform to any other
domain.

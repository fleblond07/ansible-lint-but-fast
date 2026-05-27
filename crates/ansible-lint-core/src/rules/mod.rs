pub mod yaml;
pub mod name;
pub mod fqcn;
pub mod no_changed_when;
pub mod risky_shell_pipe;
pub mod command_instead_of_module;
pub mod no_log_password;
pub mod var_naming;
pub mod partial_become;
pub mod package_latest;
pub mod risky_file_permissions;
pub mod deprecated_bare_vars;
pub mod no_free_form;

// Phase 2 rules
pub mod load_failure;
pub mod empty_string_compare;
pub mod ignore_errors;
pub mod key_order;
pub mod literal_compare;
pub mod meta;
pub mod no_handler;
pub mod no_jinja_when;
pub mod no_relative_paths;
pub mod only_builtins;
pub mod role_name;
pub mod run_once;

use crate::rule::Rule;

/// Returns all built-in rules.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // yaml/* rules
        Box::new(yaml::truthy::YamlTruthyRule),
        Box::new(yaml::line_length::YamlLineLengthRule),
        Box::new(yaml::trailing_spaces::YamlTrailingSpacesRule),
        Box::new(yaml::indentation::YamlIndentationRule),
        Box::new(yaml::empty_lines::YamlEmptyLinesRule),
        Box::new(yaml::document_start::YamlDocumentStartRule),
        Box::new(yaml::new_line_at_eof::YamlNewLineAtEofRule),
        Box::new(yaml::comments::YamlCommentsRule),
        Box::new(yaml::key_duplicates::YamlKeyDuplicatesRule),
        Box::new(yaml::octal_values::YamlOctalValuesRule),
        Box::new(yaml::brackets::YamlBracketsRule),
        Box::new(yaml::colons::YamlColonsRule),
        Box::new(yaml::commas::YamlCommasRule),
        // name/* rules
        Box::new(name::missing::NameMissingRule),
        Box::new(name::casing::NameCasingRule),
        Box::new(name::template::NameTemplateRule),
        // task-level rules (Phase 1)
        Box::new(fqcn::FqcnActionRule),
        Box::new(no_changed_when::NoChangedWhenRule),
        Box::new(risky_shell_pipe::RiskyShellPipeRule),
        Box::new(command_instead_of_module::CommandInsteadOfModuleRule),
        Box::new(command_instead_of_module::CommandInsteadOfShellRule),
        Box::new(no_log_password::NoLogPasswordRule),
        Box::new(var_naming::VarNamingRule),
        Box::new(partial_become::PartialBecomeRule),
        Box::new(package_latest::PackageLatestRule),
        Box::new(risky_file_permissions::RiskyFilePermissionsRule),
        Box::new(deprecated_bare_vars::DeprecatedBareVarsRule),
        Box::new(no_free_form::NoFreeFormRule),
        // task-level rules (Phase 2)
        Box::new(no_jinja_when::NoJinjaWhenRule),
        Box::new(ignore_errors::IgnoreErrorsRule),
        Box::new(literal_compare::LiteralCompareRule),
        Box::new(empty_string_compare::EmptyStringCompareRule),
        Box::new(key_order::KeyOrderRule),
        Box::new(no_relative_paths::NoRelativePathsRule),
        Box::new(only_builtins::OnlyBuiltinsRule),
        Box::new(no_handler::NoHandlerRule),
        Box::new(role_name::RoleNameRule),
        Box::new(run_once::RunOnceRule),
        // meta / load-failure rules
        Box::new(load_failure::LoadFailureRule),
        Box::new(meta::MetaNoInfoRule),
        Box::new(meta::MetaIncorrectRule),
        Box::new(meta::MetaNoTagsRule),
    ]
}

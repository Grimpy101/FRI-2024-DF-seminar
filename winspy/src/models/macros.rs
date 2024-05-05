#[macro_export]
macro_rules! try_get_row {
    ($row:expr, $column_name:expr) => {
        miette::IntoDiagnostic::into_diagnostic(sqlx::Row::try_get($row, $column_name))
    };
}

#[macro_export]
macro_rules! require_some {
    ($query_result_field:expr, $field_name:expr) => {
        $query_result_field.ok_or_else(|| {
            miette::miette!(
                "Failed to extract field {}: did not expect None.",
                $field_name
            )
        })
    };
}

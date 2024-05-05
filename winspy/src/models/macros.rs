#[macro_export]
macro_rules! try_get_row {
    ($row:expr, $column_name:expr) => {
        miette::IntoDiagnostic::into_diagnostic($row.try_get($column_name))
    };
}

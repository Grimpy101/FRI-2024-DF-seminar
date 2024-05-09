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

#[macro_export]
macro_rules! require_json_object {
    ($value:expr) => {{
        let Some(object) = $value.as_object() else {
            return None;
        };

        object
    }};
}

#[macro_export]
macro_rules! extract_value_from_json_object {
    ($json_object:expr, $key:expr => object) => {{
        let Some(string) = $json_object.get($key).and_then(|value| value.as_object()) else {
            return None;
        };

        string
    }};

    ($json_object:expr, $key:expr => str) => {{
        let Some(string) = $json_object.get($key).and_then(|value| value.as_str()) else {
            return None;
        };

        string
    }};

    ($json_object:expr, $key:expr => i64) => {{
        let Some(string) = $json_object.get($key).and_then(|value| value.as_i64()) else {
            return None;
        };

        string
    }};
}

#[macro_export]
macro_rules! require_json_object_value {
    ($value:expr, $key:expr) => {{
        let Some(object) = $value.as_object() else {
            None
        };
        object.get()
    }};
}

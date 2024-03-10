use super::dto::{ColumnData, DataType};

pub trait ToSql {
    fn to_sql(&self) -> ColumnData;
}

impl ToSql for i32 {
    fn to_sql(&self) -> ColumnData {
        ColumnData {
            data: self.to_string().into(),
            data_type: DataType::INTEGER,
        }
    }
}

impl ToSql for String {
    fn to_sql(&self) -> ColumnData {
        ColumnData {
            data: self.clone(),
            data_type: DataType::TEXT,
        }
    }
}

impl ToSql for f64 {
    fn to_sql(&self) -> ColumnData {
        ColumnData {
            data: self.to_string().into(),
            data_type: DataType::FLOAT,
        }
    }
}

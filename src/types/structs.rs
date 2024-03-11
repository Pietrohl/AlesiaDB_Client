use super::dto::{ColumnData, DataType};

pub trait ToSql {
    fn to_sql(&self) -> ColumnData;
}

impl Into<i32> for ColumnData {
    fn into(self) -> i32 {
        self.data.parse().unwrap()
    }
}

impl Into<String> for ColumnData {
    fn into(self) -> String {
        self.data
    }
}

impl Into<f64> for ColumnData {
    fn into(self) -> f64 {
        self.data.parse().unwrap()
    }
}

impl Into<DataType> for ColumnData {
    fn into(self) -> DataType {
        self.data_type
    }
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



impl rusqlite::types::ToSql for ColumnData {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self.data_type {
            DataType::INTEGER => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer(
                self.data.parse().unwrap(),
            ))),
            DataType::FLOAT => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Real(
                self.data.parse().unwrap(),
            ))),
            DataType::TEXT => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Text(
                self.data.clone(),
            ))),
            DataType::BLOB => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Blob(
                self.data.as_bytes().to_vec(),
            ))),
            _ => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Null)),
        }
    }
}